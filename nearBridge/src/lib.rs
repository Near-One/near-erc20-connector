use near_plugins::{
    access_control, access_control_any, pause, AccessControlRole, AccessControllable, Pausable,
    Upgradable,
};
/**
* Bridge for Near Native token
*/
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedSet;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, Promise,
    PromiseOrValue, PublicKey, ONE_YOCTO,
};
use prover::ext_prover;
pub use prover::{get_eth_address, is_valid_eth_address, EthAddress, Proof};
pub use transfer_to_near_event::TransferToNearInitiatedEvent;

use crate::prover::{parse_recipient, Recipient};

pub mod prover;
mod transfer_to_near_event;

/// Gas to call finalise method.
const FINISH_FINALISE_GAS: Gas = Gas(Gas::ONE_TERA.0 * 100);
/// Gas to call verify_log_entry on prover.
const VERIFY_LOG_ENTRY_GAS: Gas = Gas(Gas::ONE_TERA.0 * 50);
const WNEAR_DEPOSIT_GAS: Gas = Gas(Gas::ONE_TERA.0 * 10);
const WNEAR_STORAGE_DEPOSIT_GAS: Gas = Gas(Gas::ONE_TERA.0 * 5);
const FT_TRANSFER_CALL_GAS: Gas = Gas(Gas::ONE_TERA.0 * 80);
const FT_TRANSFER_GAS: Gas = Gas(Gas::ONE_TERA.0 * 5);

const WNEAR_STORAGE_KEY: &[u8] = b"wnear";

pub type Mask = u128;

#[derive(Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum ResultType {
    MigrateNearToEthereum {
        amount: Balance,
        recipient: EthAddress,
    },
}

#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    DAO,
    PauseManager,
    UnrestrictedMigrateToEthereum,
    UnrestrictedFinaliseEthToNearTransfer,
    UpgradableCodeStager,
    UpgradableCodeDeployer,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault, Pausable, Upgradable)]
#[access_control(role_type(Role))]
#[pausable(manager_roles(Role::PauseManager, Role::DAO))]
#[upgradable(access_control_roles(
    code_stagers(Role::UpgradableCodeStager, Role::DAO),
    code_deployers(Role::UpgradableCodeDeployer, Role::DAO),
    duration_initializers(Role::DAO),
    duration_update_stagers(Role::DAO),
    duration_update_appliers(Role::DAO),
))]
pub struct NearBridge {
    /// The account of the prover that we can use to prove
    pub prover_account: AccountId,

    /// Address of the associated Ethereum eNear ERC20 contract.
    pub e_near_address: EthAddress,

    /// Hashes of the events that were already used.
    pub used_events: UnorderedSet<Vec<u8>>,

    /// Mask determining all paused functions
    #[deprecated]
    paused: Mask,
}

#[near_bindgen]
impl NearBridge {
    #[init]
    #[payable]
    pub fn new(
        prover_account: AccountId,
        e_near_address: String,
        wnear_account: AccountId,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        #[allow(deprecated)]
        let mut contract = Self {
            prover_account,
            e_near_address: get_eth_address(e_near_address),
            used_events: UnorderedSet::new(b"u".to_vec()),
            paused: Mask::default(),
        };

        contract.acl_init_super_admin(env::predecessor_account_id());
        contract.acl_grant_role(Role::DAO.into(), env::predecessor_account_id());
        contract.set_wnear_account_id(wnear_account);
        contract
    }

    /// Deposit NEAR for bridging from the predecessor account ID
    /// Requirements:
    /// * `eth_recipient` must be a valid eth account
    /// * `amount` must be a positive integer
    /// * Caller of the method has to attach deposit enough to cover:
    ///   * The `amount` of Near tokens being bridged, and
    ///   * The storage difference at the fixed storage price defined in the contract.
    #[payable]
    #[result_serializer(borsh)]
    // todo: how much GAS is required to execute this method with sending the tokens back and ensure we have enough
    #[pause(except(roles(Role::DAO, Role::UnrestrictedMigrateToEthereum)))]
    pub fn migrate_to_ethereum(&mut self, eth_recipient: String) -> ResultType {
        // Predecessor must attach Near to migrate to ETH
        let attached_deposit = env::attached_deposit();
        if attached_deposit == 0 {
            env::panic_str("Attached deposit must be greater than zero");
        }

        // If the method is paused or the eth recipient address is invalid, then we need to:
        //  1) Return the attached deposit
        //  2) Panic and tell the user why
        let eth_recipient_clone = eth_recipient.clone();
        if !is_valid_eth_address(eth_recipient_clone) {
            env::panic_str("ETH address is invalid");
        }

        ResultType::MigrateNearToEthereum {
            amount: attached_deposit,
            recipient: get_eth_address(eth_recipient),
        }
    }

