# RSL v0.1 Specification

RSL (Reina Smart Language) is a language inspired by Rust’s syntax and safety principles. Phase 1 focuses on establishing a simple grammar and parser, without execution or bytecode generation. The objective is to lay a foundation that can evolve into a robust, parallel-friendly contract language.

## Language Focus

- Rust-like style: Emphasizes clarity and type safety.
- Minimal overhead: Avoids features that might complicate concurrency or determinism in Phase 1.
- Future concurrency: Designed to add parallel features once the chain’s HPC tasks expand.

## Basic Structure and Syntax

At this stage, RSL supports:

1. Contracts: A named block containing fields (state) and functions.
2. Fields: Declared with a name and type.
3. Functions: Accept parameters, optionally return a value, and manipulate contract state.

Below is a short illustration of how RSL might look in Phase 1. Note that this snippet does not execute any advanced logic; it is purely for understanding the language form:

// Update the puzzle parameter (basic assignment)
fn set_puzzle_param(value: u64) {
    puzzle_param = value;
}

// A demonstration function for HPC operations
fn compute_factor(x: u64): u64 {
    // A hypothetical multiplication that references puzzle_param
    let result = x * puzzle_param;
    return result;
}


## Phase 1 Capabilities

- Parsing Grammar:  
  The compiler can parse fields, functions, and basic types like `u64` or `String`.
- No Execution or Compilation:  
  Code above is only parsed into an internal representation (AST). Future phases will incorporate bytecode generation or real on-chain execution.

## Future Enhancements

- Concurrency:  
  Additional language features for parallel computations or asynchronous tasks.
- AI Integration:  
  Possibility of calling ROC modules (e.g., for HPC checks or AI decisions) directly from within RSL.
- Bytecode Execution:  
  Converting RSL to a low-level format (either a custom VM or WASM) to enable real contract deployment and runtime logic in advanced phases.

In Phase 1, RSL is deliberately limited, ensuring the community can focus on core parsing and grammar design before addressing execution details or concurrency features.
