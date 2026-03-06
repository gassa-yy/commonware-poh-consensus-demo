use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::core::{
    bft::BftEngine,
    invariants::InvariantTracker,
    network::NetworkModel,
    poh::Poh,
    rr::WeightedRoundRobin,
    types::{ByzantineMode, Proposal, RoundStatus, RunConfig, ScenarioConfig, ScenarioKind},
    util::SimpleRng,
};

#[derive(Clone, Debug)]
pub struct ProposerStat {
    pub id: String,
    pub power: u64,
    pub expected_share: f64,
    pub actual_share: f64,
    pub selections: u64,
}

#[derive(Clone, Debug)]
pub struct Metrics {
    pub scenario: String,
    pub finalized_blocks: u64,
    pub avg_rounds_per_finalized_block: f64,
    pub failed_rounds: u64,
    pub p50_finalize_ticks: u64,
    pub p95_finalize_ticks: u64,
    pub proposer_stats: Vec<ProposerStat>,
}

#[derive(Clone, Debug)]
pub struct SimulationResult {
    pub metrics: Metrics,
}

pub fn run_simulation(kind: ScenarioKind, cfg: RunConfig) -> SimulationResult {
    let scenario = ScenarioConfig::for_kind(kind);
    run_with_scenario(&scenario, cfg)
}

fn run_with_scenario(scenario: &ScenarioConfig, cfg: RunConfig) -> SimulationResult {
    let mut app_rng = SimpleRng::new(cfg.seed);
    let mut network = NetworkModel::new(
        cfg.seed.wrapping_add(1),
        scenario.base_latency_ticks,
        scenario.jitter_ticks,
        scenario.drop_rate,
    );

    let mut rr = WeightedRoundRobin::new(cfg.validators.clone());
    let bft = BftEngine::new(cfg.validators.clone());
    let mut poh = Poh::new(cfg.checkpoint_every);
    let mut invariants = InvariantTracker::default();

    let total_power: u64 = cfg.validators.iter().map(|v| v.power).sum();

    let mut failed_rounds = 0u64;
    let mut finalized_blocks = 0u64;
    let mut rounds_per_finalized = Vec::new();
    let mut finalize_ticks = Vec::new();

    for height in 1..=cfg.heights {
        let mut finalized = false;

        for round in 0..cfg.max_rounds {
            let proposer = rr.select();
            let events = app_rng.gen_range_inclusive(1, cfg.max_poh_events_per_round.max(1));
            for event_idx in 0..events {
                let payload = format!("H{height}R{round}E{event_idx}");
                let _ = poh.advance(&payload);
            }
            let checkpoint = poh.checkpoint();
            let _ = poh.should_emit_checkpoint();

            let mut invalid_payload = false;
            let mut equivocation = false;
            if proposer.id == "A" {
                if let Some(mode) = scenario.byzantine_mode {
                    if app_rng.gen_bool(scenario.byzantine_chance) {
                        match mode {
                            ByzantineMode::InvalidProposal => invalid_payload = true,
                            ByzantineMode::Equivocation => equivocation = true,
                        }
                    }
                }
            }

            let proposal = Proposal {
                height,
                round,
                proposer_id: proposer.id,
                checkpoint,
                invalid_payload,
                equivocation,
            };

            let outcome = bft.run_round(
                &proposal,
                &mut network,
                &mut app_rng,
                scenario.prepare_accept_prob,
                scenario.commit_accept_prob,
                scenario.timeout_ticks,
            );

            match outcome.status {
                RoundStatus::Finalized => {
                    finalized_blocks += 1;
                    rounds_per_finalized.push((round + 1) as f64);
                    finalize_ticks.push(outcome.latency_ticks);
                    let block_id = outcome.block_id.unwrap_or_else(|| "unknown".to_string());
                    if let Err(err) = invariants.note_finalization(height, &block_id) {
                        panic!("{err}");
                    }
                    finalized = true;
                    break;
                }
                RoundStatus::Timeout => {
                    failed_rounds += 1;
                    if let Err(err) =
                        invariants.ensure_timeout_advanced_round(round, Some(round + 1))
                    {
                        panic!("{err}");
                    }
                }
                RoundStatus::PrepareFailed
                | RoundStatus::CommitFailed
                | RoundStatus::ByzantineRejected => {
                    failed_rounds += 1;
                }
            }
        }

        if !finalized {
            failed_rounds += 1;
        }
    }

    let avg_rounds_per_finalized_block = if rounds_per_finalized.is_empty() {
        0.0
    } else {
        rounds_per_finalized.iter().sum::<f64>() / rounds_per_finalized.len() as f64
    };

    let proposer_stats = proposer_stats(rr.selections(), &cfg.validators, total_power);
    let metrics = Metrics {
        scenario: scenario.name.to_string(),
        finalized_blocks,
        avg_rounds_per_finalized_block,
        failed_rounds,
        p50_finalize_ticks: percentile(&finalize_ticks, 0.50),
        p95_finalize_ticks: percentile(&finalize_ticks, 0.95),
        proposer_stats,
    };

    SimulationResult { metrics }
}

