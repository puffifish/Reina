/*!
 * PoCUP Phase 1: Minimal Implementation.
 *
 * Validators must stake tokens and complete a trivial HPC puzzle.
 * Future phases will expand HPC tasks and introduce real penalties.
 */

/// A Validator in PoCUP must stake tokens and perform minimal HPC tasks.
#[derive(Debug)]
pub struct Validator {
    /// Unique identifier or name of the validator.
    pub id: String,
    /// Tokens staked by the validator.
    pub stake_amount: u64,
    /// Indicates if the validator passed the HPC puzzle.
    pub puzzle_passed: bool,
}

/// Returns true as a placeholder for a real HPC puzzle.
/// In Phase 1, this trivial puzzle always succeeds.
#[inline(always)]
pub fn trivial_puzzle() -> bool {
    true
}

/// Performs useful work by running the trivial puzzle.
/// In a real scenario, failure (puzzle_passed = false) would indicate a problem.
pub fn perform_useful_work(validator: &mut Validator) {
    validator.puzzle_passed = trivial_puzzle();
}

/// Increases the validator's stake by a specified amount.
/// Phase 1 only tracks stake without enforcing actual token locking.
pub fn stake(validator: &mut Validator, amount: u64) {
    validator.stake_amount += amount;
}

/// Checks if the validator failed the HPC puzzle and prints a warning.
/// No real penalty is enforced yet.
pub fn slash_if_needed(validator: &mut Validator) {
    if !validator.puzzle_passed {
        println!(
            "Warning: Validator {} failed the HPC puzzle. (No penalty enforced yet)",
            validator.id
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trivial_puzzle() {
        assert!(trivial_puzzle());
    }

    #[test]
    fn test_stake_and_work() {
        let mut v = Validator {
            id: "validator1".to_string(),
            stake_amount: 100,
            puzzle_passed: false,
        };
        stake(&mut v, 50);
        assert_eq!(v.stake_amount, 150);
        perform_useful_work(&mut v);
        assert!(v.puzzle_passed);
    }

    #[test]
    fn test_slash_if_needed() {
        let mut v = Validator {
            id: "validator2".to_string(),
            stake_amount: 200,
            puzzle_passed: false,
        };
        // In this test, no penalty is enforced; just ensure the function runs.
        slash_if_needed(&mut v);
    }
}