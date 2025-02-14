//! Minimal Mempool for Reina Phase 1.
//!
//! This module stores unconfirmed transactions in a simple FIFO Vec.
//! In future phases, we may switch to a priority queue (e.g., BinaryHeap or BTreeMap)
//! and add concurrency via Mutex/RwLock. For now, transactions are validated
//! with a basic fee check and stored in memory.

use crate::utils::serialization::Transaction;

/// A minimal mempool to hold unconfirmed transactions.
pub struct Mempool {
    transactions: Vec<Transaction>,
}

impl Mempool {
    /// Creates a new, empty mempool.
    pub fn new() -> Self {
        Self { transactions: Vec::new() }
    }

    /// Validates a transaction.
    /// Currently, a transaction is valid if its fee is at least 1.0.
    /// Future enhancements will integrate advanced spam detection.
    pub fn validate_transaction(&self, tx: &Transaction) -> bool {
        tx.fee >= 1.0
    }

    /// Adds a transaction to the mempool.
    /// Returns true if the transaction is valid and inserted.
    pub fn add_transaction(&mut self, tx: Transaction) -> bool {
        if self.validate_transaction(&tx) {
            self.transactions.push(tx);
            true
        } else {
            false
        }
    }

    /// Removes and returns the earliest transaction (FIFO) from the mempool.
    pub fn remove_transaction(&mut self) -> Option<Transaction> {
        if !self.transactions.is_empty() {
            Some(self.transactions.remove(0))
        } else {
            None
        }
    }

    /// Returns the current number of transactions in the mempool.
    pub fn size(&self) -> usize {
        self.transactions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_adding_transactions() {
        let mut mempool = Mempool::new();
        assert_eq!(mempool.size(), 0);
        let tx1 = dummy_tx(1, 5.0);
        let tx2 = dummy_tx(2, 10.0);
        assert!(mempool.add_transaction(tx1));
        assert!(mempool.add_transaction(tx2));
        assert_eq!(mempool.size(), 2);
    }

    #[test]
    fn test_removing_transactions() {
        let mut mempool = Mempool::new();
        mempool.add_transaction(dummy_tx(1, 5.0));
        mempool.add_transaction(dummy_tx(2, 10.0));
        let removed = mempool.remove_transaction();
        assert!(removed.is_some());
        assert_eq!(mempool.size(), 1);
    }

    #[test]
    fn test_validation_rejects_low_fee() {
        let mut mempool = Mempool::new();
        let tx = dummy_tx(1, 0.5); // fee too low
        assert!(!mempool.add_transaction(tx));
        assert_eq!(mempool.size(), 0);
    }
}