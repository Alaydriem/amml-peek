use crate::support::approx::Approx;
use peek_core::structs::{DurabilityBar, Rect};

fn bar(damage: i32, max: i32) -> DurabilityBar {
    DurabilityBar::new(true, damage, max).unwrap()
}

#[test]
fn ratio_tracks_remaining_durability() {
    assert!(Approx::near(bar(1, 1561).ratio(), 1560.0 / 1561.0));
    assert!(Approx::near(bar(781, 1562).ratio(), 0.5));
    assert!(bar(1560, 1561).ratio() > 0.0);
    assert!(Approx::near(bar(1561, 1561).ratio(), 0.0));
}

#[test]
fn ratio_clamps_to_empty_when_over_damaged() {
    assert!(Approx::near(bar(150, 100).ratio(), 0.0));
}

#[test]
fn visibility_matches_vanilla_rule() {
    assert!(DurabilityBar::new(true, 0, 1561).is_none());
    assert!(DurabilityBar::new(true, 1, 1561).is_some());
    assert!(DurabilityBar::new(true, 1561, 1561).is_some());
    assert!(DurabilityBar::new(false, 0, 0).is_none());
    assert!(DurabilityBar::new(false, 10, 0).is_none());
    assert!(DurabilityBar::new(true, 5, 0).is_none());
    assert!(DurabilityBar::new(true, -5, 100).is_none());
}

#[test]
fn color_ramp_is_green_yellow_red() {
    // ratio 0.5 is yellow
    let half = bar(1, 2).color();
    assert!(Approx::near(half.r, 1.0) && Approx::near(half.g, 1.0) && Approx::near(half.b, 0.0));
    // ratio 0 is red
    let empty = bar(100, 100).color();
    assert!(Approx::near(empty.r, 1.0) && Approx::near(empty.g, 0.0) && Approx::near(empty.b, 0.0));
    // ratio near 1 is green
    let nearly_full = bar(1, 1_000_000).color();
    assert!(nearly_full.r < 0.01 && Approx::near(nearly_full.g, 1.0));
    // nearly broken stays red-dominant
    let low = bar(95, 100).color();
    assert!(low.r > 0.9 && low.g < 0.4 && Approx::near(low.b, 0.0));
}

#[test]
fn background_matches_vanilla_slot() {
    let icon = Rect { x0: 10.0, y0: 20.0, x1: 26.0, y1: 36.0 };
    let bg = bar(1, 100).background(icon);
    assert!(Approx::near(bg.x0, icon.x0 + 2.0) && Approx::near(bg.x1, icon.x0 + 15.0));
    assert!(Approx::near(bg.y0, icon.y0 + 12.5) && Approx::near(bg.y1, icon.y0 + 14.5));
    assert!(Approx::near(bg.width(), 13.0) && Approx::near(bg.height(), 2.0));
}

#[test]
fn foreground_width_rounds_like_vanilla() {
    let icon = Rect { x0: 0.0, y0: 0.0, x1: 16.0, y1: 16.0 };
    let bg = bar(1, 2).background(icon);
    let half = bar(1, 2).foreground(icon);
    assert!(Approx::near(half.x0, bg.x0) && Approx::near(half.y0, bg.y0) && Approx::near(half.height(), 1.0));
    // iron chestplate 120/240
    assert!(Approx::near(bar(120, 240).foreground(icon).width(), 6.0));
    // bow 100/384
    assert!(Approx::near(bar(100, 384).foreground(icon).width(), 9.0));
    // wooden shovel 30/59
    assert!(Approx::near(bar(30, 59).foreground(icon).width(), 6.0));
    // diamond sword 300/1561
    assert!(Approx::near(bar(300, 1561).foreground(icon).width(), 10.0));
    // iron sword 200/250
    assert!(Approx::near(bar(200, 250).foreground(icon).width(), 2.0));
    // golden pickaxe 31/32: strip only
    assert!(Approx::near(bar(31, 32).foreground(icon).width(), 0.0));
    // one use of many used never exceeds the 12 unit bar
    assert!(bar(1, 1_000_000).foreground(icon).width() <= 12.0);
    // fully used draws no fill
    let empty = bar(100, 100).foreground(icon);
    assert!(Approx::near(empty.width(), 0.0));
    assert!(Approx::near(empty.x0, bg.x0));
}
