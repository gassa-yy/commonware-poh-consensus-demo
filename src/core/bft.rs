use crate::core::{
    network::NetworkModel,
    types::{Proposal, RoundExecution, RoundStatus, Validator},
    util::SimpleRng,
};

#[derive(Clone, Debug)]
pub struct BftEngine {
    validators: Vec<Validator>,
    quorum: u64,
}

impl BftEngine {
    pub fn new(validators: Vec<Validator>) -> Self {
        let total_power: u64 = validators.iter().map(|v| v.power).sum();
        let quorum = (2 * total_power) / 3 + 1;
        Self { validators, quorum }
    }

    pub fn quorum(&self) -> u64 {
        self.quorum
    }

    pub fn run_round(
        &self,
        proposal: &Proposal,
        network: &mut NetworkModel,
        rng: &mut SimpleRng,
        prepare_accept_prob: f64,
        commit_accept_prob: f64,
        timeout_ticks: u64,
    ) -> RoundExecution {
        if proposal.invalid_payload || proposal.equivocation {
            return RoundExecution {
                status: RoundStatus::ByzantineRejected,
                prepare_power: 0,
                commit_power: 0,
                quorum: self.quorum,
                latency_ticks: 0,
                block_id: None,
            };
        }

        let (prepare_power, prepare_quorum_at) = self.phase_vote(
            network,
            rng,
            prepare_accept_prob,
            self.quorum,
            proposal.height,
            proposal.round,
            "prepare",
        );

        let prepare_quorum_at = match prepare_quorum_at {
            Some(t) => t,
            None => {
                return RoundExecution {
                    status: RoundStatus::PrepareFailed,
                    prepare_power,
                    commit_power: 0,
                    quorum: self.quorum,
                    latency_ticks: timeout_ticks,
                    block_id: None,
                }
            }
        };

        if prepare_quorum_at > timeout_ticks {
            return RoundExecution {
                status: RoundStatus::Timeout,
                prepare_power,
                commit_power: 0,
                quorum: self.quorum,
                latency_ticks: timeout_ticks,
                block_id: None,
            };
        }

        let (commit_power, commit_quorum_delay) = self.phase_vote(
            network,
            rng,
            commit_accept_prob,
            self.quorum,
            proposal.height,
            proposal.round,
            "commit",
        );

        let commit_quorum_delay = match commit_quorum_delay {
            Some(t) => t,
            None => {
                return RoundExecution {
                    status: RoundStatus::CommitFailed,
                    prepare_power,
                    commit_power,
                    quorum: self.quorum,
                    latency_ticks: timeout_ticks,
                    block_id: None,
                }
            }
        };

        let total_latency = prepare_quorum_at + commit_quorum_delay;
        if total_latency > timeout_ticks {
            return RoundExecution {
                status: RoundStatus::Timeout,
                prepare_power,
                commit_power,
                quorum: self.quorum,
                latency_ticks: timeout_ticks,
                block_id: None,
            };
        }

        let block_id = format!(
            "H{}-R{}-P{}-T{}-{}",
            proposal.height,
            proposal.round,
            proposal.proposer_id,
            proposal.checkpoint.tick,
            &proposal.checkpoint.head[..8]
        );

        RoundExecution {
            status: RoundStatus::Finalized,
            prepare_power,
            commit_power,
            quorum: self.quorum,
            latency_ticks: total_latency,
            block_id: Some(block_id),
        }
    }

    fn phase_vote(
        &self,
        network: &mut NetworkModel,
        rng: &mut SimpleRng,
        accept_prob: f64,
        quorum: u64,
        _height: u64,
        _round: u64,
        _phase: &str,
    ) -> (u64, Option<u64>) {
        let mut votes: Vec<(u64, u64)> = Vec::with_capacity(self.validators.len());
        let mut total_power = 0;

        for validator in &self.validators {
            if !rng.gen_bool(accept_prob) {
                continue;
            }
            if let Some(delay) = network.sample_delay() {
                total_power += validator.power;
                votes.push((delay, validator.power));
            }
        }

        votes.sort_by_key(|x| x.0);
        let mut cumulative = 0;
        for (delay, power) in &votes {
            cumulative += *power;
            if cumulative >= quorum {
                return (total_power, Some(*delay));
            }
        }

        (total_power, None)
    }
}

#[cfg(test)]
mod tests {
    use super::BftEngine;
    use crate::core::{
        network::NetworkModel,
        types::{PohCheckpoint, Proposal, RoundStatus, Validator},
        util::SimpleRng,
    };

    #[test]
    fn transitions_to_finalized_when_quorum_is_reached() {
        let validators = vec![
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
        ];
        let bft = BftEngine::new(validators);
        let proposal = Proposal {
            height: 1,
            round: 0,
            proposer_id: "A".to_string(),
            checkpoint: PohCheckpoint {
                tick: 1,
                head: "abcd".repeat(16),
            },
            invalid_payload: false,
            equivocation: false,
        };

        let mut network = NetworkModel::new(7, 1, 0, 0.0);
        let mut rng = SimpleRng::new(11);
        let result = bft.run_round(&proposal, &mut network, &mut rng, 1.0, 1.0, 20);

        assert_eq!(result.status, RoundStatus::Finalized);
        assert!(result.block_id.is_some());
    }

    #[test]
    fn rejects_invalid_proposal_in_preprepare() {
        let validators = vec![
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
        ];
        let bft = BftEngine::new(validators);
        let proposal = Proposal {
            height: 1,
            round: 0,
            proposer_id: "A".to_string(),
            checkpoint: PohCheckpoint {
                tick: 1,
                head: "abcd".repeat(16),
            },
            invalid_payload: true,
            equivocation: false,
        };

        let mut network = NetworkModel::new(7, 1, 0, 0.0);
        let mut rng = SimpleRng::new(11);
        let result = bft.run_round(&proposal, &mut network, &mut rng, 1.0, 1.0, 20);

        assert_eq!(result.status, RoundStatus::ByzantineRejected);
    }
}
