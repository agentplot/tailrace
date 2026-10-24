//! One group per part of the core, each named for the core module the
//! plan has build it, over the one [`FakeWorld`](crate::world::FakeWorld)
//! (design D2, D4); the surfaces' group over a surface (design D6).

pub mod cache;
pub mod gate;
pub mod layouts;
pub mod ledger;
pub mod publish;
pub mod record;
pub mod status;
pub mod surfaces;
pub mod verify;
pub mod version;
