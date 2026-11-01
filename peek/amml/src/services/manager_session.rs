use std::path::Path;

use amm_client::link::{ApiTable, ManagerLink};
use amm_core::errors::LinkError;
use amm_core::structs::event::ModEvent;

use crate::types::ManagerDelta;

/// The registration with the mod manager. The mod has no frame, keybind or setting: the manager only tells it
/// whether the user turned it on or off.
pub struct ManagerSession {
    link: ManagerLink,
}

impl ManagerSession {
    pub const DESCRIPTOR: &'static str = include_str!("../../descriptor.hcl");

    // `config_dir` is the mod's own config folder, an absolute path
    pub fn connect(table: ApiTable, config_dir: &Path) -> Result<Self, LinkError> {
        let link = ManagerLink::without_frame(table, Self::DESCRIPTOR, config_dir)?;
        Ok(Self { link })
    }

    /// Takes the manager's queued events and gives what they changed.
    pub fn drain(&mut self) -> ManagerDelta {
        let mut delta = ManagerDelta::default();
        for event in self.link.events() {
            match event {
                ModEvent::Enabled => delta.on = Some(true),
                ModEvent::Disabled => {
                    delta.on = Some(false);
                    delta.disabled = true;
                }
                _ => {}
            }
        }
        delta
    }
}
