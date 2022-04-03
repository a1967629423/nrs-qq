use nrs_qq::provider::{AsyncRead, AsyncWrite, TcpStreamProvider};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use tokio::{io::AsyncRead as TokioAsyncRead, io::AsyncWrite as TokioAsyncWrite, net::TcpStream};
pub struct MyTcpStreamProvider(TcpStream);

impl MyTcpStreamProvider {
    pub fn new(s: TcpStream) -> Self {
        Self(s)
    }
}

fn convert_buf<'a, 'b>(
    in_buf: &'b mut nrs_qq::client::ReadBuf<'a>,
) -> &'b mut tokio::io::ReadBuf<'a> {
    unsafe {
        // let v = in_buf.unfilled_mut();
        // tokio::io::ReadBuf::new(&mut *(v as *mut [MaybeUninit<u8>] as *mut [u8]))
        &mut *(in_buf as *mut nrs_qq::client::ReadBuf as *mut tokio::io::ReadBuf)
    }
}
impl AsyncRead for MyTcpStreamProvider {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut nrs_qq::client::ReadBuf<'_>,
    ) -> Poll<Result<(), Self::IOError>> {
        Pin::new(&mut self.0)
            .poll_read(cx, &mut convert_buf(buf))
            .map_err(|_| ())
    }
}

impl AsyncWrite for MyTcpStreamProvider {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::IOError>> {
        Pin::new(&mut self.0).poll_write(cx, buf).map_err(|_| ())
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::IOError>> {
        Pin::new(&mut self.0).poll_flush(cx).map_err(|_| ())
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::IOError>> {
        Pin::new(&mut self.0).poll_shutdown(cx).map_err(|_| ())
    }
}

pub fn addrs_conver(n: impl no_std_net::ToSocketAddrs) -> Vec<std::net::SocketAddr> {
    n.to_socket_addrs()
        .unwrap()
        .map(|socket| {
            let s: std::net::SocketAddr = socket.to_string().parse().unwrap();
            s
        })
        .collect::<Vec<_>>()
}
impl TcpStreamProvider for MyTcpStreamProvider {
    type ConnectFuture = Pin<
        Box<
            dyn futures::Future<Output = Result<Self, <Self as TcpStreamProvider>::IOError>>
                + Send
                + 'static,
        >,
    >;

    fn connect<A: no_std_net::ToSocketAddrs>(addr: A) -> Self::ConnectFuture {
        let addrs = addrs_conver(addr);

        Box::pin(async move {
            let s = TcpStream::connect(addrs.as_slice()).await.unwrap();
            let res = MyTcpStreamProvider(s);
            Ok(res)
        })
    }
}
