/**
* Bridge for Near Native token
*/
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise, StorageUsage};

#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc<'_> = near_sdk::wee_alloc::WeeAlloc::INIT;

/// Price per 1 byte of storage from mainnet genesis config.
const STORAGE_PRICE_PER_BYTE: Balance = 100_000_000_000_000_000_000;

type EthereumAddress = [u8; 20];

use prover::*;
pub use prover::{validate_eth_address, Proof};

mod lock_event;
pub mod prover;
mod unlock_event;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct NearBridge {
    /// The account of the prover that we can use to prove
    pub prover_account: AccountId,

    /// Address of the associated Ethereum eNear ERC20 contract.
    pub e_near_address: EthAddress,

    /// Hashes of the events that were already used.
    pub used_events: UnorderedSet<Vec<u8>>,
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
            e_near_address: validate_eth_address(e_near_address),
            used_events: UnorderedSet::new(b"u".to_vec()),
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
    pub fn migrate_to_ethereum(&mut self, eth_recipient: String) {
        // As attached deposit includes tokens for storage, deposit amount needs to be explicit
        let attached_deposit = env::attached_deposit();
        if attached_deposit == 0 {
            env::panic(b"Attached deposit must be greater than zero");
        }

        // Check receiver on Eth side looks valid i.e. 20 bytes
        validate_eth_address(eth_recipient);

        env::log(format!("{} Near tokens locked", attached_deposit).as_bytes());
    }

    #[payable]
    pub fn finalise_eth_to_near_transfer(&mut self, #[serializer(borsh)] proof: Proof) {
        //self.check_not_paused(PAUSE_DEPOSIT);

    }
}

impl NearBridge {

    fn refund_storage(&self, initial_storage: StorageUsage) {
        let current_storage = env::storage_usage();
        let attached_deposit = env::attached_deposit();
        let refund_amount = if current_storage > initial_storage {
            let required_deposit =
                Balance::from(current_storage - initial_storage) * STORAGE_PRICE_PER_BYTE;
            assert!(
                required_deposit <= attached_deposit,
                "The required attached deposit is {}, but the given attached deposit is is {}",
                required_deposit,
                attached_deposit,
            );
            attached_deposit - required_deposit
        } else {
            attached_deposit
                + Balance::from(initial_storage - current_storage) * STORAGE_PRICE_PER_BYTE
        };
        if refund_amount > 0 {
            env::log(format!("Refunding {} tokens for storage", refund_amount).as_bytes());
            Promise::new(env::predecessor_account_id()).transfer(refund_amount);
        }
    }
}
