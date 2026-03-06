use std::{env, path::PathBuf, process};

use commonware_poh_consensus_demo::core::{
    simulator::{print_metrics, write_csv, write_json},
    types::{RunConfig, ScenarioKind},
};
use commonware_poh_consensus_demo::run_simulation;

fn main() {
    let args: Vec<String> = env::args().collect();
    let parsed = parse_args(&args).unwrap_or_else(|err| {
        eprintln!("{err}");
        print_usage();
        process::exit(2);
    });

    let mut cfg = RunConfig::default();
    cfg.seed = parsed.seed;
    cfg.heights = parsed.heights;
    cfg.max_rounds = parsed.max_rounds;

    let result = run_simulation(parsed.scenario, cfg);
    print_metrics(&result.metrics);

    if let Some(path) = parsed.json_out {
        match write_json(&result.metrics, &path) {
            Ok(p) => println!("json_written: {}", p.display()),
            Err(err) => {
                eprintln!("failed to write JSON output: {err}");
                process::exit(1);
            }
        }
    }

    if let Some(path) = parsed.csv_out {
        match write_csv(&result.metrics, &path) {
            Ok(p) => println!("csv_written: {}", p.display()),
            Err(err) => {
                eprintln!("failed to write CSV output: {err}");
                process::exit(1);
            }
        }
    }
}

#[derive(Debug)]
struct Args {
    scenario: ScenarioKind,
    seed: u64,
    heights: u64,
    max_rounds: u64,
    json_out: Option<PathBuf>,
    csv_out: Option<PathBuf>,
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut scenario = ScenarioKind::Normal;
    let mut seed = 42u64;
    let mut heights = 30u64;
    let mut max_rounds = 8u64;
    let mut json_out = None;
    let mut csv_out = None;

    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--scenario" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "missing value for --scenario".to_string())?;
                scenario = value.parse()?;
            }
            "--seed" => {
                i += 1;
                seed = args
                    .get(i)
                    .ok_or_else(|| "missing value for --seed".to_string())?
                    .parse()
                    .map_err(|_| "invalid --seed".to_string())?;
            }
            "--heights" => {
                i += 1;
                heights = args
                    .get(i)
                    .ok_or_else(|| "missing value for --heights".to_string())?
                    .parse()
                    .map_err(|_| "invalid --heights".to_string())?;
            }
            "--max-rounds" => {
                i += 1;
                max_rounds = args
                    .get(i)
                    .ok_or_else(|| "missing value for --max-rounds".to_string())?
                    .parse()
                    .map_err(|_| "invalid --max-rounds".to_string())?;
            }
            "--json-out" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "missing value for --json-out".to_string())?;
                json_out = Some(PathBuf::from(value));
            }
            "--csv-out" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "missing value for --csv-out".to_string())?;
                csv_out = Some(PathBuf::from(value));
            }
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        i += 1;
    }

    Ok(Args {
        scenario,
        seed,
        heights,
        max_rounds,
        json_out,
        csv_out,
    })
}

fn print_usage() {
    let scenarios = ScenarioKind::all()
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    eprintln!("Usage: cargo run -- --scenario <name> [--seed N] [--heights N] [--max-rounds N] [--json-out path] [--csv-out path]");
    eprintln!("Scenarios: {scenarios}");
}
