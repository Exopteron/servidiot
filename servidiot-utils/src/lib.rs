//! Various utilities for the server.
#![feature(trait_alias, downcast_unchecked)]
pub mod resources;
pub mod events;
pub mod ticks;
pub mod typemap;

pub use parking_lot;
