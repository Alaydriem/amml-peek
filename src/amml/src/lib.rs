//! @alaydriem/peek as an AMML mod: shows the contents of a shulker box next to the pointer while the pointer is over
//! the box in an inventory or chest screen, drawn through the game's own UI render context and registered with the
//! mod manager.

#![deny(unsafe_code)]

pub mod detours;
pub mod game;
pub mod peek_mod;
pub mod services;
pub mod traits;
pub mod types;

pub use peek_mod::PeekMod;

amml_sdk::amml_mod!(PeekMod);
