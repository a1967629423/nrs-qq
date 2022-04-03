use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Future;
use nrs_qq::provider::{OneShotChannelProvider, TOneShotReceiver, TOneShotSender};
use tokio::sync::oneshot::{channel, error::RecvError, Receiver, Sender};

pub struct MyOneShotProvider;

pub struct MyOneShotSender<T>(Sender<T>);

pub struct MyOneShotReceiver<T>(Receiver<T>);

impl<T: Send> Future for MyOneShotReceiver<T> {
    type Output = Result<T, RecvError>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}

impl<T: Send> TOneShotReceiver<T> for MyOneShotReceiver<T> {
    type Error = RecvError;
}

impl<T: Send> TOneShotSender<T> for MyOneShotSender<T> {
    type Error = ();

    fn send(self, t: T) -> Result<(), Self::Error> {
        self.0.send(t).map_err(|_| ())
    }
}

impl OneShotChannelProvider for MyOneShotProvider {
    type Sender<T: Send> = MyOneShotSender<T>;

    type Receiver<T: Send> = MyOneShotReceiver<T>;

    fn channel<T: Send>() -> (Self::Sender<T>, Self::Receiver<T>) {
        let (s, r) = channel::<T>();
        (MyOneShotSender(s), MyOneShotReceiver(r))
    }
}
