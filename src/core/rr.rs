use std::collections::BTreeMap;

use crate::core::types::Validator;

#[derive(Clone, Debug)]
pub struct WeightedRoundRobin {
    validators: Vec<Validator>,
    priorities: Vec<i128>,
    total_power: i128,
    selections: BTreeMap<String, u64>,
}

impl WeightedRoundRobin {
    pub fn new(validators: Vec<Validator>) -> Self {
        let total_power = validators.iter().map(|v| v.power as i128).sum();
        Self {
            priorities: vec![0; validators.len()],
            validators,
            total_power,
            selections: BTreeMap::new(),
        }
    }

    pub fn select(&mut self) -> Validator {
        for (idx, validator) in self.validators.iter().enumerate() {
            self.priorities[idx] += validator.power as i128;
        }

        let mut winner_idx = 0usize;
        for idx in 1..self.validators.len() {
            let left = (self.priorities[idx], self.validators[idx].id.as_str());
            let right = (
                self.priorities[winner_idx],
                self.validators[winner_idx].id.as_str(),
            );
            if left > right {
                winner_idx = idx;
            }
        }

        self.priorities[winner_idx] -= self.total_power;
        let winner = self.validators[winner_idx].clone();
        *self.selections.entry(winner.id.clone()).or_insert(0) += 1;
        winner
    }

    pub fn selections(&self) -> &BTreeMap<String, u64> {
        &self.selections
    }
}

#[cfg(test)]
mod tests {
    use super::WeightedRoundRobin;
    use crate::core::types::Validator;

    #[test]
    fn converges_to_voting_power_ratio() {
        let mut rr = WeightedRoundRobin::new(vec![
            Validator {
                id: "A".into(),
                power: 6,
            },
            Validator {
                id: "B".into(),
                power: 3,
            },
            Validator {
                id: "C".into(),
                power: 1,
            },
        ]);

        let rounds = 12_000u64;
        for _ in 0..rounds {
            rr.select();
        }

        let picks = rr.selections();
        let a = *picks.get("A").unwrap_or(&0) as f64 / rounds as f64;
        let b = *picks.get("B").unwrap_or(&0) as f64 / rounds as f64;
        let c = *picks.get("C").unwrap_or(&0) as f64 / rounds as f64;

        assert!((a - 0.6).abs() < 0.01);
        assert!((b - 0.3).abs() < 0.01);
        assert!((c - 0.1).abs() < 0.01);
    }
}
