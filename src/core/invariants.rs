use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct InvariantTracker {
    finalized_by_height: BTreeMap<u64, String>,
    last_finalized_height: u64,
}

impl InvariantTracker {
    pub fn note_finalization(&mut self, height: u64, block_id: &str) -> Result<(), String> {
        if let Some(existing) = self.finalized_by_height.get(&height) {
            if existing != block_id {
                return Err(format!(
                    "invariant violated: double finalization at height {height}: {existing} vs {block_id}"
                ));
            }
            return Ok(());
        }

        if height < self.last_finalized_height {
            return Err(format!(
                "invariant violated: finalized height regressed from {} to {}",
                self.last_finalized_height, height
            ));
        }

        self.last_finalized_height = height;
        self.finalized_by_height
            .insert(height, block_id.to_string());
        Ok(())
    }

    pub fn ensure_timeout_advanced_round(
        &self,
        timeout_round: u64,
        next_round: Option<u64>,
    ) -> Result<(), String> {
        match next_round {
            Some(round) if round > timeout_round => Ok(()),
            _ => Err(format!(
                "invariant violated: round did not advance after timeout at round {timeout_round}"
            )),
        }
    }
}