fn proposer_stats(
    counts: &BTreeMap<String, u64>,
    validators: &[crate::core::types::Validator],
    total_power: u64,
) -> Vec<ProposerStat> {
    let total_selections: u64 = counts.values().sum();
    let total_selections_f = total_selections.max(1) as f64;

    validators
        .iter()
        .map(|v| {
            let selections = *counts.get(&v.id).unwrap_or(&0);
            let expected_share = v.power as f64 / total_power as f64;
            let actual_share = selections as f64 / total_selections_f;
            ProposerStat {
                id: v.id.clone(),
                power: v.power,
                expected_share,
                actual_share,
                selections,
            }
        })
        .collect()
}

fn percentile(values: &[u64], p: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let idx = ((sorted.len() as f64 - 1.0) * p.clamp(0.0, 1.0)).round() as usize;
    sorted[idx]
}

pub fn print_metrics(metrics: &Metrics) {
    println!("scenario: {}", metrics.scenario);
    println!("finalized_blocks: {}", metrics.finalized_blocks);
    println!(
        "avg_rounds_per_finalized_block: {:.3}",
        metrics.avg_rounds_per_finalized_block
    );
    println!("failed_rounds: {}", metrics.failed_rounds);
    println!("p50_finalize_ticks: {}", metrics.p50_finalize_ticks);
    println!("p95_finalize_ticks: {}", metrics.p95_finalize_ticks);
    println!("proposer_distribution:");
    for stat in &metrics.proposer_stats {
        println!(
            "  {} power={} expected={:.3} actual={:.3} picks={}",
            stat.id, stat.power, stat.expected_share, stat.actual_share, stat.selections
        );
    }
}

pub fn write_json(metrics: &Metrics, path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    let path = path.as_ref().to_path_buf();
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!("  \"scenario\": \"{}\",\n", metrics.scenario));
    out.push_str(&format!(
        "  \"finalized_blocks\": {},\n",
        metrics.finalized_blocks
    ));
    out.push_str(&format!(
        "  \"avg_rounds_per_finalized_block\": {:.6},\n",
        metrics.avg_rounds_per_finalized_block
    ));
    out.push_str(&format!(
        "  \"failed_rounds\": {},\n",
        metrics.failed_rounds
    ));
    out.push_str(&format!(
        "  \"p50_finalize_ticks\": {},\n",
        metrics.p50_finalize_ticks
    ));
    out.push_str(&format!(
        "  \"p95_finalize_ticks\": {},\n",
        metrics.p95_finalize_ticks
    ));
    out.push_str("  \"proposer_distribution\": [\n");
    for (i, stat) in metrics.proposer_stats.iter().enumerate() {
        out.push_str(&format!(
            "    {{\"id\":\"{}\",\"power\":{},\"expected\":{:.6},\"actual\":{:.6},\"picks\":{}}}",
            stat.id, stat.power, stat.expected_share, stat.actual_share, stat.selections
        ));
        if i + 1 != metrics.proposer_stats.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n");
    out.push_str("}\n");

    fs::write(&path, out)?;
    Ok(path)
}

pub fn write_csv(metrics: &Metrics, path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    let path = path.as_ref().to_path_buf();
    let mut file = fs::File::create(&path)?;
    writeln!(
        file,
        "scenario,finalized_blocks,avg_rounds_per_finalized_block,failed_rounds,p50_finalize_ticks,p95_finalize_ticks"
    )?;
    writeln!(
        file,
        "{},{},{:.6},{},{},{}",
        metrics.scenario,
        metrics.finalized_blocks,
        metrics.avg_rounds_per_finalized_block,
        metrics.failed_rounds,
        metrics.p50_finalize_ticks,
        metrics.p95_finalize_ticks
    )?;
    writeln!(file, "")?;
    writeln!(
        file,
        "validator,power,expected_share,actual_share,selections"
    )?;
    for stat in &metrics.proposer_stats {
        writeln!(
            file,
            "{},{},{:.6},{:.6},{}",
            stat.id, stat.power, stat.expected_share, stat.actual_share, stat.selections
        )?;
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use crate::core::types::{RunConfig, ScenarioKind};

    use super::run_simulation;

    #[test]
    fn deterministic_seed_produces_same_metrics() {
        let cfg = RunConfig::default();
        let a = run_simulation(ScenarioKind::Normal, cfg.clone());
        let b = run_simulation(ScenarioKind::Normal, cfg);
        assert_eq!(a.metrics.finalized_blocks, b.metrics.finalized_blocks);
        assert_eq!(a.metrics.failed_rounds, b.metrics.failed_rounds);
        assert_eq!(a.metrics.p95_finalize_ticks, b.metrics.p95_finalize_ticks);
    }
}
