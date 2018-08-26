use std::fmt;
use std::fs::OpenOptions;
use std::io;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::{FromRawHandle, IntoRawHandle};
use std::path::Path;

use futures::{Async, Future, Poll};
use mio_named_pipes;
use miow::pipe::NamedPipeBuilder;
use tokio::io::{AsyncRead, AsyncWrite};
use winapi::um::winbase::FILE_FLAG_OVERLAPPED;

use NamedPipeListener;

pub struct NamedPipeStream {
    io: NamedPipeListener,
}

#[derive(Debug)]
pub struct ConnectFuture {
    inner: State,
}

#[derive(Debug)]
enum State {
    Waiting(NamedPipeStream),
    Error(io::Error),
    Empty,
}

impl NamedPipeStream {
    pub fn connect(path: &Path) -> ConnectFuture {
        let raw_handle = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(FILE_FLAG_OVERLAPPED)
            .open(path)
            .map(|f| f.into_raw_handle());
        let inner = match raw_handle {
            Ok(handle) => {
                let mio_pipe = unsafe { mio_named_pipes::NamedPipe::from_raw_handle(handle) };
                State::Waiting(NamedPipeStream {
                    io: NamedPipeListener::from_pipe(mio_pipe, &path.to_path_buf()),
                })
            }
            Err(e) => State::Error(e),
        };
        ConnectFuture { inner }
    }

    pub(crate) fn new_connection(path: &Path) -> io::Result<NamedPipeStream> {
        let raw_handle = NamedPipeBuilder::new(path)
            .first(false)
            .inbound(true)
            .outbound(true)
            .out_buffer_size(65536)
            .in_buffer_size(65536)
            .create()?
            .into_raw_handle();
        let mio_pipe = unsafe { mio_named_pipes::NamedPipe::from_raw_handle(raw_handle) };
        Ok(NamedPipeStream {
            io: NamedPipeListener::from_pipe(mio_pipe, &path.to_path_buf()),
        })
    }
}

impl io::Read for NamedPipeStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.io.read(buf)
    }
}

impl AsyncRead for NamedPipeStream {}

impl io::Write for NamedPipeStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.io.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.io.flush()
    }
}

impl AsyncWrite for NamedPipeStream {
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        self.io.shutdown()
    }
}

impl fmt::Debug for NamedPipeStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.io.fmt(f)
    }
}

impl Future for ConnectFuture {
    type Item = NamedPipeStream;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<NamedPipeStream, io::Error> {
        use std::mem;

        match self.inner {
            State::Waiting(ref mut stream) => {
                if let Async::NotReady = stream.io.poll_write_ready()? {
                    return Ok(Async::NotReady);
                }
            }
            State::Error(_) => {
                let e = match mem::replace(&mut self.inner, State::Empty) {
                    State::Error(e) => e,
                    _ => unreachable!(),
                };

                return Err(e);
            }
            State::Empty => panic!("can't poll stream twice"),
        }

        match mem::replace(&mut self.inner, State::Empty) {
            State::Waiting(stream) => Ok(Async::Ready(stream)),
            _ => unreachable!(),
        }
    }
}
