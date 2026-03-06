# Commonware Interface Guide

This document describes how this repo maps PoH checkpoint consensus into native
Commonware integration points.

## Target crates and version intent

When crates are resolvable from crates.io, use:

- `commonware-consensus = "2026.2.0"`
- `commonware-p2p = "2026.2.0"`
- `commonware-runtime = "2026.2.0"`
- `commonware-cryptography = "2026.2.0"`

As of March 6, 2026 (local run in this workspace), crates.io DNS resolution is
blocked in the execution environment, so these dependencies could not be pulled
or compiled here.

## Checkpoint payload contract

The checkpoint payload used by the new example is implemented in
`src/commonware/checkpoint.rs` and is intentionally strict so it can be passed
to a real consensus transport unchanged:

- Versioned binary format (`CHECKPOINT_PAYLOAD_VERSION = 1`)
- Contains `height`, `round`, PoH `tick`, PoH `head`, and `app_data`
- App payload hard cap: `MAX_CHECKPOINT_APP_BYTES = 1024`
- Deterministic encode/decode with validation and explicit errors

## Interface mapping (no mempool)

For checkpoint-only consensus:

1. PoH emits checkpoint `(tick, head)` from the sequencer.
2. Producer builds `PohCheckpointPayload`.
3. Payload bytes are proposed to Commonware consensus (simplex path).
4. Votes/certificates are transported over Commonware P2P channels.
5. Runtime drives task scheduling and deterministic replay tests.
6. Cryptography verifies signatures over payload digest/certificates.

## Minimal runnable demo

`examples/commonware_poh_checkpoint.rs` demonstrates:

- No mempool transactions
- One proposal per emitted PoH checkpoint
- Weighted quorum-based finalization flow
- Payload encoding/decoding checks on the hot path

This is a compileable adapter-level demo in this sandbox, and is ready to be
wired to real `commonware-*` crate APIs once registry access is available.
