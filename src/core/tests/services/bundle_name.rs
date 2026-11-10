use peek_core::services::BundleName;

#[test]
fn plain_and_coloured_bundles_match() {
    assert!(BundleName::is_bundle("minecraft:bundle"));
    assert!(BundleName::is_bundle("minecraft:red_bundle"));
}

#[test]
fn other_items_do_not_match() {
    assert!(!BundleName::is_bundle("minecraft:bundle_shell"));
    assert!(!BundleName::is_bundle("minecraft:red_shulker_box"));
    assert!(!BundleName::is_bundle(""));
}
