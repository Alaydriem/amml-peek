use amm_client::link::{LinkState, ManagerLink};
use amm_core::errors::LinkError;
use amm_core::structs::event::ModEvent;
use amml_api::Ctx;

use crate::types::ManagerDelta;

/// The registration with the mod manager. The mod has no frame, keybind or setting: the manager only tells it
/// whether the user turned it on or off. Dropping the link ends the registration.
pub struct PeekLink {
    link: ManagerLink,
}

impl PeekLink {
    pub const DESCRIPTOR: &'static str = include_str!("../../descriptor.hcl");

    /// The link waits for the tables of the manager; `state` says when it is connected.
    pub fn start(ctx: &Ctx) -> Result<Self, LinkError> {
        Ok(Self { link: ManagerLink::start_without_frame(ctx, Self::DESCRIPTOR)? })
    }

    pub fn state(&self) -> LinkState {
        self.link.state()
    }

    /// Takes the queued events of the manager and gives what they changed.
    pub fn drain(&mut self) -> ManagerDelta {
        let mut delta = ManagerDelta::default();
        for event in self.link.events() {
            match event {
                ModEvent::Enabled => delta.on = Some(true),
                ModEvent::Disabled | ModEvent::ManagerGone => {
                    delta.on = Some(false);
                    delta.disabled = true;
                }
                _ => {}
            }
        }
        delta
    }
}
