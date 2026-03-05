# Commonware Mapping for This Design

## Directly useful components

1. `commonware-consensus::simplex`
   - BFT finalization core
   - fast block/finality path (network-hop oriented)
   - consensus over opaque hashes (good for custom execution)

2. `commonware-p2p::authenticated`
   - authenticated encrypted peer communication
   - suitable for validator + mempool-sealer networking

3. `commonware-runtime`
   - `tokio` runtime for production
   - `deterministic` runtime for reproducible consensus/network testing

4. `commonware-cryptography`
   - signatures / threshold primitives / VRF-related building blocks

5. `commonware-consensus::aggregation`
   - useful if you need async certificate recovery over external sequencer outputs

## What remains custom

- PoH engine itself (continuous hash chain + checkpoint format)
- weighted round-robin proposer selector by voting power
- proposal validation rule that checks PoH ordering constraints
- DEX-specific execution state machine (orderbook, risk, liquidation)
