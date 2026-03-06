# Architecture Overview

## Goal

Demonstrate a Rust-first path to combine:

- PoH-like ordering evidence
- voting-power weighted proposer rotation
- 3-phase BFT finalization
- deterministic scenario simulation and invariant enforcement

## Dataflow

1. Scenario config sets network and behavior parameters.
2. PoH chain advances with synthetic events per round.
3. Weighted round-robin selects proposer.
4. Proposer builds `(height, round)` proposal with PoH checkpoint.
5. BFT runs Pre-Prepare/Prepare/Commit over simulated network.
6. On quorum commit, block finalizes and invariants are checked.
7. Metrics aggregate finalization, failure, fairness, and latency signals.

## Components

- `src/core/poh.rs`: deterministic hash-chain checkpointing
- `src/core/rr.rs`: weighted RR proposer rotation
- `src/core/network.rs`: latency/drop sampling
- `src/core/bft.rs`: 3-phase voting and timeout outcomes
- `src/core/invariants.rs`: safety/liveness guardrails
- `src/core/simulator.rs`: scenario loop + metrics + export
- `src/main.rs`: CLI scenario entrypoint
