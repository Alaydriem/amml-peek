pub struct BundleName;

impl BundleName {
    /// Whether the item type name is a bundle, plain or coloured (`minecraft:bundle`, `minecraft:red_bundle`).
    pub fn is_bundle(type_name: &str) -> bool {
        type_name.strip_prefix("minecraft:").unwrap_or(type_name).ends_with("bundle")
    }
}
