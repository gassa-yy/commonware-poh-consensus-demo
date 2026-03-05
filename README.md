# commonware-poh-consensus-demo

A **teaching demo** for a DEX-oriented consensus design:

- PoH-like sequencing (continuous hash chain + checkpoints)
- Weighted round-robin proposer selection (by voting power)
- 3-phase BFT flow (Pre-Prepare / Prepare / Commit)
- Mapping to Commonware components (`consensus::simplex`, `p2p::authenticated`, `runtime`, `cryptography`)

> This is not production consensus code. It is an educational, executable model.

## Why this demo

You asked for a practical way to understand how to combine:

1. PoH for verifiable ordering
2. proposer rotation by voting power
3. BFT finalization and fork-choice

This repo gives you runnable examples and architecture notes.

## Quick start

### 1) Run the simulator (Python)

```bash
python3 demos/simulator.py
```

What it prints:

- proposer per round
- PoH head/checkpoint
- phase transitions
- whether a block got finalized or moved to next round

### 2) View architecture diagrams

- `docs/architecture.md`
- `docs/bft_3phase.mmd`
- `docs/poh_rr_bft.mmd`

### 3) Read Commonware integration plan

- `docs/commonware_mapping.md`

## Repository structure

```text
.
├── demos/
│   └── simulator.py
├── docs/
│   ├── architecture.md
│   ├── bft_3phase.mmd
│   ├── poh_rr_bft.mmd
│   └── commonware_mapping.md
└── README.md
```

## Core concepts in this demo

### PoH sequencing

PoH here is modeled as a deterministic hash chain:

- each event advances chain state
- checkpoints are emitted every N ticks
- proposals include PoH checkpoints as ordering evidence

### Weighted round-robin proposer

Each validator has voting power `w_i` and dynamic `priority_i`:

- every round: `priority_i += w_i`
- proposer: validator with max priority
- selected proposer: `priority_selected -= sum(weights)`

This gives long-run proposer frequency proportional to voting power.

### 3-phase BFT

For each `(height, round)`:

1. Pre-Prepare: proposer broadcasts block
2. Prepare: validators verify and vote prepare
3. Commit: after enough prepare votes, validators vote commit

Block finalizes when commit quorum is reached.

## Commonware mapping (practical)

- `commonware-consensus::simplex` → BFT finalization engine
- `commonware-p2p::authenticated` → authenticated networking for validators/mempool sealer
- `commonware-runtime` → production runtime + deterministic simulation runtime
- `commonware-cryptography` → signatures/threshold certs/VRF as needed

PoH engine remains custom and can be plugged into proposal validation rules.

---

If you want, next step can be a Rust prototype that actually wires `simplex` with a PoH proposal verifier.
