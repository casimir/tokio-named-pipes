use std::io;

use futures::{Async, Poll, Stream};
use mio::Ready;

use {NamedPipeListener, NamedPipeStream};

#[must_use = "streams do nothing unless polled"]
#[derive(Debug)]
pub struct Incoming {
    pipe: NamedPipeListener,
}

impl Incoming {
    pub(crate) fn new(listener: NamedPipeListener) -> Incoming {
        Incoming { pipe: listener }
    }
}

impl Stream for Incoming {
    type Item = NamedPipeStream;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, io::Error> {
        try_ready!(self.pipe.poll_read_ready(Ready::readable()));

        match self.pipe.connect() {
            Ok(()) => {
                let stream = NamedPipeStream::new_connection(&self.pipe.path)?;
                Ok(Async::Ready(Some(stream)))
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                try!(self.pipe.clear_read_ready(Ready::readable()));
                Ok(Async::NotReady)
            }
            Err(e) => Err(e),
        }
    }
}
