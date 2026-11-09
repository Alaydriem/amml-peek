use alaydriem_peek::services::RenderTelemetry;

#[test]
fn a_report_after_five_seconds_gives_count_mean_p99_and_max() {
    let mut telemetry = RenderTelemetry::new();

    for micros in 1..=99 {
        assert!(telemetry.record(micros as f32, 0.05 * (micros - 1) as f64).is_none());
    }
    let report = telemetry.record(100.0, 5.0).unwrap();

    assert_eq!(report.count, 100);
    assert_eq!(report.mean_us, 50.5);
    assert_eq!(report.p99_us, 99.0);
    assert_eq!(report.max_us, 100.0);
}

#[test]
fn the_next_window_starts_clean() {
    let mut telemetry = RenderTelemetry::new();
    telemetry.record(900.0, 0.0);
    telemetry.record(900.0, 5.0).unwrap();

    assert!(telemetry.record(10.0, 6.0).is_none());
    let report = telemetry.record(30.0, 11.0).unwrap();

    assert_eq!(report.count, 2);
    assert_eq!(report.max_us, 30.0);
    assert_eq!(report.mean_us, 20.0);
}

#[test]
fn the_kept_samples_are_bounded_and_the_oldest_are_overwritten() {
    let mut telemetry = RenderTelemetry::new();
    let total = RenderTelemetry::CAPACITY * 3;

    for index in 0..total {
        telemetry.record(index as f32, 0.0);
    }

    assert_eq!(telemetry.retained(), RenderTelemetry::CAPACITY);
    let report = telemetry.record(0.0, 5.0).unwrap();
    assert_eq!(report.count as usize, total + 1);
    assert_eq!(report.max_us, (total - 1) as f32);
    // the 1024 kept samples are the newest ones, so the 99th percentile is among them
    assert!(report.p99_us >= (total - RenderTelemetry::CAPACITY) as f32);
}
