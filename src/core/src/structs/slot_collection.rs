/// The group of slots of a container screen a hovered slot belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotCollection {
    Container,
    Hotbar,
    Inventory,
    Other,
}

impl SlotCollection {
    /// The name the log lines use.
    pub fn name(self) -> &'static str {
        match self {
            Self::Container => "container",
            Self::Hotbar => "hotbar",
            Self::Inventory => "inventory",
            Self::Other => "other",
        }
    }
}
