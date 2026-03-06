# commonware-poh-consensus-demo

Rust-first educational simulator for:

- PoH-like sequencing
- weighted round-robin proposer selection
- 3-phase BFT (Pre-Prepare, Prepare, Commit)
- scenario-driven simulation with metrics and invariant checks
- mapping to Commonware integration points

The original Python demo is still available at `demos/simulator.py`.

## Quick start

### Rust simulator

```bash
cargo test
cargo run -- --scenario normal
cargo run -- --scenario byzantine --seed 7 --heights 20 --max-rounds 10
```

Available scenarios:

- `normal`
- `high-latency`
- `packet-drops`
- `byzantine`
- `timeout`

Optional exports:

```bash
cargo run -- --scenario packet-drops --json-out out/packet_drops.json --csv-out out/packet_drops.csv
```

### Python demo (kept for comparison)

```bash
python3 demos/simulator.py
```

## Architecture

```text
             +-------------------------+
             |  Scenario Runner (CLI)  |
             +-----------+-------------+
                         |
                         v
+------------------+  +-----------------+  +------------------+
|  PoH Hash Chain  |->| Weighted RR      |->| 3-Phase BFT       |
|  (core/poh.rs)   |  | (core/rr.rs)     |  | (core/bft.rs)     |
+------------------+  +-----------------+  +----+--------------+
                                                  |
                                          +-------v--------+
                                          | Network Model   |
                                          | (latency/drop)  |
                                          +-------+--------+
                                                  |
                                          +-------v--------+
                                          | Invariants +    |
                                          | Metrics Output  |
                                          +----------------+
```

## Metrics emitted per run

- finalized block count
- average rounds per finalized block
- failed rounds
- proposer distribution vs voting power
- p50/p95 finalization latency in simulated ticks

## Example output

```text
scenario: normal
finalized_blocks: 30
avg_rounds_per_finalized_block: 1.000
failed_rounds: 0
p50_finalize_ticks: 6
p95_finalize_ticks: 8
proposer_distribution:
  A power=5 expected=0.500 actual=0.500 picks=15
  B power=3 expected=0.300 actual=0.300 picks=9
  C power=2 expected=0.200 actual=0.200 picks=6
```

## Repo layout

```text
.
├── Cargo.toml
├── src/
│   ├── main.rs
│   └── core/
│       ├── bft.rs
│       ├── invariants.rs
│       ├── network.rs
│       ├── poh.rs
│       ├── rr.rs
│       ├── simulator.rs
│       └── types.rs
├── tests/
│   └── sim_invariants.rs
├── demos/
│   └── simulator.py
└── docs/
    ├── architecture.md
    ├── commonware_mapping.md
    ├── INVARIANTS.md
    └── LIMITATIONS.md
```

## Notes

This is an educational simulator, not production consensus code. Simplifications are called out in `docs/LIMITATIONS.md`.
