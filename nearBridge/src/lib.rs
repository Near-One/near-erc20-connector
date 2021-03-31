/**
* Bridge for Near Native token
*/
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedSet};
use near_sdk::{
    env, near_bindgen, AccountId, Balance, Promise, ext_contract, Gas
};

use admin_controlled::{AdminControlled, Mask};

#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc<'_> = near_sdk::wee_alloc::WeeAlloc::INIT;

/// Price per 1 byte of storage from mainnet genesis config.
const STORAGE_PRICE_PER_BYTE: Balance = 100_000_000_000_000_000_000;

pub use transfer_to_near_event::TransferToNearInitiatedEvent;
use prover::*;
pub use prover::{is_valid_eth_address, get_eth_address, Proof};

mod transfer_to_near_event;
pub mod prover;

/// Gas to call finalise method.
const FINISH_FINALISE_GAS: Gas = 50_000_000_000_000;

const NO_DEPOSIT: Balance = 0;

/// Gas to call verify_log_entry on prover.
const VERIFY_LOG_ENTRY_GAS: Gas = 50_000_000_000_000;

const PAUSE_MIGRATE_TO_ETH: Mask = 1 << 0;
const PAUSE_ETH_TO_NEAR_TRANSFER: Mask = 1 << 1;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct NearBridge {
    /// The account of the prover that we can use to prove
    pub prover_account: AccountId,

    /// Address of the associated Ethereum eNear ERC20 contract.
    pub e_near_address: EthAddress,

    /// Hashes of the events that were already used.
    pub used_events: UnorderedSet<Vec<u8>>,

    /// Mask determining all paused functions
    paused: Mask,
}

impl Default for NearBridge {
    fn default() -> Self {
        env::panic(b"Contract should be initialized before usage.")
    }
}

#[near_bindgen]
impl NearBridge {
    #[init]
    pub fn new(prover_account: AccountId, e_near_address: String) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            prover_account,
            e_near_address: get_eth_address(e_near_address),
            used_events: UnorderedSet::new(b"u".to_vec()),
            paused: Mask::default(),
        }
    }

    /// Deposit NEAR for bridging from the predecessor account ID
    /// Requirements:
    /// * `eth_recipient` must be a valid eth account
    /// * `amount` must be a positive integer
    /// * Caller of the method has to attach deposit enough to cover:
    ///   * The `amount` of Near tokens being bridged, and
    ///   * The storage difference at the fixed storage price defined in the contract.
    #[payable]
    // todo: how much GAS is required to execute this method with sending the tokens back and ensure we have enough
    pub fn migrate_to_ethereum(&mut self, eth_recipient: String) {
        // Predecessor must attach Near to migrate to ETH
        let attached_deposit = env::attached_deposit();
        if attached_deposit == 0 {
            env::panic(b"Attached deposit must be greater than zero");
        }

        // If the method is paused or the eth recipient address is invalid, then we need to:
        //  1) Return the attached deposit
        //  2) Panic and tell the user why
        if self.is_paused(PAUSE_MIGRATE_TO_ETH) || is_valid_eth_address(eth_recipient) == false {
            Promise::new(env::predecessor_account_id()).transfer(attached_deposit);
            env::panic(b"Method is either paused or ETH address is invalid");
        }

        env::log(format!("{} Near tokens locked", attached_deposit).as_bytes());
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
        assert_self();
        assert!(verification_success, "Failed to verify the proof");

        let required_deposit = self.record_proof(&proof);

        assert!(
            env::attached_deposit() >= required_deposit
        );

        Promise::new(new_owner_id).transfer(amount);
    }

    /// Record proof to make sure it is not re-used later for anther deposit.
    fn record_proof(&mut self, proof: &Proof) -> Balance {
        // TODO: Instead of sending the full proof (clone only relevant parts of the Proof)
        //       log_index / receipt_index / header_data
        assert_self();
        let initial_storage = env::storage_usage();
        let mut data = proof.log_index.try_to_vec().unwrap();
        data.extend(proof.receipt_index.try_to_vec().unwrap());
        data.extend(proof.header_data.clone());
        let key = env::sha256(&data);
        assert!(
            !self.used_events.contains(&key),
            "Event cannot be reused for depositing."
        );
        self.used_events.insert(&key);
        let current_storage = env::storage_usage();

        let required_deposit =
            Balance::from(current_storage - initial_storage) * STORAGE_PRICE_PER_BYTE;
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

pub fn assert_self() {
    assert_eq!(env::predecessor_account_id(), env::current_account_id());
}

admin_controlled::impl_admin_controlled!(NearBridge, paused);
