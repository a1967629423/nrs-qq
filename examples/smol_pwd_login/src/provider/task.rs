use futures::{Future, FutureExt};
use nrs_qq::provider::{TJoinHandle, TaskProvider};
use smol::future::yield_now;
use smol::Task;
use smol::Timer;

pub struct MyJoinHandle<T>(Task<T>);

impl<T: Send> futures::Future for MyJoinHandle<T> {
    type Output = Result<T, <Self as TJoinHandle<T>>::Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.0.poll_unpin(cx).map(|v| Ok(v))
    }
}

struct YieldFuture(bool);
impl YieldFuture {
    pub fn new() -> Self {
        YieldFuture(false)
    }
}
impl futures::Future for YieldFuture {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.0 {
            std::task::Poll::Ready(())
        } else {
            self.0 = true;
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
        }
    }
}

impl<T: Send> TJoinHandle<T> for MyJoinHandle<T> {
    fn detach(self) -> () {
        self.0.detach()
    }
}

pub struct MyTaskProvider;

impl TaskProvider for MyTaskProvider {
    type YieldFuture = impl Future<Output = ()> + Send;

    type SleepFuture = impl Future<Output = ()> + Send;

    type TimeoutFuture<T: futures::Future + Send> =
        impl Future<Output = Result<T::Output, Self::TimeoutError>> + Send;

    type SpawnJoinHandle<T: Send> = MyJoinHandle<T>;

    fn yield_now() -> Self::YieldFuture {
        yield_now()
    }

    fn sleep(duration: core::time::Duration) -> Self::SleepFuture {
        async move {
            Timer::after(duration).await;
        }
    }

    fn timeout<T: futures::Future + Send>(
        duration: core::time::Duration,
        future: T,
    ) -> Self::TimeoutFuture<T> {
        let mut timeout_fu = Box::pin(async move {
            Timer::after(duration).await;
        })
        .fuse();
        let mut fu = Box::pin(async move { future.await }).fuse();
        Box::pin(async move {
            futures::select! {
                _ = timeout_fu => Err(()),
                res = fu => Ok(res)
            }
        })
    }

    fn spawn<T>(future: T) -> Self::SpawnJoinHandle<T::Output>
    where
        T: futures::Future + Send + 'static,
        T::Output: Send + 'static,
    {
        MyJoinHandle(smol::spawn(future))
    }
}
