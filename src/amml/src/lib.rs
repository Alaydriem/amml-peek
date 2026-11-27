//! @alaydriem/peek as an AMML mod: shows the contents of a shulker box next to the pointer while the pointer is over
//! the box in an inventory or chest screen, drawn through the UI draw calls of the loader and registered with the mod
//! manager.

#![deny(unsafe_code)]

#[forbid(unsafe_code)]
pub mod game;
#[forbid(unsafe_code)]
pub mod peek_mod;
#[forbid(unsafe_code)]
pub mod services;
#[forbid(unsafe_code)]
pub mod traits;
#[forbid(unsafe_code)]
pub mod types;

pub use peek_mod::PeekMod;

amml_api::amml_mod!(PeekMod);
