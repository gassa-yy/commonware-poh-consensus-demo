use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Validator {
    pub id: String,
    pub power: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PohCheckpoint {
    pub tick: u64,
    pub head: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub height: u64,
    pub round: u64,
    pub proposer_id: String,
    pub checkpoint: PohCheckpoint,
    pub invalid_payload: bool,
    pub equivocation: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoundStatus {
    Finalized,
    PrepareFailed,
    CommitFailed,
    Timeout,
    ByzantineRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoundExecution {
    pub status: RoundStatus,
    pub prepare_power: u64,
    pub commit_power: u64,
    pub quorum: u64,
    pub latency_ticks: u64,
    pub block_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ByzantineMode {
    InvalidProposal,
    Equivocation,
}

#[derive(Clone, Debug)]
pub struct ScenarioConfig {
    pub name: &'static str,
    pub base_latency_ticks: u64,
    pub jitter_ticks: u64,
    pub drop_rate: f64,
    pub prepare_accept_prob: f64,
    pub commit_accept_prob: f64,
    pub timeout_ticks: u64,
    pub byzantine_mode: Option<ByzantineMode>,
    pub byzantine_chance: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScenarioKind {
    Normal,
    HighLatency,
    PacketDrops,
    Byzantine,
    Timeout,
}

impl ScenarioKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ScenarioKind::Normal => "normal",
            ScenarioKind::HighLatency => "high-latency",
            ScenarioKind::PacketDrops => "packet-drops",
            ScenarioKind::Byzantine => "byzantine",
            ScenarioKind::Timeout => "timeout",
        }
    }

    pub fn all() -> [ScenarioKind; 5] {
        [
            ScenarioKind::Normal,
            ScenarioKind::HighLatency,
            ScenarioKind::PacketDrops,
            ScenarioKind::Byzantine,
            ScenarioKind::Timeout,
        ]
    }
}

impl fmt::Display for ScenarioKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ScenarioKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" => Ok(ScenarioKind::Normal),
            "high-latency" => Ok(ScenarioKind::HighLatency),
            "packet-drops" => Ok(ScenarioKind::PacketDrops),
            "byzantine" => Ok(ScenarioKind::Byzantine),
            "timeout" => Ok(ScenarioKind::Timeout),
            _ => Err(format!(
                "unknown scenario '{s}', expected one of: normal, high-latency, packet-drops, byzantine, timeout"
            )),
        }
    }
}

impl ScenarioConfig {
    pub fn for_kind(kind: ScenarioKind) -> Self {
        match kind {
            ScenarioKind::Normal => Self {
                name: "normal",
                base_latency_ticks: 2,
                jitter_ticks: 2,
                drop_rate: 0.01,
                prepare_accept_prob: 0.98,
                commit_accept_prob: 0.98,
                timeout_ticks: 25,
                byzantine_mode: None,
                byzantine_chance: 0.0,
            },
            ScenarioKind::HighLatency => Self {
                name: "high-latency",
                base_latency_ticks: 9,
                jitter_ticks: 7,
                drop_rate: 0.03,
                prepare_accept_prob: 0.98,
                commit_accept_prob: 0.98,
                timeout_ticks: 45,
                byzantine_mode: None,
                byzantine_chance: 0.0,
            },
            ScenarioKind::PacketDrops => Self {
                name: "packet-drops",
                base_latency_ticks: 3,
                jitter_ticks: 3,
                drop_rate: 0.25,
                prepare_accept_prob: 0.96,
                commit_accept_prob: 0.95,
                timeout_ticks: 25,
                byzantine_mode: None,
                byzantine_chance: 0.0,
            },
            ScenarioKind::Byzantine => Self {
                name: "byzantine",
                base_latency_ticks: 3,
                jitter_ticks: 3,
                drop_rate: 0.05,
                prepare_accept_prob: 0.95,
                commit_accept_prob: 0.94,
                timeout_ticks: 25,
                byzantine_mode: Some(ByzantineMode::InvalidProposal),
                byzantine_chance: 0.45,
            },
            ScenarioKind::Timeout => Self {
                name: "timeout",
                base_latency_ticks: 10,
                jitter_ticks: 8,
                drop_rate: 0.18,
                prepare_accept_prob: 0.90,
                commit_accept_prob: 0.88,
                timeout_ticks: 18,
                byzantine_mode: None,
                byzantine_chance: 0.0,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct RunConfig {
    pub seed: u64,
    pub heights: u64,
    pub max_rounds: u64,
    pub max_poh_events_per_round: u64,
    pub checkpoint_every: u64,
    pub validators: Vec<Validator>,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            heights: 30,
            max_rounds: 8,
            max_poh_events_per_round: 3,
            checkpoint_every: 4,
            validators: vec![
                Validator {
                    id: "A".to_string(),
                    power: 5,
                },
                Validator {
                    id: "B".to_string(),
                    power: 3,
                },
                Validator {
                    id: "C".to_string(),
                    power: 2,
                },
            ],
        }
    }
}
