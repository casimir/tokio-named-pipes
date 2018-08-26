#![cfg(windows)]

extern crate futures;
extern crate tokio;
extern crate tokio_named_pipes;

use tokio::io::{read_to_end, write_all};
use tokio::runtime::current_thread::Runtime;
use tokio_named_pipes::{NamedPipeListener, NamedPipeStream};

use futures::sync::oneshot;
use futures::{Future, Stream};

#[test]
fn echo() {
    let sock_path = r"\\.\pipe\tokio-connect".into();

    let mut rt = Runtime::new().unwrap();

    let server = NamedPipeListener::bind(&sock_path).unwrap();
    let (tx, rx) = oneshot::channel();

    rt.spawn({
        server
            .incoming()
            .into_future()
            .and_then(move |(sock, _)| {
                tx.send(sock.unwrap()).unwrap();
                Ok(())
            })
            .map_err(|e| panic!("err={:?}", e))
    });

    let client = NamedPipeStream::connect(&sock_path).wait().unwrap();
    let server = rt.block_on(rx).unwrap();

    // Write to the client
    rt.block_on(write_all(client, b"hello")).unwrap();

    // Read from the server
    let (_, buf) = rt.block_on(read_to_end(server, vec![])).unwrap();

    assert_eq!(buf, b"hello");
}
