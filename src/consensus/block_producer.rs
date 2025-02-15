// File: src/consensus/block_producer.rs
//! Minimal Block Producer for Phase 1.
//!
//! This module simulates block production by maintaining its own block counter,
//! pulling transactions from a mempool, and simulating validator work (via PoCUP functions).
//! The produced block includes a sequential block number, a default previous hash,
//! a batch of transactions, and the current timestamp. Future phases will integrate
//! real previous block linking and advanced consensus logic.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::node::chain_manager::ChainManager;
use crate::node::mempool::Mempool;
use crate::pocup::pocup::{perform_useful_work, slash_if_needed};
use crate::utils::serialization::Transaction;

/// A minimal Block structure for Phase 1.
#[derive(Debug, Clone)]
pub struct Block {
    /// Sequential block number.
    pub block_number: u64,
    /// Previous block's hash; Phase 1 uses a default value.
    pub previous_hash: [u8; 32],
    /// List of transactions included in this block.
    pub transactions: Vec<Transaction>,
    /// Block timestamp in seconds since UNIX_EPOCH.
    pub timestamp: u64,
    /// Placeholder signature.
    pub signature: Vec<u8>,
}

/// BlockProducer produces new blocks by pulling transactions from the mempool
/// and simulating validator work. It holds a reference to a ChainManager for access
/// to validators (for PoCUP tasks) and its own block counter.
pub struct BlockProducer<'a> {
    /// Reference to the ChainManager (for validator work).
    pub chain_manager: &'a mut ChainManager,
    /// Internal block counter for sequential block numbering.
    pub block_counter: u64,
}

impl<'a> BlockProducer<'a> {
    /// Creates a new BlockProducer with the given ChainManager.
    /// Initializes the block counter to 1.
    pub fn new(chain_manager: &'a mut ChainManager) -> Self {
        Self {
            chain_manager,
            block_counter: 1,
        }
    }

    /// Produces a new block by:
    /// 1. Using the internal block counter as the new block number.
    /// 2. Setting previous_hash to a default ([0u8;32]) since no prior block is tracked.
    /// 3. Pulling up to two transactions from the mempool.
    /// 4. Running PoCUP tasks on each validator (simulate work and slashing).
    /// 5. Setting the block timestamp to SystemTime::now().
    /// 6. Incrementing the block counter.
    pub fn produce_block(&mut self, mempool: &mut Mempool) -> Block {
        let block_number = self.block_counter;
        let previous_hash = [0u8; 32]; // Phase 1 uses a default previous hash.

        // Pull up to 2 transactions from the mempool (FIFO).
        let mut transactions = Vec::new();
        for _ in 0..2 {
            if let Some(tx) = mempool.remove_transaction() {
                transactions.push(tx);
            }
        }

        // Simulate PoCUP work on validators.
        // For each validator in the chain manager, perform useful work and check for slashing.
        for v in &mut self.chain_manager.validators {
            perform_useful_work(v);
            slash_if_needed(v);
        }

        // Get current timestamp.
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX_EPOCH")
            .as_secs();

        let block = Block {
            block_number,
            previous_hash,
            transactions,
            timestamp,
            signature: Vec::new(), // Placeholder; no real signature yet.
        };

        self.block_counter += 1;
        block
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::chain_manager::ChainManager;
    use crate::node::mempool::Mempool;
    use crate::utils::serialization::Transaction;

    fn dummy_tx(id: u64, fee: f64) -> Transaction {
        Transaction {
            id,
            amount: 1000,
            fee,
            version: 1,
            sender: "Alice".to_string(),
            recipient: "Bob".to_string(),
            signature: vec![1, 2, 3, 4],
        }
    }

    #[test]
    fn test_produce_block() {
        // Create a dummy ChainManager with some validators.
        let mut chain_manager = ChainManager::new();
        chain_manager.add_validator("Validator_A".to_string(), 100);
        chain_manager.add_validator("Validator_B".to_string(), 200);

        // Create a mempool and add a few transactions.
        let mut mempool = Mempool::new();
        for i in 1..=3 {
            mempool.add_transaction(dummy_tx(i, i as f64 * 10.0));
        }

        let mut producer = BlockProducer::new(&mut chain_manager);
        let block = producer.produce_block(&mut mempool);

        // Block number should match initial counter.
        assert_eq!(block.block_number, 1);
        // Previous hash is default.
        assert_eq!(block.previous_hash, [0u8; 32]);
        // Up to 2 transactions are pulled.
        assert!(block.transactions.len() <= 2);
    }
}