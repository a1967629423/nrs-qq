use core::fmt::Debug;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use nrq_engine::RQError;

use crate::client::ReadBuf;
use crate::{Handler, QEvent};
use core::hash::Hash;

pub trait TSender<T: Clone + Send>: Clone + Send + Sync {
    type Error: Debug + Send = ();
    type Receiver: TReceiver<T>;
    type SenderFuture<'a>: Future<Output = Result<usize, Self::Error>> + Send + 'a
    where
        Self: 'a;
    fn send(&self, value: T) -> Self::SenderFuture<'_>;
    fn is_closed(&self) -> bool;
    fn close_channel(&self) -> ();
    fn subscribe(&self) -> Self::Receiver;
}
pub trait TReceiver<T>: Send + Sync
where
    T: Send + Clone,
{
    type Error: Debug + Send = ();
    type RecvFuture<'a>: Future<Output = Result<T, Self::Error>>
        + futures::future::FusedFuture
        + Send
        + 'a
    where
        Self: 'a;
    // fn close(&self) -> ();
    fn recv(&mut self) -> Self::RecvFuture<'_>;
}
pub trait TOneShotSender<T>: Send + Sync {
    type Error: Debug + Send = ();
    fn send(self, t: T) -> Result<(), Self::Error>;
}
pub trait TOneShotReceiver<T>: Send + Future<Output = Result<T, Self::Error>> {
    type Error: Debug + Send = ();
}
pub trait ChannelProvider: Sync {
    type Sender<T: 'static + Clone + Send + Sync>: TSender<T>;
    type Receiver<T: 'static + Clone + Send + Sync>: TReceiver<T>;
    fn channel<T: 'static + Clone + Send + Sync>(
        buff: usize,
    ) -> (Self::Sender<T>, Self::Receiver<T>);
}
pub trait OneShotChannelProvider: Sync {
    type Sender<T: Send>: TOneShotSender<T>;
    type Receiver<T: Send>: TOneShotReceiver<T>;
    fn channel<T: Send>() -> (Self::Sender<T>, Self::Receiver<T>);
}
pub trait TRwLockReadGuard<'a, T: ?Sized>: core::ops::Deref<Target = T> {}
pub trait TRwLockWriteGuard<'a, T: ?Sized>:
    core::ops::Deref<Target = T> + core::ops::DerefMut
{
}
pub trait TRwLock<T: ?Sized>: Send + Sync {
    type ReadGuard<'a>: TRwLockReadGuard<'a, T> + Send
    where
        Self: 'a;
    type WriteGuard<'a>: TRwLockWriteGuard<'a, T> + Send
    where
        Self: 'a;
    type ReadFuture<'a>: Future<Output = Self::ReadGuard<'a>> + Send
    where
        Self: 'a;
    type WriteFuture<'a>: Future<Output = Self::WriteGuard<'a>> + Send
    where
        Self: 'a;
    fn new(value: T) -> Self;
    fn read(&self) -> Self::ReadFuture<'_>;
    fn write(&self) -> Self::WriteFuture<'_>;
}
pub trait RwLockProvider: Sync {
    type RwLock<T>: TRwLock<T>
    where
        T: Send + Sync;
}

pub trait TJoinHandle<T>: Send + Sync + Unpin + Future<Output = Result<T, Self::Error>> {
    type Error = ();
    fn detach(self) -> ();
}

pub trait TaskProvider: Sync {
    type YieldFuture: Future<Output = ()> + Send;
    type SleepFuture: Future<Output = ()> + Send;
    type TimeoutError: Debug + Send = ();
    type TimeoutFuture<T: Future + Send>: Future<Output = Result<T::Output, Self::TimeoutError>>
        + Send;
    type SpawnJoinHandle<T: Send>: TJoinHandle<T>;
    fn yield_now() -> Self::YieldFuture;
    fn sleep(duration: core::time::Duration) -> Self::SleepFuture;
    fn timeout<T: Future + Send>(
        duration: core::time::Duration,
        future: T,
    ) -> Self::TimeoutFuture<T>;
    fn spawn<T>(future: T) -> Self::SpawnJoinHandle<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static;
}
pub trait AsyncRead {
    type IOError: Debug = ();
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), Self::IOError>>;
}
pub trait AsyncWrite {
    type IOError: Debug + From<RQError> = ();
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::IOError>>;
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::IOError>>;
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<Result<(), Self::IOError>>;
}

