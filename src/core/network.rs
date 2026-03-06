use crate::core::util::SimpleRng;

#[derive(Clone, Debug)]
pub struct NetworkModel {
    base_latency_ticks: u64,
    jitter_ticks: u64,
    drop_rate: f64,
    rng: SimpleRng,
}

impl NetworkModel {
    pub fn new(seed: u64, base_latency_ticks: u64, jitter_ticks: u64, drop_rate: f64) -> Self {
        Self {
            base_latency_ticks,
            jitter_ticks,
            drop_rate,
            rng: SimpleRng::new(seed),
        }
    }

    pub fn sample_delay(&mut self) -> Option<u64> {
        if self.rng.gen_bool(self.drop_rate) {
            return None;
        }

        let jitter = if self.jitter_ticks == 0 {
            0
        } else {
            self.rng.gen_range_inclusive(0, self.jitter_ticks)
        };
        Some(self.base_latency_ticks + jitter)
    }
}
