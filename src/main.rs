// File: src/node/main.rs
//! Minimal Node for Reina: a single–node MVP without HTTP.
//! Demonstrates PoCUP tasks via the chain manager.

use reina::node::chain_manager::ChainManager;
use reina::pocup::pocup::Validator;
use reina::utils::serialization::Transaction; // For completeness (future expansion)

fn main() {
    // Create a new chain manager.
    let mut cm = ChainManager::new();
    // Add a few validators with different stakes.
    cm.add_validator("Validator_A".to_string(), 1000);
    cm.add_validator("Validator_B".to_string(), 2000);
    cm.add_validator("Validator_C".to_string(), 1500);

    println!("Running PoCUP tasks on validators...");

    // Run PoCUP tasks: each validator performs its work and is slashed if needed.
    cm.run_pocup_tasks();

    // Print a summary of each validator.
    for v in &cm.validators {
        println!(
            "Validator: {}, Stake: {}, Puzzle passed: {}",
            v.id, v.stake_amount, v.puzzle_passed
        );
    }

    // Here we could serialize the validators to a file for persistence.
    // For now, this is left as a future enhancement.
}