//! Async SSL streams
//!
//! This is an **experimental an insecure** library, not intended for production
//! use yet. Right now this is largely proof of concept, and soon hopefully it
//! will be much more fleshed out.

extern crate futures;
#[macro_use]
extern crate cfg_if;
#[macro_use]
extern crate log;

use std::io::{self, Read, Write};

use futures::{Task, Poll};
use futures::io::Ready;
use futures::stream::Stream;

cfg_if! {
    if #[cfg(any(feature = "force-openssl",
                 all(not(target_os = "macos"),
                     not(target_os = "windows"))))] {
        mod openssl;
        use self::openssl as imp;

        pub mod backend {
            pub mod openssl {
                pub use openssl::ServerContextExt;
                pub use openssl::ClientContextExt;
            }
        }
    } else if #[cfg(target_os = "macos")] {
        mod secure_transport;
        use self::secure_transport as imp;

        pub mod backend {
            pub mod secure_transport {
                pub use secure_transport::ServerContextExt;
                pub use secure_transport::ClientContextExt;
            }
        }
    } else {
    }
}

pub struct ServerContext {
    inner: imp::ServerContext,
}

pub struct ClientContext {
    inner: imp::ClientContext,
}

pub struct SslStream<S> {
    inner: imp::SslStream<S>,
}

impl ClientContext {
    pub fn new() -> io::Result<ClientContext> {
        imp::ClientContext::new().map(|s| ClientContext { inner: s })
    }

    pub fn handshake<S>(self,
                        domain: &str,
                        stream: S) -> io::Result<SslStream<S>>
        where S: Read + Write + Stream<Item=Ready, Error=io::Error>,
    {
        self.inner.handshake(domain, stream).map(|s| {
            SslStream { inner: s }
        })
    }
}

impl ServerContext {
    pub fn handshake<S>(self, stream: S) -> io::Result<SslStream<S>>
        where S: Read + Write + Stream<Item=Ready, Error=io::Error>,
    {
        self.inner.handshake(stream).map(|s| {
            SslStream { inner: s }
        })
    }
}

impl<S> Stream for SslStream<S>
    where S: Stream<Item=Ready, Error=io::Error>,
{
    type Item = Ready;
    type Error = io::Error;

    fn poll(&mut self, task: &mut Task) -> Poll<Option<Ready>, io::Error> {
        self.inner.poll(task)
    }

    fn schedule(&mut self, task: &mut Task) {
        self.inner.schedule(task)
    }
}

impl<S: Read + Write> Read for SslStream<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<S: Read + Write> Write for SslStream<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}