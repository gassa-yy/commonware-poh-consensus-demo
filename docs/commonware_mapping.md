# Commonware Integration Mapping

This repo keeps consensus logic educational while making Rust components map cleanly to Commonware crates.

## Current simulator module -> Commonware target

- `core/bft.rs` -> `commonware-consensus::simplex`
- `core/network.rs` -> `commonware-p2p::authenticated` + runtime transport tasks
- `core/simulator.rs` deterministic execution loop -> `commonware-runtime::deterministic`
- production async orchestration path -> `commonware-runtime` (Tokio runtime)
- proposal integrity and signatures (future) -> `commonware-cryptography`

## What remains custom

- PoH hash-chain/sequencer (`core/poh.rs`)
- weighted RR proposer policy (`core/rr.rs`)
- proposal validity rules that bind PoH checkpoints to blocks
- domain execution logic (DEX/orderbook/risk checks)

## Integration sketch

1. Keep PoH output as part of proposal payload.
2. Feed proposal digest into `simplex` for finality voting.
3. Broadcast votes/proposals over authenticated P2P channels.
4. Run deterministic tests with `commonware-runtime::deterministic` using scenario seeds.
5. Promote stable paths to production runtime and real cryptographic verification.

## Practical migration path

- Step 1: replace simulator BFT transitions with `simplex` state transitions.
- Step 2: replace probabilistic network with authenticated message routing.
- Step 3: keep scenario harness for regression testing under deterministic runtime.
- Step 4: add certificate types and signature checks from `commonware-cryptography`.
