# Architecture Overview

## Goal

Show a minimal path to combine:

- PoH (verifiable order evidence)
- voting-power proposer rotation
- BFT finalization

## Dataflow

1. Transactions/orders enter intake.
2. PoH chain advances continuously and emits checkpoints.
3. Round-robin selects proposer by voting power.
4. Proposer creates proposal containing PoH checkpoint.
5. Validators run 3-phase BFT.
6. On commit quorum, block is finalized.

## Notes

- PoH is sequencing evidence, not finality.
- BFT provides finality and fork choice.
- Round-robin controls proposer fairness and liveness under timeout.
