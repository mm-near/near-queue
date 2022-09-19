//! Contract that can be used for different types of loadtesting.
//! Based on the rust-counter example.

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise};

near_sdk::setup_alloc!();

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Winner {
    owner: AccountId,
    amount: Balance,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SlotInfo {
    metadata: String,
    winner: Option<Winner>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Reservations {
    owner: AccountId,
    slots: LookupMap<u64, SlotInfo>,
}

impl Default for Reservations {
    fn default() -> Self {
        Self { slots: LookupMap::new(b"r".to_vec()) }
    }
}
#[near_bindgen]
impl Reservations {
    pub fn add_slot(&mut self, slot_time: u64, metadata: String) {
        if !self.slots.contains_key(&slot_time) {
            self.slots.insert(&slot_time, &SlotInfo { metadata, winner: None });
        }
        // check if owner
    }

    pub fn claim_and_remove_slot(&self, slot_time: u64) {
        if let Some(slot) = self.slots.get(&slot_time) {
            if let Some(winner) = slot.winner {
                Promise::new(self.owner).transfer(winner.amount);
            }
        }
        // and then remove the entry

        // check if owner
    }

    pub fn bet(&self, slot_time: u64) {}
}