    #[payable]
    #[pause(except(roles(Role::DAO, Role::UnrestrictedFinaliseEthToNearTransfer)))]
    pub fn finalise_eth_to_near_transfer(&mut self, #[serializer(borsh)] proof: Proof) -> Promise {
        let event = TransferToNearInitiatedEvent::from_log_entry_data(&proof.log_entry_data);
        assert_eq!(
            event.e_near_address,
            self.e_near_address,
            "Event's address {} does not match locker address of this token {}",
            hex::encode(&event.e_near_address),
            hex::encode(&self.e_near_address),
        );

        let proof_1 = proof.clone();

        ext_prover::ext(self.prover_account.clone())
            .with_static_gas(VERIFY_LOG_ENTRY_GAS)
            .verify_log_entry(
                proof.log_index,
                proof.log_entry_data,
                proof.receipt_index,
                proof.receipt_data,
                proof.header_data,
                proof.proof,
                false, // Do not skip bridge call. This is only used for development and diagnostics.
            )
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(FINISH_FINALISE_GAS)
                    .with_attached_deposit(env::attached_deposit())
                    .finish_eth_to_near_transfer(event.recipient, event.amount, proof_1),
            )
    }

    /// Finish depositing once the proof was successfully validated. Can only be called by the contract
    /// itself.
    #[payable]
    pub fn finish_eth_to_near_transfer(
        &mut self,
        #[callback]
        #[serializer(borsh)]
        verification_success: bool,
        #[serializer(borsh)] new_owner_id: String,
        #[serializer(borsh)] amount: Balance,
        #[serializer(borsh)] proof: Proof,
    ) -> Promise {
        near_sdk::assert_self();
        assert!(verification_success, "Failed to verify the proof");

        let required_deposit = self.record_proof(&proof);
        if env::attached_deposit() < required_deposit {
            env::panic_str("Attached deposit is not sufficient to record proof");
        }

        let Recipient { target, message } = parse_recipient(&new_owner_id)
            .unwrap_or_else(|| env::panic_str("Failed to parse recipient"));

        match message {
            Some(message) => {
                let wnear_account_id = self
                    .get_wnear_account_id()
                    .unwrap_or_else(|| env::panic_str("WNear address hasn't been set"));
                ext_wnear_token::ext(wnear_account_id.clone())
                    .with_static_gas(WNEAR_DEPOSIT_GAS)
                    .with_attached_deposit(amount)
                    .near_deposit()
                    .then(
                        ext_wnear_token::ext(wnear_account_id)
                            .with_static_gas(FT_TRANSFER_CALL_GAS)
                            .with_attached_deposit(ONE_YOCTO)
                            .ft_transfer_call(target, amount.into(), None, message),
                    )
            }
            None => Promise::new(target).transfer(amount),
        }
    }

    pub fn get_avialable_balance(&self) -> U128 {
        U128(
            env::account_balance()
                - env::attached_deposit()
                - env::storage_byte_cost() * env::storage_usage() as u128,
        )
    }

    #[access_control_any(roles(Role::DAO))]
    #[payable]
    pub fn send_to_omni_bridge(&mut self, omni_bridge: AccountId) -> Promise {
        let amount = self.get_avialable_balance().0;
        let wnear_account_id = self
            .get_wnear_account_id()
            .unwrap_or_else(|| env::panic_str("WNear address hasn't been set"));

        ext_wnear_token::ext(wnear_account_id.clone())
            .with_static_gas(WNEAR_DEPOSIT_GAS)
            .with_attached_deposit(amount)
            .near_deposit()
            .then(
                ext_wnear_token::ext(wnear_account_id)
                    .with_static_gas(FT_TRANSFER_GAS)
                    .with_attached_deposit(ONE_YOCTO)
                    .ft_transfer(omni_bridge, amount.into(), None),
            )
    }

    /// Checks whether the provided proof is already used
    pub fn is_used_proof(&self, #[serializer(borsh)] proof: Proof) -> bool {
        self.used_events.contains(&proof.get_key())
    }

    /// Record proof to make sure it is not re-used later for anther deposit.
    fn record_proof(&mut self, proof: &Proof) -> Balance {
        // TODO: Instead of sending the full proof (clone only relevant parts of the Proof)
        //       log_index / receipt_index / header_data
        near_sdk::assert_self();
        let initial_storage = env::storage_usage();
        let proof_key = proof.get_key();
        assert!(
            !self.used_events.contains(&proof_key),
            "Event cannot be reused for depositing."
        );
        self.used_events.insert(&proof_key);
        let current_storage = env::storage_usage();

        let required_deposit =
            Balance::from(current_storage - initial_storage) * env::storage_byte_cost();

        required_deposit
    }

    #[payable]
    #[access_control_any(roles(Role::DAO))]
    pub fn set_wnear_account_id(&mut self, wnear: AccountId) -> Promise {
        env::storage_write(WNEAR_STORAGE_KEY, &wnear.try_to_vec().unwrap());

        ext_wnear_token::ext(wnear)
            .with_static_gas(WNEAR_STORAGE_DEPOSIT_GAS)
            .with_attached_deposit(env::attached_deposit())
            .storage_deposit(env::current_account_id())
    }

    pub fn get_wnear_account_id(&self) -> Option<AccountId> {
        AccountId::try_from_slice(&env::storage_read(WNEAR_STORAGE_KEY)?).ok()
    }

    #[access_control_any(roles(Role::DAO))]
    pub fn attach_full_access_key(&self, public_key: PublicKey) -> Promise {
        Promise::new(env::current_account_id()).add_full_access_key(public_key)
    }

    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }
}

