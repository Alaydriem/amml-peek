/// The draw times of one window, in microseconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TelemetryReport {
    pub count: u32,
    pub mean_us: f32,
    pub p99_us: f32,
    pub max_us: f32,
}
