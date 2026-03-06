# Simulation Invariants

The Rust simulator enforces these invariants and fails fast on violation:

1. No double-finalization at the same height
- At a given block height, only one block ID may be finalized.

2. Monotonic finalized height
- Finalized height cannot decrease.

3. Round increases on timeout
- If a round times out, the next attempted round must be strictly greater.

## Where enforced

- `src/core/invariants.rs` stores invariant state and checks.
- `src/core/simulator.rs` calls checks during each run.

## Why these are useful

- They make safety violations explicit during experimentation.
- They catch broken control-flow in round advancement.
- They ensure metrics are derived from consistent state transitions.