#[ext_contract(ext_self)]
pub trait ExtNearBridge {
    #[result_serializer(borsh)]
    fn finish_eth_to_near_transfer(
        &mut self,
        #[callback]
        #[serializer(borsh)]
        verification_success: bool,
        #[serializer(borsh)] new_owner_id: String,
        #[serializer(borsh)] amount: Balance,
        #[serializer(borsh)] proof: Proof,
    ) -> Promise;
}

#[ext_contract(ext_wnear_token)]
pub trait ExtWNearToken {
    fn ft_transfer_call(
        &self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128>;

    fn ft_transfer(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    ) -> PromiseOrValue<U128>;

    fn near_deposit(&self);
    fn storage_deposit(&self, account_id: AccountId);
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::test_utils::test_env::bob;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    use super::*;
    use std::convert::TryInto;
    use uint::rustc_hex::FromHex;

    macro_rules! inner_set_env {
        ($builder:ident) => {
            $builder
        };

        ($builder:ident, $key:ident:$value:expr $(,$key_tail:ident:$value_tail:expr)*) => {
            {
               $builder.$key($value.try_into().unwrap());
               inner_set_env!($builder $(,$key_tail:$value_tail)*)
            }
        };
    }

    macro_rules! set_env {
        ($($key:ident:$value:expr),* $(,)?) => {
            let mut builder = VMContextBuilder::new();
            let mut builder = &mut builder;
            builder = inner_set_env!(builder, $($key: $value),*);
            testing_env!(builder.build());
        };
    }

    fn alice_near_account() -> AccountId {
        "alice.near".parse().unwrap()
    }
    fn prover_near_account() -> AccountId {
        "prover".parse().unwrap()
    }
    fn wnear_near_account() -> AccountId {
        "wrap.near".parse().unwrap()
    }
    fn e_near_eth_address() -> String {
        "68a3637ba6e75c0f66b61a42639c4e9fcd3d4824".to_string()
    }
    fn alice_eth_address() -> String {
        "25ac31a08eba29067ba4637788d1dbfb893cebf1".to_string()
    }
    fn invalid_eth_address() -> String {
        "25Ac31A08EBA29067Ba4637788d1DbFB893cEBf".to_string()
    }

    fn sample_proof() -> Proof {
        Proof {
            log_index: 0,
            log_entry_data: vec![],
            receipt_index: 0,
            receipt_data: vec![],
            header_data: vec![],
            proof: vec![],
        }
    }

    fn create_proof(e_near: String) -> Proof {
        let event_data = TransferToNearInitiatedEvent {
            e_near_address: e_near
                .from_hex::<Vec<_>>()
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),
            sender: "00005474e89094c44da98b954eedeac495271d0f".to_string(),
            amount: 1000,
            recipient: "123".to_string(),
        };

        Proof {
            log_index: 0,
            log_entry_data: event_data.to_log_entry_data(),
            receipt_index: 0,
            receipt_data: vec![],
            header_data: vec![],
            proof: vec![],
        }
    }

