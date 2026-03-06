use crate::core::types::PohCheckpoint;
use crate::core::util::{fnv1a64, mix64};

#[derive(Clone, Debug)]
pub struct Poh {
    state: u64,
    tick: u64,
    checkpoint_every: u64,
}

impl Poh {
    pub fn new(checkpoint_every: u64) -> Self {
        Self {
            state: fnv1a64(b"genesis"),
            tick: 0,
            checkpoint_every: checkpoint_every.max(1),
        }
    }

    pub fn advance(&mut self, payload: &str) -> PohCheckpoint {
        self.tick += 1;
        let mut input = Vec::with_capacity(32 + payload.len());
        input.extend_from_slice(&self.state.to_le_bytes());
        input.extend_from_slice(b"|");
        input.extend_from_slice(&self.tick.to_le_bytes());
        input.extend_from_slice(b"|");
        input.extend_from_slice(payload.as_bytes());
        let base = fnv1a64(&input);
        self.state = mix64(base ^ self.tick.rotate_left(17));
        self.checkpoint()
    }

    pub fn checkpoint(&self) -> PohCheckpoint {
        PohCheckpoint {
            tick: self.tick,
            head: format!("{:016x}", self.state),
        }
    }

    pub fn should_emit_checkpoint(&self) -> bool {
        self.tick % self.checkpoint_every == 0
    }
}

#[cfg(test)]
mod tests {
    use super::Poh;

    #[test]
    fn deterministic_for_same_sequence() {
        let mut a = Poh::new(4);
        let mut b = Poh::new(4);

        for event in ["x", "y", "z"] {
            a.advance(event);
            b.advance(event);
        }

        assert_eq!(a.checkpoint(), b.checkpoint());
    }
}
