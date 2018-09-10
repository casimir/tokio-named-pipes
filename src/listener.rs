use std::fmt;
use std::io::{self, Read, Write};
use std::os::windows::io::*;
use std::path::PathBuf;

use futures::{Async, Poll};
use mio::Ready;
use mio_named_pipes;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::reactor::PollEvented2;

use {Incoming, NamedPipeStream};

pub struct NamedPipeListener {
    io: PollEvented2<mio_named_pipes::NamedPipe>,
    pub path: PathBuf,
}

impl NamedPipeListener {
    pub fn bind(path: &PathBuf) -> io::Result<NamedPipeListener> {
        let inner = try!(mio_named_pipes::NamedPipe::new(path));
        Ok(NamedPipeListener::from_pipe(inner, path))
    }

    pub fn from_pipe(pipe: mio_named_pipes::NamedPipe, path: &PathBuf) -> NamedPipeListener {
        NamedPipeListener {
            io: PollEvented2::new(pipe),
            path: path.clone(),
        }
    }

    pub fn connect(&self) -> io::Result<()> {
        self.io.get_ref().connect()
    }

    pub fn poll_read_ready(&self, ready: Ready) -> Poll<Ready, io::Error> {
        self.io.poll_read_ready(ready)
    }

    pub fn clear_read_ready(&self, ready: Ready) -> io::Result<()> {
        self.io.clear_read_ready(ready)
    }

    pub fn poll_write_ready(&self) -> Poll<Ready, io::Error> {
        self.io.poll_write_ready()
    }

    pub fn poll_accept(&self) -> Poll<NamedPipeStream, io::Error> {
        try_ready!(self.poll_read_ready(Ready::readable()));

        match self.connect() {
            Ok(()) => {
                let stream = NamedPipeStream::new_connection(&self.path)?;
                Ok(Async::Ready(stream))
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                try!(self.clear_read_ready(Ready::readable()));
                Ok(Async::NotReady)
            }
            Err(e) => Err(e),
        }
    }

    pub fn incoming(self) -> Incoming {
        Incoming::new(self)
    }
}

impl Read for NamedPipeListener {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.io.read(buf)
    }
}

impl AsyncRead for NamedPipeListener {}

impl Write for NamedPipeListener {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.io.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.io.flush()
    }
}

impl AsyncWrite for NamedPipeListener {
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        self.io.shutdown()
    }
}

impl fmt::Debug for NamedPipeListener {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.io.get_ref().fmt(f)
    }
}

impl AsRawHandle for NamedPipeListener {
    fn as_raw_handle(&self) -> RawHandle {
        self.io.get_ref().as_raw_handle()
    }
}