    #[test]
    fn can_migrate_near_to_eth_with_valid_params() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract = NearBridge::new(
            prover_near_account(),
            e_near_eth_address(),
            wnear_near_account(),
        );

        // lets deposit 1 Near
        let deposit_amount = 1_000_000_000_000_000_000_000_000u128;
        set_env!(
            predecessor_account_id: alice_near_account(),
            attached_deposit: deposit_amount,
        );

        contract.migrate_to_ethereum(alice_eth_address());
    }

    #[test]
    #[should_panic]
    fn migrate_near_to_eth_panics_when_attached_deposit_is_zero() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract = NearBridge::new(
            prover_near_account(),
            e_near_eth_address(),
            wnear_near_account(),
        );

        contract.migrate_to_ethereum(alice_eth_address());
    }

    #[test]
    #[should_panic]
    fn migrate_near_to_eth_panics_when_contract_is_paused() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract = NearBridge::new(
            prover_near_account(),
            e_near_eth_address(),
            wnear_near_account(),
        );

        contract.pa_pause_feature("migrate_to_ethereum".to_owned());

        // lets deposit 1 Near
        let deposit_amount = 1_000_000_000_000_000_000_000_000u128;
        set_env!(
            predecessor_account_id: bob(),
            attached_deposit: deposit_amount,
        );

        contract.migrate_to_ethereum(alice_eth_address());
    }

    #[test]
    #[should_panic]
    fn migrate_near_to_eth_panics_when_eth_address_is_invalid() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract = NearBridge::new(
            prover_near_account(),
            e_near_eth_address(),
            wnear_near_account(),
        );

        contract.migrate_to_ethereum(invalid_eth_address());
    }

    #[test]
    #[should_panic]
    fn finalise_eth_to_near_transfer_panics_when_contract_is_paused() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract = NearBridge::new(
            prover_near_account(),
            e_near_eth_address(),
            wnear_near_account(),
        );

        contract.pa_pause_feature("finalise_eth_to_near_transfer".to_owned());

        contract.finalise_eth_to_near_transfer(sample_proof());
    }

    #[test]
    #[should_panic]
    fn finalise_eth_to_near_transfer_panics_when_event_originates_from_wrong_contract() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract = NearBridge::new(
            prover_near_account(),
            e_near_eth_address(),
            wnear_near_account(),
        );

        contract.finalise_eth_to_near_transfer(create_proof(alice_eth_address()));
    }

    #[test]
    #[should_panic(expected = "Attached deposit is not sufficient to record proof")]
    fn finish_eth_to_near_transfer_panics_if_attached_deposit_is_not_sufficient_to_record_proof() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract = NearBridge::new(
            prover_near_account(),
            e_near_eth_address(),
            wnear_near_account(),
        );

        contract.finish_eth_to_near_transfer(
            true,
            bob().to_string(),
            10,
            create_proof(e_near_eth_address()),
        );
    }

    #[test]
    fn finalise_eth_to_near_transfer_works_with_valid_params() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract = NearBridge::new(
            prover_near_account(),
            e_near_eth_address(),
            wnear_near_account(),
        );

        // Alice deposit 1 Near to migrate to eth
        let deposit_amount = 1_000_000_000_000_000_000_000_000u128;
        set_env!(
            predecessor_account_id: alice_near_account(),
            attached_deposit: deposit_amount,
        );

        contract.migrate_to_ethereum(alice_eth_address());

        // todo adjust attached deposit down

        // Lets suppose Alice migrates back
        contract.finalise_eth_to_near_transfer(create_proof(e_near_eth_address()));

        // todo asserts i.e. that alice has received the 1 near back etc.
    }

    #[test]
    fn test_set_wnear_account_id() {
        set_env!(
            predecessor_account_id: alice_near_account(),
            signer_account_id: alice_near_account()
        );

        let mut contract = NearBridge::new(
            prover_near_account(),
            e_near_eth_address(),
            "old_wnear_near.near".parse().unwrap(),
        );

        contract.set_wnear_account_id(wnear_near_account());

        assert_eq!(
            contract.get_wnear_account_id().unwrap(),
            wnear_near_account()
        );
    }
}
