use std::io;

use futures::{Async, Poll, Stream};

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
        let stream = try_ready!(self.pipe.poll_accept());
        Ok(Async::Ready(Some(stream)))
    }
}
