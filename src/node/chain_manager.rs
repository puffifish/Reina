//! Minimal ChainManager for PoCUP.
//! Manages a list of validators and runs PoCUP tasks on them.

use crate::pocup::pocup::{Validator, perform_useful_work, slash_if_needed};

/// ChainManager holds a list of PoCUP validators.
pub struct ChainManager {
    /// Validators managed by the node.
    pub validators: Vec<Validator>,
}

impl ChainManager {
    /// Creates a new, empty ChainManager.
    pub fn new() -> Self {
        Self { validators: Vec::new() }
    }

    /// Adds a new validator with the given id and stake.
    /// The validator's `puzzle_passed` is initially false.
    pub fn add_validator(&mut self, id: String, stake_amount: u64) {
        let validator = Validator { id, stake_amount, puzzle_passed: false };
        self.validators.push(validator);
    }

    /// Runs PoCUP tasks on all validators.
    /// For each validator, it calls `perform_useful_work` and then `slash_if_needed`.
    pub fn run_pocup_tasks(&mut self) {
        for v in &mut self.validators {
            perform_useful_work(v);
            slash_if_needed(v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_and_run() {
        let mut cm = ChainManager::new();
        cm.add_validator("validator1".to_string(), 1000);
        assert_eq!(cm.validators.len(), 1);
        // Initially, puzzle_passed is false.
        assert!(!cm.validators[0].puzzle_passed);
        cm.run_pocup_tasks();
        // trivial_puzzle always returns true in Phase 1.
        assert!(cm.validators[0].puzzle_passed);
    }
}