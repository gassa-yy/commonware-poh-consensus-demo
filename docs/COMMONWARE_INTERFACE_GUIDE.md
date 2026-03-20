# COMMONWARE_INTERFACE_GUIDE

## Goal
Use Commonware stack APIs to build a **PoH checkpoint consensus** flow (no mempool), so interface boundaries are clear.

## Crates used
- `commonware-consensus = "2026.2.0"`
- `commonware-p2p = "2026.2.0"`
- `commonware-runtime = "2026.2.0"`
- `commonware-cryptography = "2026.2.0"`

## Where each interface is used

### 1) Consensus trait boundary
In `examples/commonware_poh_checkpoint.rs`:
- `Automaton`
- `CertifiableAutomaton`
- `Relay`
- `Reporter`

This is the exact boundary where your app-specific payload (PoH checkpoint bytes) plugs into consensus logic.

### 2) Cryptography
In `src/commonware/demo.rs`:
- `ed25519::PrivateKey`
- `Signer` / `Verifier`
- `Sha256` (`Hasher`)

This signs and verifies votes over checkpoint digest, then checks quorum.

### 3) Runtime and P2P
In `examples/commonware_poh_checkpoint.rs` and `src/commonware/demo.rs`:
- `commonware_runtime::deterministic::Config` (runtime path visibility)
- `commonware_p2p::Recipients` (recipient model boundary)

## Payload contract
`src/commonware/checkpoint.rs` defines strict, versioned encoding:
- fields: `height`, `round`, `checkpoint_tick`, `checkpoint_head_hex`, `app_data`
- max app bytes: `1024`
- deterministic `encode/decode`
- explicit validation errors

## Why this matters for your next step

When you add mempool later, you only change **payload construction** and application logic:
- checkpoint-only payload -> checkpoint + tx batch root payload
- same consensus boundary (`Automaton/Relay/Reporter`)
- same signature/quorum path

That keeps your architecture stable while evolving functionality.
