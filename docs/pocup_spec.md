# PoCUP Specification (Phase 1)

Reina uses a concept called Proof-of-Commitment & Useful Participation (PoCUP) to blend traditional staking with straightforward tasks that benefit the network. In the early stages (Phase 1), the system focuses on simplicity, ensuring that anyone can participate while laying the groundwork for more advanced tasks in the future.

## Overview

PoCUP is designed around two core ideas:

1. Stake: Validators lock up tokens, demonstrating a financial commitment to behave honestly.
2. Useful Work: Validators also perform small tasks—or “puzzles”—that prove they are actively contributing beyond mere financial stake.

## Phase 1 Goals

- Basic Staking: 
  Each validator keeps a record of how many tokens they have locked. Phase 1 won’t enforce formal locking or slashing yet, but the stake amount is tracked.
  
- Trivial Task: 
  Instead of a complex puzzle, we use a simple placeholder that always succeeds. This minimal approach tests how the system might handle more demanding tasks later.

- Slashing Stub: 
  If a validator fails this placeholder task (in theory), the system only logs a warning. No real penalties are applied in Phase 1, though the structure is ready to evolve.

## Staking and Task Flow

1. Add Stake:  
   Validators register their stake amount. In a future phase, these tokens might be locked up more formally.

2. Perform a Puzzle (Placeholder):
   A small routine—currently trivial—runs to confirm each validator’s willingness to do some extra work. It always passes in Phase 1, but it proves the system’s ability to request tasks and record results.

3. Log Failures: 
   If for some reason the placeholder fails, the system simply notes it. Future versions can implement real slashing or stake penalties.

## Future Plans

- Meaningful Tasks:  
  Later, these placeholders will be replaced by actual HPC tasks that are verifiable and beneficial to the network (e.g., verifying advanced cryptographic proofs or small calculations for on-chain AI).

- Reputation and Slashing:  
  Validators who repeatedly fail tasks may lose a portion of their stake. Conversely, those who excel might earn reputation-based rewards.

- Optional Quantum Resistance:  
  In advanced phases, PoCUP might incorporate post-quantum methods for signing or verifying puzzles without significantly increasing overhead.

This design ensures that validators are financially invested and also show a willingness to perform real work, keeping the network more secure and engaged.