use peek_core::services::CountLabel;

#[test]
fn zero_and_one_have_no_label() {
    assert_eq!(CountLabel::text(0), None);
    assert_eq!(CountLabel::text(1), None);
}

#[test]
fn larger_counts_show_the_number() {
    assert_eq!(CountLabel::text(2).as_deref(), Some("2"));
    assert_eq!(CountLabel::text(64).as_deref(), Some("64"));
}
