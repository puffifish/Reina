// File: src/node/main.rs
//! Minimal Reina Node MVP (Phase 1)
//!
//! This binary demonstrates a single–node flow by integrating a ChainManager 
//! (holding validators), a Mempool (storing unconfirmed transactions), PoCUP tasks,
//! and a basic RSL contract parser. No HTTP server is included.

use reina::node::chain_manager::ChainManager;
use reina::node::mempool::Mempool;
use reina::pocup::pocup::{stake, perform_useful_work, slash_if_needed};
use reina::rsl::parse_rsl;
use reina::utils::serialization::Transaction;

fn main() {
    println!("Starting Reina Phase 1 node demo...");

    // Create a ChainManager (holding validators) and add some validators.
    let mut chain_manager = ChainManager::new();
    chain_manager.add_validator("Validator_A".to_string(), 100);
    println!("Added Validator_A with stake 100.");
    chain_manager.add_validator("Validator_B".to_string(), 200);
    println!("Added Validator_B with stake 200.");
    chain_manager.add_validator("Validator_C".to_string(), 150);
    println!("Added Validator_C with stake 150.");

    // Demonstrate PoCUP: for each validator, perform work and possibly slash.
    println!("Running PoCUP tasks on validators...");
    for validator in &mut chain_manager.validators {
        // Simulate work: this sets puzzle_passed to true (trivial puzzle).
        perform_useful_work(validator);
        // If the validator had failed the puzzle, we would slash them.
        slash_if_needed(validator);
        println!("Validator {}: stake = {}, puzzle_passed = {}",
            validator.id, validator.stake_amount, validator.puzzle_passed);
    }

    // Create a mempool and add a few dummy transactions.
    let mut mempool = Mempool::new();
    for i in 1..=5 {
        let tx = Transaction {
            id: i,
            amount: 1000,
            fee: (i * 10) as f64,
            version: 1,
            sender: "Alice".to_string(),
            recipient: "Bob".to_string(),
            signature: vec![1, 2, 3, 4],
        };
        if mempool.add_transaction(tx) {
            println!("Inserted transaction {} into mempool.", i);
        } else {
            println!("Failed to insert transaction {}.", i);
        }
    }
    println!("Mempool size: {}", mempool.size());

    // Run a single PoCUP round on mempool transactions (for demo, simply remove one).
    if let Some(tx) = mempool.remove_transaction() {
        println!("Removed transaction {} from mempool.", tx.id);
    }
    println!("Mempool size after removal: {}", mempool.size());

    // Optionally, parse a small RSL contract for demonstration.
    let rsl_source = r#"
        contract Demo {
            let counter: u64;
            fn inc(v: u64) {
                counter = counter + v;
            }
        }
    "#;
    match parse_rsl(rsl_source) {
        Ok(ast) => println!("Parsed RSL contract: {:?}", ast),
        Err(e) => println!("RSL parsing error: {:?}", e),
    }

    // End of demo – in a real node, an infinite loop would continue block production, etc.
    println!("Reina Phase 1 node demo completed.");
}