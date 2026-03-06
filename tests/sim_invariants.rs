use commonware_poh_consensus_demo::core::{
    invariants::InvariantTracker,
    types::{RunConfig, ScenarioKind},
};
use commonware_poh_consensus_demo::run_simulation;

#[test]
fn simulation_runs_and_produces_metrics() {
    let cfg = RunConfig {
        heights: 10,
        max_rounds: 6,
        ..RunConfig::default()
    };
    let res = run_simulation(ScenarioKind::Normal, cfg);
    assert!(res.metrics.finalized_blocks > 0);
    assert!(res.metrics.p95_finalize_ticks >= res.metrics.p50_finalize_ticks);
}

#[test]
fn invariant_catches_double_finalization() {
    let mut tracker = InvariantTracker::default();
    tracker
        .note_finalization(1, "block-1")
        .expect("first finalization should pass");
    let err = tracker
        .note_finalization(1, "block-2")
        .expect_err("must reject two different blocks at same height");
    assert!(err.contains("double finalization"));
}

#[test]
fn invariant_requires_round_advance_after_timeout() {
    let tracker = InvariantTracker::default();
    let err = tracker
        .ensure_timeout_advanced_round(3, Some(3))
        .expect_err("round should increase after timeout");
    assert!(err.contains("did not advance"));
}
