use peek_core::services::ShulkerName;

#[test]
fn shulker_boxes_match() {
    assert!(ShulkerName::is_shulker("minecraft:undyed_shulker_box"));
    assert!(ShulkerName::is_shulker("minecraft:red_shulker_box"));
}

#[test]
fn other_items_do_not_match() {
    assert!(!ShulkerName::is_shulker("minecraft:chest"));
    assert!(!ShulkerName::is_shulker("minecraft:shulker_shell"));
    assert!(!ShulkerName::is_shulker(""));
}
