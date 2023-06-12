/**
* Bridge for Near Native token
*/
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedSet;
use near_sdk::{env, ext_contract, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, Promise};

use admin_controlled::{AdminControlled, Mask};

near_sdk::setup_alloc!();

use prover::ext_prover;
pub use prover::{get_eth_address, is_valid_eth_address, EthAddress, Proof};
pub use transfer_to_near_event::TransferToNearInitiatedEvent;

pub mod prover;
mod transfer_to_near_event;

/// Gas to call finalise method.
const FINISH_FINALISE_GAS: Gas = 50_000_000_000_000;

const NO_DEPOSIT: Balance = 0;

// Fee can be set in 6 decimal precision (10% -> 0.1 * 10e6)
const FEE_DECIMAL_PRECISION: u128 = 1_000_000;

/// Gas to call verify_log_entry on prover.
const VERIFY_LOG_ENTRY_GAS: Gas = 50_000_000_000_000;

const PAUSE_MIGRATE_TO_ETH: Mask = 1 << 0;
const PAUSE_ETH_TO_NEAR_TRANSFER: Mask = 1 << 1;

#[derive(Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum ResultType {
    MigrateNearToEthereum {
        amount: Balance,
        recipient: EthAddress,
    },
}

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone, PartialEq, Eq)]
pub enum FeeType {
    Deposit,
    Withdraw,
}

#[derive(Default, BorshDeserialize, BorshSerialize, Debug, Clone, Copy, PartialEq)]
pub struct TransferFeePercentage {
    near_to_eth: u128,
    eth_to_near: u128,
}

#[derive(Default, BorshDeserialize, BorshSerialize, Debug, Clone, Copy, PartialEq)]
pub struct FeeBounds {
    lower_bound: u128,
    upper_bound: u128,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct NearBridge {
    /// The account of the prover that we can use to prove
    pub prover_account: AccountId,

    /// Address of the associated Ethereum eNear ERC20 contract.
    pub e_near_address: EthAddress,

    /// Hashes of the events that were already used.
    pub used_events: UnorderedSet<Vec<u8>>,

    /// Mask determining all paused functions
    paused: Mask,

    /// Fee percentage for both side transfer
    pub transfer_fee_percentage: TransferFeePercentage,

    /// deposit fee bounds for transfer near -> eth
    pub deposit_fee_bounds: FeeBounds,

    /// withdraw fee bounds for transfer eth -> near
    pub withdraw_fee_bounds: FeeBounds,

    /// Owner's account id.
    pub owner_id: AccountId,

    // Cumulative fee amount for near -> eth and eth -> near transfers
    pub cumulative_fee_amount: u128,
}

#[near_bindgen]
impl NearBridge {
    #[init]
    pub fn new(prover_account: AccountId, e_near_address: String, owner_id: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            prover_account,
            e_near_address: get_eth_address(e_near_address),
            used_events: UnorderedSet::new(b"u".to_vec()),
            paused: Mask::default(),
            transfer_fee_percentage: TransferFeePercentage {
                near_to_eth: 0,
                eth_to_near: 0,
            },
            deposit_fee_bounds: FeeBounds {
                lower_bound: 0,
                upper_bound: 0,
            },
            withdraw_fee_bounds: FeeBounds {
                lower_bound: 0,
                upper_bound: 0,
            },
            owner_id,
            cumulative_fee_amount: 0,
        }
    }

