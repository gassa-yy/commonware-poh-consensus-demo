# Limitations

This simulator is intentionally simplified and does **not** model:

- cryptographic signatures, certificates, or equivocation proofs
- real asynchronous networking behavior beyond probabilistic latency/drop
- validator-specific network partitions and adaptive adversaries
- transaction execution/state machine conflicts
- view-sync and pacemaker protocols found in production BFT
- persistent storage, crash recovery, or reconfiguration

## Modeling assumptions

- Probabilities drive vote acceptance and message delivery.
- Timeout handling is round-based and coarse-grained.
- Byzantine behavior is represented as invalid proposals or equivocation flags.
- PoH is a deterministic hash chain, not a production verifiable delay function.

Use this as a teaching scaffold and integration sketch, not as a safety proof or benchmark.
