/// What the manager's events of one poll changed.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ManagerDelta {
    /// The on state the last enable or disable event set.
    pub on: Option<bool>,
    /// Whether any disable event came, also when an enable followed it.
    pub disabled: bool,
}