    fn is_owner(&self) -> bool {
        self.owner_id == env::predecessor_account_id()
            || env::current_account_id() == env::predecessor_account_id()
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
    pub fn migrate_to_ethereum(&mut self, eth_recipient: String) -> ResultType {
        self.check_not_paused(PAUSE_MIGRATE_TO_ETH);

        // Predecessor must attach Near to migrate to ETH
        let attached_deposit = env::attached_deposit();
        if attached_deposit == 0 {
            env::panic(b"Attached deposit must be greater than zero");
        }

        // If the method is paused or the eth recipient address is invalid, then we need to:
        //  1) Return the attached deposit
        //  2) Panic and tell the user why
        let eth_recipient_clone = eth_recipient.clone();
        if !is_valid_eth_address(eth_recipient_clone) {
            env::panic(b"ETH address is invalid");
        }

        let transfer_fee_percentage = self.get_transfer_fee_percentage();

        let mut fee_amount =
            (attached_deposit * transfer_fee_percentage.near_to_eth) / FEE_DECIMAL_PRECISION;
        fee_amount = self.check_fee_bounds(fee_amount, FeeType::Deposit);
        let amount_to_transfer = attached_deposit - fee_amount;
        self.cumulative_fee_amount += fee_amount;

        ResultType::MigrateNearToEthereum {
            amount: amount_to_transfer,
            recipient: get_eth_address(eth_recipient),
        }
    }

    pub fn set_transfer_fee_percentage(&mut self, near_to_eth: u128, eth_to_near: u128) {
        assert!(
            self.is_owner(),
            "Only owner can set the transfer fee percentage"
        );
        self.transfer_fee_percentage = TransferFeePercentage {
            near_to_eth,
            eth_to_near,
        };
    }

    /// Fee bounds for near -> eth transfers [Deposit]
    pub fn set_deposit_fee_bounds(&mut self, lower_bound: u128, upper_bound: u128) {
        assert!(self.is_owner(), "Only owner can set the deposit fee bounds");
        assert!(
            lower_bound < upper_bound,
            "Lower bound can't be less than upper bound value"
        );
        self.deposit_fee_bounds = FeeBounds {
            lower_bound,
            upper_bound,
        };
    }

    /// Fee bounds for eth -> near transfer [Withdraw]
    pub fn set_withdraw_fee_bounds(&mut self, lower_bound: u128, upper_bound: u128) {
        assert!(
            self.is_owner(),
            "Only owner can set the withdraw fee bounds"
        );
        assert!(
            lower_bound < upper_bound,
            "Lower bound can't be less than upper bound value"
        );
        self.withdraw_fee_bounds = FeeBounds {
            lower_bound,
            upper_bound,
        };
    }

    pub fn get_transfer_fee_percentage(&self) -> TransferFeePercentage {
        self.transfer_fee_percentage
    }

    pub fn get_deposit_fee_bounds(&self) -> FeeBounds {
        self.deposit_fee_bounds
    }

    pub fn get_withdraw_fee_bounds(&self) -> FeeBounds {
        self.withdraw_fee_bounds
    }

    fn check_fee_bounds(&self, amount: u128, fee_type: FeeType) -> u128 {
        let fee_bounds = if fee_type == FeeType::Deposit {
            self.get_deposit_fee_bounds()
        } else {
            self.get_withdraw_fee_bounds()
        };

        if amount < fee_bounds.lower_bound {
            return fee_bounds.lower_bound;
        } else if amount > fee_bounds.upper_bound && fee_bounds.upper_bound != 0 {
            return fee_bounds.upper_bound;
        }
        amount
    }

    #[payable]
    pub fn finalise_eth_to_near_transfer(&mut self, #[serializer(borsh)] proof: Proof) {
        self.check_not_paused(PAUSE_ETH_TO_NEAR_TRANSFER);

        let event = TransferToNearInitiatedEvent::from_log_entry_data(&proof.log_entry_data);
        assert_eq!(
            event.e_near_address,
            self.e_near_address,
            "Event's address {} does not match locker address of this token {}",
            hex::encode(&event.e_near_address),
            hex::encode(&self.e_near_address),
        );

        let proof_1 = proof.clone();

        ext_prover::verify_log_entry(
            proof.log_index,
            proof.log_entry_data,
            proof.receipt_index,
            proof.receipt_data,
            proof.header_data,
            proof.proof,
            false, // Do not skip bridge call. This is only used for development and diagnostics.
            &self.prover_account,
            NO_DEPOSIT,
            VERIFY_LOG_ENTRY_GAS,
        )
        .then(ext_self::finish_eth_to_near_transfer(
            event.recipient,
            event.amount,
            proof_1,
            &env::current_account_id(),
            env::attached_deposit(),
            FINISH_FINALISE_GAS,
        ));
    }

    /// Finish depositing once the proof was successfully validated. Can only be called by the contract
    /// itself.
    #[payable]
    pub fn finish_eth_to_near_transfer(
        &mut self,
        #[callback]
        #[serializer(borsh)]
        verification_success: bool,
        #[serializer(borsh)] new_owner_id: AccountId,
        #[serializer(borsh)] amount: Balance,
        #[serializer(borsh)] proof: Proof,
    ) {
        near_sdk::assert_self();
        assert!(verification_success, "Failed to verify the proof");
        let transfer_fee_percentage = self.get_transfer_fee_percentage();

        let required_deposit = self.record_proof(&proof);
        if env::attached_deposit() < required_deposit {
            env::panic(b"Attached deposit is not sufficient to record proof");
        }

        let mut fee_amount = (amount * transfer_fee_percentage.eth_to_near) / FEE_DECIMAL_PRECISION;
        fee_amount = self.check_fee_bounds(fee_amount, FeeType::Withdraw);
        let amount_to_transfer = amount - fee_amount;
        self.cumulative_fee_amount += fee_amount;

        // Amount after fee deductions is transfered to new_owner
        Promise::new(new_owner_id).transfer(amount_to_transfer);
    }

    pub fn claim_fees(&mut self, amount: u128) {
        assert!(self.is_owner(), "Only owner can claim the fee");
        assert!(
            self.cumulative_fee_amount > 0 && amount <= self.cumulative_fee_amount,
            "Invalid fee amount to claim"
        );
        Promise::new(env::predecessor_account_id()).transfer(amount);
        self.cumulative_fee_amount -= amount;
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
}

#[ext_contract(ext_self)]
pub trait ExtNearBridge {
    #[result_serializer(borsh)]
    fn finish_eth_to_near_transfer(
        &mut self,
        #[callback]
        #[serializer(borsh)]
        verification_success: bool,
        #[serializer(borsh)] new_owner_id: AccountId,
        #[serializer(borsh)] amount: Balance,
        #[serializer(borsh)] proof: Proof,
    ) -> Promise;
}

admin_controlled::impl_admin_controlled!(NearBridge, paused);

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, MockedBlockchain};

