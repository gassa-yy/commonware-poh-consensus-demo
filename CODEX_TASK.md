# Codex Task Brief — Upgrade `commonware-poh-consensus-demo`

## Context
This repo is an educational prototype for:
- PoH-like sequencing
- weighted round-robin proposer selection
- 3-phase BFT flow
- mapping to Commonware components

Current code is intentionally simple. Please upgrade it into a stronger engineering demo.

## Goals
1. Keep it educational, but improve correctness and structure.
2. Add reproducible simulation scenarios.
3. Add clearer interfaces so later Rust/Commonware port is straightforward.

## Required Deliverables

### A) Architecture + Code Refactor
- Refactor Python simulator into modules:
  - `core/poh.py`
  - `core/rr.py`
  - `core/bft.py`
  - `core/network.py` (latency/drop simulation)
  - `core/types.py`
  - `main.py`
- Use dataclasses/types consistently.
- Add deterministic seed control via CLI args.

### B) Scenario Runner
Implement at least 5 scenarios:
1. Normal network
2. High latency
3. Packet drops
4. Byzantine proposer behavior (invalid proposal / equivocation simulation)
5. Leader timeout and round advance

For each scenario, output metrics:
- block finalized count
- avg rounds per finalized block
- failed rounds
- proposer distribution vs voting power
- p50/p95 finalize time (simulated ticks)

### C) Better Weighted RR
- Keep weighted round-robin algorithm.
- Add tests proving long-run proposer frequency approximates voting power ratio.

### D) BFT Safety/Liveness Assertions (simulation-level)
- Add invariants:
  - no double-finalization at same height
  - monotonic finalized height
  - round increases on timeout
- Fail fast if violated.

### E) Docs
- Update README with:
  - architecture diagram
  - scenario instructions
  - sample outputs
- Add `docs/INVARIANTS.md`
- Add `docs/LIMITATIONS.md` (what this sim does NOT model)

### F) Testing
- Add tests (`pytest`) for:
  - PoH determinism
  - RR fairness trend
  - BFT phase transitions
  - invariant checks

## Nice-to-Have
- Export run results to JSON/CSV
- Small plot script for proposer distribution and finalization latency

## Constraints
- Keep dependencies minimal.
- Keep code readable and teaching-friendly.
- No fake claims: explicitly mark simplifications.

## Acceptance Criteria
- `pytest` passes
- `python main.py --scenario normal` runs
- `python main.py --scenario byzantine` runs
- README contains runnable examples and expected output

## Commit Style
Use clear commits:
- `refactor(sim): split core modules`
- `feat(sim): add scenario runner`
- `test(rr): add fairness convergence test`
- `docs: update readme and invariants`
