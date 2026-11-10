use amm_core::services::DescriptorChecker;

const DESCRIPTOR: &str = include_str!("../descriptor.hcl");

#[test]
fn the_descriptor_passes_the_manager_checker_with_no_claim_keybind_or_setting() {
    let descriptor = DescriptorChecker::new(128, 256).parse(DESCRIPTOR).expect("the descriptor parses and checks");

    assert_eq!(descriptor.namespace, "alaydriem");
    assert_eq!(descriptor.name, "peek");
    assert_eq!(descriptor.id(), "@alaydriem/peek");
    assert_eq!(descriptor.display_name, "Shulker Peek");
    assert_eq!(descriptor.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(descriptor.author, "Alaydriem");
    assert!(descriptor.claims.is_empty());
    assert!(descriptor.keybinds.is_empty());
    assert!(descriptor.settings.is_empty());
}
