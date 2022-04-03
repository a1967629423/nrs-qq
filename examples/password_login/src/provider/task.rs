use futures::Future;
use nrs_qq::provider::{TJoinHandle, TaskProvider};
use tokio::task::{JoinError, JoinHandle};

pub struct MyTaskProvider;

pub struct MyJoinHandle<T>(JoinHandle<T>);

impl<T: Send> Future for MyJoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::pin::Pin::new(&mut self.0).poll(cx)
    }
}

impl<T: Send> TJoinHandle<T> for MyJoinHandle<T> {
    type Error = JoinError;
    fn detach(self) -> () {}
}

impl TaskProvider for MyTaskProvider {
    type YieldFuture = impl Future<Output = ()> + Send;

    type SleepFuture = impl Future<Output = ()> + Send;

    type TimeoutError = ();

    type TimeoutFuture<T: futures::Future + Send> =
        impl Future<Output = Result<T::Output, Self::TimeoutError>> + Send;

    type SpawnJoinHandle<T: Send> = MyJoinHandle<T>;

    fn yield_now() -> Self::YieldFuture {
        tokio::task::yield_now()
    }

    fn sleep(duration: core::time::Duration) -> Self::SleepFuture {
        tokio::time::sleep(duration)
    }

    fn timeout<T: futures::Future + Send>(
        duration: core::time::Duration,
        future: T,
    ) -> Self::TimeoutFuture<T> {
        let f = tokio::time::timeout(duration, future);

        async move { f.await.map_err(|_| ()) }
    }

    fn spawn<T>(future: T) -> Self::SpawnJoinHandle<T::Output>
    where
        T: futures::Future + Send + 'static,
        T::Output: Send + 'static,
    {
        let res = tokio::spawn(future);
        MyJoinHandle(res)
    }
}
