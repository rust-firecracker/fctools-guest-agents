use std::{pin::Pin, task::Poll};

use pin_project_lite::pin_project;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::Stream;
use tokio_vsock::{VsockAddr, VsockListener, VsockStream};
use tonic::transport::server::Connected;

pin_project! {
    pub struct VsockListenerStream {
        #[pin] inner: VsockListener,
    }
}

impl VsockListenerStream {
    pub fn new(vsock_listener: VsockListener) -> Self {
        Self {
            inner: vsock_listener,
        }
    }
}

impl Stream for VsockListenerStream {
    type Item = std::io::Result<VsockStreamConnectable>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.project().inner.poll_accept(cx) {
            Poll::Ready(Ok((stream, _))) => Poll::Ready(Some(Ok(VsockStreamConnectable {
                peer_addr: stream.peer_addr().ok(),
                inner: stream,
            }))),
            Poll::Ready(Err(err)) => Poll::Ready(Some(Err(err))),
            Poll::Pending => Poll::Pending,
        }
    }
}

pin_project! {
    pub struct VsockStreamConnectable {
        peer_addr: Option<VsockAddr>,
        #[pin] inner: VsockStream,
    }
}

impl AsyncRead for VsockStreamConnectable {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.project().inner.poll_read(cx, buf)
    }
}

impl AsyncWrite for VsockStreamConnectable {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        self.project().inner.poll_write(cx, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        self.project().inner.poll_shutdown(cx)
    }
}

#[derive(Debug, Clone)]
pub struct VsockConnectInfo {
    #[allow(unused)]
    pub peer_addr: Option<VsockAddr>,
}

impl Connected for VsockStreamConnectable {
    type ConnectInfo = VsockConnectInfo;

    fn connect_info(&self) -> Self::ConnectInfo {
        VsockConnectInfo {
            peer_addr: self.peer_addr,
        }
    }
}
