#![feature(generic_associated_types)]
#![feature(associated_type_defaults)]
#![feature(type_alias_impl_trait)]
#![feature(map_first_last)]
#![feature(async_closure)]
#![no_std]
pub mod client;
pub mod ext;
pub mod provider;
pub mod structs;
extern crate alloc;

pub use client::handler;
pub use client::handler::Handler;
pub use client::handler::QEvent;
pub use client::Client;
pub use engine::error::{RQError, RQResult};
use engine::jce;
pub use engine::msg;
pub use engine::protocol::device;
pub use engine::protocol::version;
pub use nrq_engine as engine;
pub use provider::*;
