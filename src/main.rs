// File: src/node/main.rs
//! Minimal Reina Node MVP (Phase 1)
//!
//! This binary demonstrates a single–node flow by integrating a ChainManager 
//! (holding validators), a Mempool (storing unconfirmed transactions), PoCUP tasks,
//! and a basic RSL contract parser. It now includes a continuous block production loop,
//! simulating ongoing block creation. No HTTP server is included.

use reina::node::chain_manager::ChainManager;
use reina::node::mempool::Mempool;
use reina::consensus::block_producer::Block; // Minimal Block struct
use reina::pocup::pocup::{stake, perform_useful_work, slash_if_needed};
use reina::rsl::parse_rsl;
use reina::utils::serialization::Transaction;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn main() {
    println!("Starting Reina Phase 1 node demo...");

    // Create a ChainManager and add validators.
    let mut chain_manager = ChainManager::new();
    chain_manager.add_validator("Validator_A".to_string(), 100);
    println!("Added Validator_A with stake 100.");
    chain_manager.add_validator("Validator_B".to_string(), 200);
    println!("Added Validator_B with stake 200.");
    chain_manager.add_validator("Validator_C".to_string(), 150);
    println!("Added Validator_C with stake 150.");

    // Run PoCUP tasks on validators.
    println!("Running PoCUP tasks on validators...");
    for validator in &mut chain_manager.validators {
        perform_useful_work(validator);
        slash_if_needed(validator);
        println!("Validator {}: stake = {}, puzzle_passed = {}",
            validator.id, validator.stake_amount, validator.puzzle_passed);
    }

    // Create a Mempool and add some dummy transactions.
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

    // Run one PoCUP round on mempool transactions (for demo, remove one transaction).
    if let Some(tx) = mempool.remove_transaction() {
        println!("Removed transaction {} from mempool.", tx.id);
    }
    println!("Mempool size after removal: {}", mempool.size());

    // Optionally, parse a small RSL contract.
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

    // Continuous Block Production Loop:
    // In Phase 1, we simulate block production by creating a new block in each loop iteration.
    println!("Entering continuous block production loop...");
    let mut block_number = 1u64;
    loop {
        println!("Producing block #{}...", block_number);
        // Pull up to 3 transactions from the mempool (FIFO).
        let mut txs = Vec::new();
        for _ in 0..3 {
            if let Some(tx) = mempool.remove_transaction() {
                txs.push(tx);
            }
        }
        // Get current timestamp.
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time error")
            .as_secs();
        // Construct a new block with default previous hash.
        let block = Block {
            block_number,
            previous_hash: [0u8; 32],
            transactions: txs,
            timestamp,
            signature: Vec::new(), // Placeholder signature.
        };
        println!(
            "Produced block #{} with {} transactions at timestamp {}.",
            block.block_number, block.transactions.len(), block.timestamp
        );
        block_number += 1;
        // Sleep for 5 seconds before producing the next block.
        thread::sleep(Duration::from_secs(5));
    }
}