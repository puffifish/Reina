//! Sentinel Module for ROC.
//!
//! Provides minimal spam detection for transactions in Phase 1.
//! Rules: reject if fee < 1.0 or if sender equals recipient.
//! Future versions will implement advanced AI spam detection.

use crate::utils::serialization::Transaction;

/// Returns true if the transaction passes spam checks; false otherwise.
#[inline(always)]
pub fn check_spam(tx: &Transaction) -> bool {
    if tx.fee < 1.0 {
        return false;
    }
    if tx.sender == tx.recipient {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::serialization::Transaction;

    #[test]
    fn test_check_spam() {
        let tx_valid = Transaction {
            id: 1,
            amount: 1000,
            fee: 5.0,
            version: 1,
            sender: "Alice".to_string(),
            recipient: "Bob".to_string(),
            signature: vec![1, 2, 3, 4],
        };
        let tx_low_fee = Transaction { fee: 0.5, ..tx_valid.clone() };
        let tx_same = Transaction { sender: "Alice".to_string(), recipient: "Alice".to_string(), ..tx_valid.clone() };

        assert!(check_spam(&tx_valid));
        assert!(!check_spam(&tx_low_fee));
        assert!(!check_spam(&tx_same));
    }
}