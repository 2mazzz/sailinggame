/// Seeded gust process (also used, slower, for wind shifts): a low-pass-filtered (Ornstein-Uhlenbeck) noise `g`,
/// kept within [-1, 1]. The wind is `mean * (1 + intensity * g)`.
#[derive(Debug, Clone, Copy)]
pub struct Gusts {
    rng: u64,
    value: f64,
    time_constant: f64,
}

/// Standard deviation of `g` before clamping.
const SIGMA: f64 = 0.5;

impl Gusts {
    /// `time_constant` (s): how long a typical excursion lasts.
    pub fn new(seed: u64, time_constant: f64) -> Self {
        Self {
            rng: seed,
            value: 0.0,
            time_constant,
        }
    }

    /// Current gust factor in [-1, 1].
    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn advance(&mut self, dt: f64) {
        let decay = (-dt / self.time_constant).exp();
        let noise = self.gaussian();
        self.value =
            (self.value * decay + SIGMA * (1.0 - decay * decay).sqrt() * noise).clamp(-1.0, 1.0);
    }

    /// splitmix64, uniform in [0, 1).
    fn uniform(&mut self) -> f64 {
        self.rng = self.rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        (z >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Approximately standard normal (sum of 12 uniforms).
    fn gaussian(&mut self) -> f64 {
        (0..12).map(|_| self.uniform()).sum::<f64>() - 6.0
    }
}
