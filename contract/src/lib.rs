//! Contract that can be used for different types of loadtesting.
//! Based on the rust-counter example.

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, TreeMap};
use near_sdk::serde::Serialize;
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise};
use near_sdk::{log, PromiseOrValue};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Debug, PartialEq)]
pub struct Winner {
    owner: AccountId,
    amount: Balance,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Debug, PartialEq)]
pub struct SlotInfo {
    metadata: String,
    winner: Option<Winner>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Reservations {
    slots: LookupMap<AccountId, TreeMap<u64, SlotInfo>>,
}

impl Default for Reservations {
    fn default() -> Self {
        Self {
            slots: LookupMap::new(b"r".to_vec()),
        }
    }
}
#[near_bindgen]
impl Reservations {
    pub fn get_slots_info(&self, account_id: AccountId) -> Vec<(u64, SlotInfo)> {
        if let Some(tree_map) = self.slots.get(&account_id) {
            log!("Account {} has {} entries", account_id, tree_map.len());
            tree_map.iter().collect()
        } else {
            vec![]
        }
    }
    pub fn get_detailed_info(&self, account_id: AccountId, slot_time: u64) -> SlotInfo {
        self.slots
            .get(&account_id)
            .unwrap()
            .get(&slot_time)
            .unwrap()
    }

    pub fn add_slot(&mut self, slot_time: u64, metadata: String) {
        let account_id = near_sdk::env::predecessor_account_id();
        let mut tree_map = self.slots.get(&account_id).unwrap_or_else(|| {
            let new_tree = TreeMap::new(format!("p_{}", account_id).try_to_vec().unwrap());
            self.slots.insert(&account_id, &new_tree);
            new_tree
        });

        if !tree_map.contains_key(&slot_time) {
            tree_map.insert(
                &slot_time,
                &SlotInfo {
                    metadata,
                    winner: None,
                },
            );
            log!("Added slot {} to {}", slot_time, account_id);
        } else {
            env::panic_str(format!("slot already present {}", slot_time).as_str());
        }
    }

    pub fn claim_and_remove_slot(&mut self, slot_time: u64) -> PromiseOrValue<u32> {
        let account_id = near_sdk::env::predecessor_account_id();
        if let Some(mut tree_map) = self.slots.get(&account_id) {
            if let Some(slot) = tree_map.get(&slot_time) {
                if let Some(winner) = slot.winner {
                    log!(
                        "claiming slot {} {} amount: {}",
                        account_id,
                        slot_time,
                        winner.amount
                    );
                    tree_map.remove(&slot_time);
                    return PromiseOrValue::Promise(
                        Promise::new(account_id).transfer(winner.amount),
                    );
                } else {
                    log!("claiming empty slot {} {}", account_id, slot_time);
                    tree_map.remove(&slot_time);
                }
            }
        };
        return PromiseOrValue::Value(0);
    }

    #[payable]
    pub fn bet(&mut self, account_id: AccountId, slot_time: u64) -> PromiseOrValue<u32> {
        if let Some(mut tree_map) = self.slots.get(&account_id) {
            if let Some(mut slot) = tree_map.get(&slot_time) {
                if let Some(ref mut winner) = slot.winner {
                    if winner.amount < env::attached_deposit() {
                        // Make sure we have enough gas..

                        log!(
                            "New highest bet for {}{} {}",
                            account_id,
                            slot_time,
                            env::attached_deposit()
                        );

                        // Refund.
                        let refund = Promise::new(winner.owner.clone()).transfer(winner.amount);

                        winner.amount = env::attached_deposit();
                        winner.owner = env::predecessor_account_id();
                        tree_map.insert(&slot_time, &slot);
                        return PromiseOrValue::Promise(refund);
                    } else {
                        env::panic_str(
                            format!(
                                "Attached amount {} is smaller than current winning bid {}",
                                env::attached_deposit(),
                                winner.amount
                            )
                            .as_str(),
                        );
                    }
                } else {
                    slot.winner = Some(Winner {
                        owner: env::predecessor_account_id(),
                        amount: env::attached_deposit(),
                    });
                    tree_map.insert(&slot_time, &slot);
                }
            } else {
                env::panic_str(format!("Didn't find slot {}", slot_time).as_str());
            }
        } else {
            env::panic_str(format!("Didn't find account {}", account_id).as_str());
        }
        return PromiseOrValue::Value(0);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::{testing_env, VMContext};

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id("bob_near".parse().unwrap())
            .is_view(is_view)
            .build()
    }

    #[test]
    fn set_get_message() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Reservations::default();
        contract.add_slot(1, "hello".to_string());
        assert_eq!(get_logs(), vec!["Added slot 1 to bob.near"]);

        let context = get_context(true);
        testing_env!(context);
        let result = vec![(
            1 as u64,
            SlotInfo {
                metadata: "hello".to_string(),
                winner: None,
            },
        )];

        assert_eq!(result, contract.get_slots_info("bob_near".parse().unwrap()));
        assert_eq!(get_logs(), vec!["get_status for account_id bob_near"])
    }
    /*
    #[test]
    fn get_nonexistent_message() {
        let context = get_context(true);
        testing_env!(context);
        let contract = Reservations::default();
        assert_eq!(None, contract.add_slot(2, "francis.near".parse().unwrap()));
        assert_eq!(get_logs(), vec!["get_status for account_id francis.near"])
    }*/
}
