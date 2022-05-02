use futures::{Future, FutureExt};
use nrs_qq::{ChannelProvider, TReceiver, TSender};
use tokio::sync::broadcast::{channel, error::RecvError, Receiver, Sender};

pub struct MyChannelProvider;
#[derive(Clone)]
pub struct MySender<T>(Sender<T>);

pub struct MyReceiver<T>(Receiver<T>);

impl<T> TReceiver<T> for MyReceiver<T>
where
    T: Send + Clone,
{
    type Error = RecvError;

    type RecvFuture<'a>
    
    = impl futures::Future<Output = Result<T, Self::Error>>
        + futures::future::FusedFuture
        + Send
        + 'a where Self: 'a;

    fn recv<'t>(&'t mut self) -> Self::RecvFuture<'t> {
        self.0.recv().fuse()
    }
}

impl<T: Clone + Send> TSender<T> for MySender<T> {
    type Receiver = MyReceiver<T>;
    type SenderFuture<'a>
    
    = impl Future<Output = Result<usize, Self::Error>> + Send + 'a where Self: 'a;
    fn send(&self, value: T) -> Self::SenderFuture<'_> {
        futures::future::ready(self.0.send(value).map_err(|_| ()))
    }

    fn is_closed(&self) -> bool {
        todo!()
    }

    fn close_channel(&self) -> () {
        todo!()
    }

    fn subscribe(&self) -> Self::Receiver {
        MyReceiver(self.0.subscribe())
    }
}

impl ChannelProvider for MyChannelProvider {
    type Sender<T: 'static + Clone + Send + Sync> = MySender<T>;

    type Receiver<T: 'static + Clone + Send + Sync> = MyReceiver<T>;

    fn channel<T: 'static + Clone + Send + Sync>(
        buff: usize,
    ) -> (Self::Sender<T>, Self::Receiver<T>) {
        let (sender, receiver) = channel::<T>(buff);
        (MySender(sender), MyReceiver(receiver))
    }
}
