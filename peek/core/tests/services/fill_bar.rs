use crate::support::approx::Approx;
use peek_core::services::FillBar;
use peek_core::structs::Rect;

#[test]
fn fraction_is_the_weight_over_64_clamped() {
    assert_eq!(FillBar::fraction(Some(0)), Some(0.0));
    assert_eq!(FillBar::fraction(Some(32)), Some(0.5));
    assert_eq!(FillBar::fraction(Some(64)), Some(1.0));
    assert_eq!(FillBar::fraction(Some(80)), Some(1.0));
}

#[test]
fn no_weight_and_a_negative_weight_have_no_fraction() {
    assert_eq!(FillBar::fraction(None), None);
    assert_eq!(FillBar::fraction(Some(-1)), None);
}

#[test]
fn the_bar_sits_where_the_durability_bar_goes() {
    let icon = Rect { x0: 10.0, y0: 20.0, x1: 26.0, y1: 36.0 };

    assert!(Approx::rect(FillBar::background(icon), 12.0, 32.5, 25.0, 34.5));
    assert!(Approx::rect(FillBar::foreground(icon, 1.0), 12.0, 32.5, 24.0, 33.5));
    assert!(Approx::rect(FillBar::foreground(icon, 0.5), 12.0, 32.5, 18.0, 33.5));
    assert!(Approx::near(FillBar::foreground(icon, 0.0).width(), 0.0));
}
