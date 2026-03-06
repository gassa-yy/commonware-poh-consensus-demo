# commonware-poh-consensus-demo

A **Commonware-first** demo showing how to run
**PoH checkpoint consensus (without mempool)**.

## What is completed

- Real Commonware deps in Cargo (`2026.2.0`):
  - `commonware-consensus`
  - `commonware-p2p`
  - `commonware-runtime`
  - `commonware-cryptography`
- Strict PoH checkpoint payload codec:
  - `src/commonware/checkpoint.rs`
- End-to-end checkpoint-only finalization demo (signature + quorum):
  - `src/commonware/demo.rs`
- Runnable example using actual Commonware interfaces (`Automaton`, `CertifiableAutomaton`, `Relay`, `Reporter`):
  - `examples/commonware_poh_checkpoint.rs`

## Run

```bash
cargo check
cargo test
cargo run --example commonware_poh_checkpoint
```

Expected output includes:
- Commonware runtime module visibility
- trait-boundary verify/certify logs
- finalized heights summary (`finalized_heights=5/5`)
- per-height digest and voted power

## Project layout

```text
src/
  commonware/
    checkpoint.rs      # payload encoding/validation boundary
    demo.rs            # checkpoint-only finalization flow
  core/                # legacy simulator modules (kept for reference)
examples/
  commonware_poh_checkpoint.rs
tests/
  poh_checkpoint_payload.rs
  sim_invariants.rs
```

## Notes

- This demo intentionally uses **no mempool** and finalizes **PoH checkpoints**.
- `src/core/*` and `src/main.rs` are legacy simulation paths retained as reference only.
