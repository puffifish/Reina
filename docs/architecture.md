# Reina Architecture (Phase 1)

This document is a preliminary description of Reina's design (Phase 1). Further details will be added as work advances.

## 1. Overview

Reina is all about
- PoCUP (Proof of Commitment & Useful Participation), a staking system combined with basic HPC (useful work) workloads.
- ROC (Reina On-Chain AI) for low AI checks and spam filtering
- RSL (Reina Smart Language), a Rust-inspired contract language.
- Custom serialisation for high throughput (100K+ TPS).

## 2. Phase 1 Objectives

1. PoCUP (Minimal):
- Trivial HPC or useful work placeholder
- Basic stake reasoning, stubbed or minimalist slashing.

2. ROC–Sentinel (Spam Filtering):
- Simple rule-based filtering of transactions

3. RSL v0.1
- Defining a minimal grammar and parser (without yet running it).

4. Low Hardware Requirements
- Light nodes accommodate approximately 1 vCPU, 512 MB to 1 GB of RAM.
- Heavy nodes (validators) on around 2 vCPUs, 2–4 GB of RAM, processing small HPC workloads in Phase 1.

## 3. Top-Level Modules

- PoCUP module (src/pocup): Used for staking, puzzle verifications, and possible expansions of HPC work.

- ROC Module (src/roc): Supports on-blockchain AI (currently spam detection), to be succeeded in the future by HPC verification and governance AI.

- RSL Module (src/rsl): The parser of the custom language and future execution semantics. - Serialization (src/serialization): Already present, using low-overhead and zero-copy mechanisms.

## 4. Future Phases
- Phase 2 can also supply aggregator nodes or advanced AI workloads, aside from partial execution of RSL.
- Phase 3 would scale up HPC workloads significantly, combined with concurrency in RSL and advanced ROC modules.
- The roadmap includes evolving to full mainnet with set tokenomics, aggregator bridging, or even quantum-resistance in case it's a performance enabler.