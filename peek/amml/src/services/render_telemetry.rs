use crate::types::TelemetryReport;

/// The draw times of the panel, reported every few seconds. Bound: `CAPACITY` samples are kept for the percentile;
/// the oldest are overwritten, while the count, the mean and the max cover the whole window.
pub struct RenderTelemetry {
    samples: Vec<f32>,
    scratch: Vec<f32>,
    next: usize,
    filled: usize,
    count: u32,
    sum: f64,
    max: f32,
    window_start: Option<f64>,
}

impl RenderTelemetry {
    pub const CAPACITY: usize = 1024;
    // seconds between two reports
    const WINDOW: f64 = 5.0;

    pub fn new() -> Self {
        Self {
            samples: vec![0.0; Self::CAPACITY],
            scratch: Vec::with_capacity(Self::CAPACITY),
            next: 0,
            filled: 0,
            count: 0,
            sum: 0.0,
            max: 0.0,
            window_start: None,
        }
    }

    /// Adds one draw time at `now` seconds. Gives the report of the window when it has run for five seconds, and
    /// starts the next window.
    pub fn record(&mut self, micros: f32, now: f64) -> Option<TelemetryReport> {
        let start = *self.window_start.get_or_insert(now);
        self.samples[self.next] = micros;
        self.next = (self.next + 1) % Self::CAPACITY;
        self.filled = (self.filled + 1).min(Self::CAPACITY);
        self.count = self.count.saturating_add(1);
        self.sum += f64::from(micros);
        self.max = self.max.max(micros);
        if now - start < Self::WINDOW {
            return None;
        }
        self.scratch.clear();
        self.scratch.extend_from_slice(&self.samples[..self.filled]);
        self.scratch.sort_unstable_by(f32::total_cmp);
        let rank = (self.scratch.len() * 99).div_ceil(100).saturating_sub(1);
        let report = TelemetryReport {
            count: self.count,
            mean_us: (self.sum / f64::from(self.count)) as f32,
            p99_us: self.scratch.get(rank).copied().unwrap_or(0.0),
            max_us: self.max,
        };
        self.next = 0;
        self.filled = 0;
        self.count = 0;
        self.sum = 0.0;
        self.max = 0.0;
        self.window_start = None;
        Some(report)
    }

    /// The samples kept for the percentile.
    pub fn retained(&self) -> usize {
        self.filled
    }
}

impl Default for RenderTelemetry {
    fn default() -> Self {
        Self::new()
    }
}
