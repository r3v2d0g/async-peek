/**************************************************************************************************
 *                                                                                                *
 * This Source Code Form is subject to the terms of the Mozilla Public                            *
 * License, v. 2.0. If a copy of the MPL was not distributed with this                            *
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.                                       *
 *                                                                                                *
 **************************************************************************************************/

// ======================================== Documentation ======================================= \\

//! This crate provides [`AsyncPeek`], a trait to read data asynchronously without removing it
//! from the queue (like when using the blocking methods [`std::net::TcpStream::peek()`] and
//! [`std::net::UdpSocket::peek()`]).

// =========================================== Imports ========================================== \\

use cfg_if::cfg_if;
use core::cmp;
use core::future::Future;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::task::{Context, Poll};
use std::io::Error;

#[cfg(feature = "async-io")]
use std::net::{TcpStream, UdpSocket};

// ========================================= Interfaces ========================================= \\

/// Read data asynchronously without removing it from the queue.
pub trait AsyncPeek {
    /// Attempts to read data into `buf` without removing it from the queue.
    ///
    /// Returns the number of bytes read on success, or [`io::Error`] if an error is encountered.
    ///
    /// If no data is available, the current task is registered to be notified when data becomes
    /// available or the stream is closed, and `Poll::Ready` is returned.
    ///
    /// [`io::Error`]: std::io::Error
    fn poll_peek(
        self: Pin<&mut Self>,
        ctx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>>;
}

/// An extension trait which adds utility methods to [`AsyncPeek`] types.
pub trait AsyncPeekExt: AsyncPeek {
    /// Tries to read data into `buf` without removing it from the queue.
    ///
    /// Returns the number of bytes read, or [`io::Error`] if an error is encountered.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # smol::block_on(async {
    /// #
    /// use async_peek::{AsyncPeek, AsyncPeekExt};
    /// use smol::Async;
    /// use std::net::{TcpStream, ToSocketAddrs};
    ///
    /// let addr = "127.0.0.1:0".to_socket_addrs()?.next().unwrap();
    /// let mut stream = Async::<TcpStream>::connect(addr).await?;
    /// let mut buf = [0; 64];
    ///
    /// let n = stream.peek(&mut buf).await?;
    /// println!("Peeked {} bytes", n);
    /// #
    /// # std::io::Result::Ok(()) });
    /// ```
    ///
    /// [`io::Error`]: std::io::Error
    fn peek<'peek>(&'peek mut self, buf: &'peek mut [u8]) -> Peek<'peek, Self>
    where
        Self: Unpin,
    {
        Peek { peek: self, buf }
    }
}

// ============================================ Types =========================================== \\

/// Future for the [`peek()`] function.
///
/// [`peek()`]: AsyncPeekExt::peek()
pub struct Peek<'peek, P: ?Sized> {
    peek: &'peek mut P,
    buf: &'peek mut [u8],
}

// ======================================== macro_rules! ======================================== \\

macro_rules! impl_for_net {
    ($net:ty) => {
        impl AsyncPeek for $net {
            fn poll_peek(
                mut self: Pin<&mut Self>,
                ctx: &mut Context,
                buf: &mut [u8],
            ) -> Poll<Result<usize, Error>> {
                let fut = self.peek(buf);
                ufut::pin!(fut);

                fut.poll(ctx)
            }
        }
    };
}

// ======================================= impl AsyncPeek ======================================= \\

impl AsyncPeek for &[u8] {
    fn poll_peek(
        self: Pin<&mut Self>,
        _: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>> {
        let len = cmp::min(buf.len(), self.len());
        buf[0..len].copy_from_slice(&self[0..len]);

        Poll::Ready(Ok(len))
    }
}

impl<T> AsyncPeek for &mut T
where
    T: AsyncPeek + Unpin + ?Sized,
{
    fn poll_peek(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>> {
        Pin::new(&mut **self).poll_peek(ctx, buf)
    }
}

impl<T> AsyncPeek for Box<T>
where
    T: AsyncPeek + Unpin + ?Sized,
{
    fn poll_peek(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>> {
        Pin::new(&mut **self).poll_peek(ctx, buf)
    }
}

impl<T> AsyncPeek for Pin<T>
where
    T: DerefMut + Unpin,
    <T as Deref>::Target: AsyncPeek,
{
    fn poll_peek(
        self: Pin<&mut Self>,
        ctx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>> {
        self.get_mut().as_mut().poll_peek(ctx, buf)
    }
}



cfg_if! {
    if #[cfg(feature = "async-io")] {
        impl_for_net!(async_io::Async<TcpStream>);
        impl_for_net!(async_io::Async<UdpSocket>);
        impl_for_net!(&async_io::Async<TcpStream>);
        impl_for_net!(&async_io::Async<UdpSocket>);
    }
}

cfg_if! {
    if #[cfg(feature = "async-net")] {
        impl_for_net!(async_net::TcpStream);
        impl_for_net!(async_net::UdpSocket);
        impl_for_net!(&async_net::TcpStream);
        impl_for_net!(&async_net::UdpSocket);
    }
}

cfg_if! {
    if #[cfg(feature = "async-std")] {
        impl_for_net!(async_std::net::TcpStream);
        impl_for_net!(&async_std::net::TcpStream);
    }
}

cfg_if! {
    if #[cfg(feature = "tokio")] {
        impl_for_net!(tokio::net::TcpStream);
        impl_for_net!(&tokio::net::TcpStream);
    }
}

// ====================================== impl AsyncPeekExt ===================================== \\

impl<P: AsyncPeek + ?Sized> AsyncPeekExt for P {}

// ========================================= impl Future ======================================== \\

impl<P: AsyncPeek + Unpin> Future for Peek<'_, P> {
    type Output = Result<usize, Error>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let this = &mut *self;
        Pin::new(&mut this.peek).poll_peek(ctx, this.buf)
    }
}
