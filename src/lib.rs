#![cfg(windows)]

#[macro_use]
extern crate futures;
extern crate mio;
extern crate mio_named_pipes;
extern crate miow;
extern crate tokio;
extern crate winapi;

mod incoming;
mod listener;
mod stream;

pub use incoming::Incoming;
pub use listener::NamedPipeListener;
pub use stream::NamedPipeStream;