    use super::*;
    use near_sdk::env::sha256;
    use std::convert::TryInto;
    use std::panic;
    use uint::rustc_hex::{FromHex, ToHex};

    const UNPAUSE_ALL: Mask = 0;

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
        "alice.near".to_string()
    }
    fn owner_account() -> AccountId {
        "owner.near".to_string()
    }

    fn contract_account() -> AccountId {
        "contract.near".to_string()
    }

    fn user_near_account() -> AccountId {
        "user.near".to_string()
    }
    fn prover_near_account() -> AccountId {
        "prover".to_string()
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

    /// Generate a valid ethereum address
    fn ethereum_address_from_id(id: u8) -> String {
        let mut buffer = vec![id];
        sha256(buffer.as_mut())
            .into_iter()
            .take(20)
            .collect::<Vec<_>>()
            .to_hex()
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

        let mut contract =
            NearBridge::new(prover_near_account(), e_near_eth_address(), owner_account());

        // lets deposit 1 Near
        let deposit_amount = 1_000_000_000_000_000_000_000_000u128;
        set_env!(
            predecessor_account_id: alice_near_account(),
            attached_deposit: deposit_amount,
        );

        contract.migrate_to_ethereum(alice_eth_address());
    }

    #[test]
    fn test_set_transfer_fee_percentage() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract =
            NearBridge::new(prover_near_account(), e_near_eth_address(), owner_account());

        contract.set_transfer_fee_percentage(100_000u128, 200_000u128);
        let expected_transfer_fee_percentage = contract.get_transfer_fee_percentage();
        assert_eq!(
            expected_transfer_fee_percentage.near_to_eth, 100_000u128,
            "fee percentage for near to eth transfer didn't matched"
        );
        assert_eq!(
            expected_transfer_fee_percentage.eth_to_near, 200_000u128,
            "fee percentage for eth to near transfer didn't matched"
        );
    }

    #[test]
    #[should_panic]
    fn test_set_transfer_fee_percentage_should_panic_if_setter_is_other_than_owner() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract =
            NearBridge::new(prover_near_account(), e_near_eth_address(), owner_account());

        // here caller is not owner or contract
        set_env!(predecessor_account_id: user_near_account());
        contract.set_transfer_fee_percentage(100_000u128, 200_000u128);
    }

    #[test]
    fn test_set_deposit_fee_bounds() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract =
            NearBridge::new(prover_near_account(), e_near_eth_address(), owner_account());

        set_env!(predecessor_account_id: owner_account());

        contract.set_deposit_fee_bounds(100u128, 200u128);
        let expected_deposit_fee_bounds = contract.get_deposit_fee_bounds();
        assert_eq!(
            expected_deposit_fee_bounds.lower_bound, 100u128,
            "lower fee bound didn't matched"
        );
        assert_eq!(
            expected_deposit_fee_bounds.upper_bound, 200u128,
            "upper fee bound didn't matched"
        );
    }

    #[test]
    fn test_set_withdraw_fee_bounds() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract =
            NearBridge::new(prover_near_account(), e_near_eth_address(), owner_account());

        set_env!(predecessor_account_id: owner_account());

        contract.set_withdraw_fee_bounds(100u128, 200u128);
        let expected_withdraw_fee_bounds = contract.get_withdraw_fee_bounds();
        assert_eq!(
            expected_withdraw_fee_bounds.lower_bound, 100u128,
            "lower fee bound didn't matched"
        );
        assert_eq!(
            expected_withdraw_fee_bounds.upper_bound, 200u128,
            "upper fee bound didn't matched"
        );
    }

    #[test]
    fn test_migrate_near_to_eth_with_fee() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract =
            NearBridge::new(prover_near_account(), e_near_eth_address(), owner_account());
        // set fee percentage
        contract.set_transfer_fee_percentage(100_000u128, 200_000u128);
        //set fee bounds
        contract.set_deposit_fee_bounds(100u128, 200u128);

        // lets deposit 1 Near
        let deposit_amount = 1_000_000_000_000_000_000_000_000u128;
        set_env!(
            predecessor_account_id: user_near_account(),
            attached_deposit: deposit_amount,
        );
        let amount_after_fee_deduction = 1_000_000_000_000_000_000_000_000u128 - 200;
        let actual_result = contract.migrate_to_ethereum(alice_eth_address());
        let expected_result = ResultType::MigrateNearToEthereum {
            amount: amount_after_fee_deduction,
            recipient: get_eth_address(alice_eth_address()),
        };
        assert_eq!(
            actual_result, expected_result,
            "result not matched as expected"
        );
    }

    #[test]
    fn test_fee_claim_after_migration_near_to_eth() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract =
            NearBridge::new(prover_near_account(), e_near_eth_address(), owner_account());
        println!("BALANCE-1: {}", env::account_balance());
        // set fee percentage
        contract.set_transfer_fee_percentage(100_000u128, 200_000u128);
        //set fee bounds
        contract.set_deposit_fee_bounds(100u128, 200u128);

        // lets deposit 1 Near
        let deposit_amount = 1_000_000_000_000_000_000_000_000u128;
        set_env!(
            predecessor_account_id: user_near_account(),
            attached_deposit: deposit_amount,
        );
        let amount_after_fee_deduction = 1_000_000_000_000_000_000_000_000u128 - 200;
        let actual_result = contract.migrate_to_ethereum(alice_eth_address());
        let expected_result = ResultType::MigrateNearToEthereum {
            amount: amount_after_fee_deduction,
            recipient: get_eth_address(alice_eth_address()),
        };
        assert_eq!(
            actual_result, expected_result,
            "result not matched as expected"
        );

        assert_eq!(
            contract.cumulative_fee_amount, 200u128,
            "cumulative fee amount doesn't matched before claim"
        );
        set_env!(predecessor_account_id: owner_account());
        contract.claim_fees(50);
        assert_eq!(
            contract.cumulative_fee_amount, 150,
            "cumulative fee amount doesn't matched after claim"
        );
    }

    #[test]
    #[should_panic]
    fn migrate_near_to_eth_panics_when_attached_deposit_is_zero() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract =
            NearBridge::new(prover_near_account(), e_near_eth_address(), owner_account());

        contract.migrate_to_ethereum(alice_eth_address());
    }

    #[test]
    #[should_panic]
    fn migrate_near_to_eth_panics_when_contract_is_paused() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract =
            NearBridge::new(prover_near_account(), e_near_eth_address(), owner_account());

        contract.set_paused(PAUSE_MIGRATE_TO_ETH);

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
    fn migrate_near_to_eth_panics_when_eth_address_is_invalid() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract =
            NearBridge::new(prover_near_account(), e_near_eth_address(), owner_account());

        contract.migrate_to_ethereum(invalid_eth_address());
    }

    #[test]
    #[should_panic]
    fn finalise_eth_to_near_transfer_panics_when_contract_is_paused() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract =
            NearBridge::new(prover_near_account(), e_near_eth_address(), owner_account());

        contract.set_paused(PAUSE_ETH_TO_NEAR_TRANSFER);

        contract.finalise_eth_to_near_transfer(sample_proof());
    }

    #[test]
    #[should_panic]
    fn finalise_eth_to_near_transfer_panics_when_event_originates_from_wrong_contract() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract =
            NearBridge::new(prover_near_account(), e_near_eth_address(), owner_account());

        contract.finalise_eth_to_near_transfer(create_proof(alice_eth_address()));
    }

    #[test]
    #[should_panic]
    fn finish_eth_to_near_transfer_panics_if_attached_deposit_is_not_sufficient_to_record_proof() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract =
            NearBridge::new(prover_near_account(), e_near_eth_address(), owner_account());
        let mock_proof = create_proof(e_near_eth_address());
        let required_deposit = 1720000000000000000000u128;

        // attached deposit is 1000 less than to required deposit for storing proof {to test panic}
        let deposit_amount = required_deposit - 1000u128;
        set_env!(
            predecessor_account_id: alice_near_account(),
            attached_deposit: 0,
        );

        contract.finish_eth_to_near_transfer(true, user_near_account(), deposit_amount, mock_proof);
    }

    #[test]
    fn finalise_eth_to_near_transfer_works_with_valid_params() {
        set_env!(predecessor_account_id: alice_near_account());

        let mut contract =
            NearBridge::new(prover_near_account(), e_near_eth_address(), owner_account());

        // Alice deposit 1 Near to migrate to eth
        let deposit_amount = 1_000_000_000_000_000_000_000_000u128;
        set_env!(
            predecessor_account_id: alice_near_account(),
            attached_deposit: deposit_amount,
        );

        contract.migrate_to_ethereum(alice_eth_address());

        // todo adjust attached deposit down

        // Lets suppose Alice migrates back
        contract.finalise_eth_to_near_transfer(create_proof(e_near_eth_address()))

        // todo asserts i.e. that alice has received the 1 near back etc.
    }
}
