/// Where the registration with the mod manager stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    /// No manager, or not wanted: the mod does nothing.
    Off,
    /// The manager was not there at enable; the first render call tries again.
    Waiting,
    Attached,
}
