use std::io;
use std::net::Shutdown;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::runtime::{AsyncRead, AsyncWrite};
use super::MaybeTlsStream;

use self::Inner::*;

pub struct MaybeUdsStream {
    inner: Inner,
}

enum Inner {
    MaybeTls(MaybeTlsStream),
    #[cfg(all(feature = "postgres", unix))]
    UnixStream(crate::runtime::UnixStream),
}

impl MaybeUdsStream {
    #[cfg(all(feature = "postgres", unix))]
    pub async fn connect_uds<S: AsRef<std::ffi::OsStr>>(p: S) -> crate::Result<Self> {
        let conn = crate::runtime::UnixStream::connect(p.as_ref()).await?;
        Ok(Self {
            inner: Inner::UnixStream(conn),
        })
    }
    pub async fn connect(host: &str, port: u16) -> crate::Result<Self> {
        let conn = MaybeTlsStream::connect(host, port).await?;
        Ok(Self {
            inner: Inner::MaybeTls(conn),
        })
    }

    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    pub async fn upgrade(
        &mut self,
        host: &str,
        connector: async_native_tls::TlsConnector,
    ) -> crate::Result<()> {
        match &mut self.inner {
            MaybeTls(conn) => conn.upgrade(host, connector).await,
            #[cfg(all(feature = "postgres", unix))]
            UnixStream(_) => {
                return Err(tls_err!("TLS is not supported with unix domain sockets").into())
            }
        }
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        match self.inner {
            MaybeTls(ref conn) => conn.shutdown(how),
            #[cfg(all(feature = "postgres", unix))]
            UnixStream(ref conn) => conn.shutdown(how),
        }
    }
}

macro_rules! forward_pin (
    ($self:ident.$method:ident($($arg:ident),*)) => (
        match &mut $self.inner {
            MaybeTls(ref mut conn) => Pin::new(conn).$method($($arg),*),
            #[cfg(all(feature = "postgres", unix))]
            UnixStream(ref mut conn) => Pin::new(conn).$method($($arg),*),
        }
    )
);

impl AsyncRead for MaybeUdsStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        forward_pin!(self.poll_read(cx, buf))
    }

    #[cfg(feature = "runtime-async-std")]
    fn poll_read_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        bufs: &mut [std::io::IoSliceMut],
    ) -> Poll<io::Result<usize>> {
        forward_pin!(self.poll_read_vectored(cx, bufs))
    }
}

impl AsyncWrite for MaybeUdsStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        forward_pin!(self.poll_write(cx, buf))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        forward_pin!(self.poll_flush(cx))
    }

    #[cfg(feature = "runtime-async-std")]
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        forward_pin!(self.poll_close(cx))
    }

    #[cfg(feature = "runtime-tokio")]
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        forward_pin!(self.poll_shutdown(cx))
    }

    #[cfg(feature = "runtime-async-std")]
    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        bufs: &[std::io::IoSlice],
    ) -> Poll<io::Result<usize>> {
        forward_pin!(self.poll_write_vectored(cx, bufs))
    }
}

