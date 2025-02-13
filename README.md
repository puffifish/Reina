# Reina

Reina is a next-generation Layer 1 blockchain that combines:

- PoCUP (Proof-of-Commitment & Useful Participation): A consensus model merging token staking with small, verifiable tasks.
- ROC (Reina On-Chain AI): Focused on streamlined, deterministic AI checks in early phases, with plans for advanced inference later.
- RSL (Reina Smart Language): A Rust-inspired contract language aimed at high concurrency and user-friendliness.

Our core objective is to achieve robust performance on **minimal hardware**, allowing both validators and light nodes to operate seamlessly on low-end or free-tier configurations. By starting with modest HPC tasks and simple AI checks, we lay a foundation for future expansions that can handle more complex computations without sacrificing decentralization.

## Current Status (Phase 1)

- PoCUP: Implements a basic staking record and a placeholder task (a trivial puzzle).
- ROC: Includes a minimal spam detection module (ROC–Sentinel) for transaction filtering.
- RSL: Provides a prototype grammar and parser; no execution or bytecode generation yet.
- Custom Serialization: Leverages zero-copy, compact data handling for high throughput.

All features in Phase 1 are deliberately scoped to remain accessible and easy to integrate, setting the stage for increasingly advanced HPC tasks, AI modules, and a robust smart contract environment in upcoming phases.

## Social & Community

- **Discord**  
  discord.com/invite/vTcsnbjFTf  
  Join our server for real-time discussions, dev logs, and official announcements.

- **Reddit**  
  [r/ReinaProject](https://www.reddit.com/r/ReinaProject/)  
  Find pinned updates, testnet news, and community Q&A.

- **Twitter**  
  [@puffifish_dev](https://twitter.com/puffifish_dev)  
  Follow for short daily progress notes and upcoming milestones.

## Documentation

This repository contains a `docs/` folder that outlines each subsystem in Phase 1:

- `POCUP_SPEC.md`: Basic stake logic and trivial puzzle approach.  
- `ROC_SPEC.md`: Early on-chain AI framework, focusing on spam checks.  
- `RSL_SPEC.md`: Grammar and parser definition for the Rust-inspired language.

We encourage you to explore the docs and follow our development progress on social channels.

## Roadmap & Future Plans

- Phase 2: Potential aggregator-based HPC tasks, partial RSL execution, and expanded AI checks.  
- Phase 3: Deeper concurrency, advanced HPC, and possibly quantum-safe options if performance remains stable.  
- Long-Term: A polished mainnet, bridging to other ecosystems, and real on-chain federated learning or advanced AI functionalities.

## Contributing

We welcome feedback, proposals, and pull requests. Phase 1 aims to keep the codebase modular and straightforward, making it easier for new contributors to review or extend specific components. Please see the issues tab or join our Discord for current discussion topics.

## License

Reina is released under the [Apache 2.0 License](LICENSE). We appreciate all contributions to make the project more secure, efficient, and accessible to a broad range of hardware configurations.
