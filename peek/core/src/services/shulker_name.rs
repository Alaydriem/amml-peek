pub struct ShulkerName;

impl ShulkerName {
    pub fn is_shulker(type_name: &str) -> bool {
        type_name.ends_with("shulker_box")
    }
}
