// ─── RNG ─────────────────────────────────────────────────────────────────────
pub struct Rng {
    state: u64,
}
impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next_f64() * (hi - lo + 1) as f64) as i64
    }

    pub fn bool(&mut self) -> bool {
        (self.next_u64() >> 63) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_rng() {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64;
        let mut r = Rng::new(seed ^ 0xDEAD_BEEF_1984_CAFE);

        let mut escapes = 0i64;
        for _ in 0..10000 {
            if r.bool() {
                escapes += 1;
            }
        }
        let n = (5000 - escapes).abs();
        assert!(n < 200);
    }
}