macro_rules! deref_async_write {
    () => {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize, Self::IOError>> {
            Pin::new(&mut **self).poll_write(cx, buf)
        }

        fn poll_flush(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::IOError>> {
            Pin::new(&mut **self).poll_flush(cx)
        }

        fn poll_shutdown(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::IOError>> {
            Pin::new(&mut **self).poll_shutdown(cx)
        }
    };
}

impl<T: ?Sized + AsyncWrite + Unpin> AsyncWrite for alloc::boxed::Box<T> {
    type IOError = T::IOError;
    deref_async_write!();
}

impl<T: ?Sized + AsyncWrite + Unpin> AsyncWrite for &mut T {
    type IOError = T::IOError;
    deref_async_write!();
}

impl AsyncWrite for alloc::vec::Vec<u8> {
    type IOError = ();
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::IOError>> {
        self.get_mut().extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
    }
    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::IOError>> {
        Poll::Ready(Ok(()))
    }
    fn poll_shutdown(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::IOError>> {
        Poll::Ready(Ok(()))
    }
}

impl<P> AsyncWrite for Pin<P>
where
    P: core::ops::DerefMut + Unpin,
    P::Target: AsyncWrite,
{
    type IOError = <<P as core::ops::Deref>::Target as AsyncWrite>::IOError;
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::IOError>> {
        self.get_mut().as_mut().poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::IOError>> {
        self.get_mut().as_mut().poll_flush(cx)
    }
    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::IOError>> {
        self.get_mut().as_mut().poll_shutdown(cx)
    }
}

pub trait TcpStreamProvider: Sized + AsyncRead + AsyncWrite + Sync + Send {
    type IOError: Debug + Send = ();
    type ConnectFuture: Future<Output = Result<Self, <Self as TcpStreamProvider>::IOError>> + Send;
    fn connect<A: no_std_net::ToSocketAddrs>(addr: A) -> Self::ConnectFuture;
}

pub trait TMutexGuard<'a, T: ?Sized>: core::ops::Deref<Target = T> + core::ops::DerefMut {}
pub trait TMutex<T>: Send + Sync {
    type Guard<'a>: TMutexGuard<'a, T> + Send
    where
        Self: 'a;
    type Future<'a>: Future<Output = Self::Guard<'a>> + Send
    where
        Self: 'a;
    fn new(value: T) -> Self;
    fn lock(&self) -> Self::Future<'_>;
}

pub trait MutexProvider: Sync {
    type Mutex<T: Send + Sync>: TMutex<T>;
}
impl<P> Handler<P> for ()
where
    P: ClientProvider,
{
    type Future = impl Future<Output = ()> + Send;
    fn handle(&self, _event: QEvent<P>) -> Self::Future {
        futures::future::ready(())
    }
}
pub trait DataCache<K, V> {
    fn new() -> Self;
    fn cache_get(&self, key: &K) -> Option<&V>;
    fn cache_get_or_set_with<F: FnOnce() -> V>(&mut self, key: K, f: F) -> &mut V;
    fn cache_set(&mut self, key: K, value: V) -> Option<V>;
    fn cache_misses(&self) -> Option<usize>;
    fn flush(&mut self) -> ();
    fn cache_reset_metrics(&mut self) -> ();
    fn cache_remove(&mut self, key: &K) -> Option<V>;
}
/// ## 单个缓存
/// 主要是为了满足cache_get_or_set_with签名，如果能放弃此签名可以干掉缓存
pub struct SingleCache<K, V>(Option<(K, V)>);
impl<K: Ord, V> SingleCache<K, V> {
    pub fn contains_key(&self, key: &K) -> bool {
        if let Some(ref v) = self.0 {
            if &v.0 == key {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if let Some(ref mut v) = self.0 {
            if &v.0 == key {
                Some(&mut v.1)
            } else {
                None
            }
        } else {
            None
        }
    }
}
impl<K: Ord, V> DataCache<K, V> for SingleCache<K, V> {
    fn new() -> Self {
        SingleCache(None)
    }

    fn cache_get(&self, key: &K) -> Option<&V> {
        if let Some(ref v) = self.0 {
            if &v.0 == key {
                Some(&v.1)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn cache_get_or_set_with<F: FnOnce() -> V>(&mut self, key: K, f: F) -> &mut V {
        if self.contains_key(&key) {
            return self.get_mut(&key).unwrap();
        }
        let v = f();
        self.0.replace((key, v));
        &mut self.0.as_mut().unwrap().1
    }

    fn cache_set(&mut self, key: K, value: V) -> Option<V> {
        let old = self.0.replace((key, value));
        old.map(|(_, v)| v)
    }

    fn cache_misses(&self) -> Option<usize> {
        None
    }

    fn flush(&mut self) -> () {}

    fn cache_reset_metrics(&mut self) -> () {}

    fn cache_remove(&mut self, key: &K) -> Option<V> {
        if !self.contains_key(key) {
            return None;
        }
        self.0.take().map(|(_, v)| v)
    }
}
/// ## 客户端内的功能提供
/// ### 其中，必须提供您的功能有:
/// 1. ChannelProvider -- 这里最好提供multicast。mpsc只能运行，在客户端断开时会接收不到断开消息
/// 2. OneShotChannelProvider
/// 3. RwLockProvider -- 异步读写锁，处理好异步死锁问题就行
/// 4. MutexProvider -- 异步互斥锁
/// 5. TaskProvider  -- 异步工具，客户端所用到的一些基础异步工具。这些工具大部分异步运行时都有提供，直接封装就行。
/// 6. TcpStreamProvider -- 异步TCP连接 -- 多线程同步也可以
/// ### 还有部分可选功能:
/// 1.Handler -- 用于处理聊天消息，如果没有提供客户端就不会处理聊天消息
/// 2.Cache   -- 用于缓存数据，如果没有提供客户端会使用默认的单缓存容器
pub trait ClientProvider {
    type CP: ChannelProvider + Send;
    type OSCP: OneShotChannelProvider + Send;
    type RLP: RwLockProvider + Send;
    type MP: MutexProvider + Send;
    type TP: TaskProvider + Send;
    type TCP: TcpStreamProvider + Send;
    type Handler: Handler<Self> + Send + Sync
    where
        Self: Sized,
    = ();
    type Cache<K, V>: DataCache<K, V> + Send + Sync
    where
        K: Ord + Hash + Send + Sync,
        V: Send + Sync,
    = SingleCache<K, V>;
}
