use std::ops::{Deref, DerefMut};

use nrs_qq::provider::{MutexProvider, TMutex, TMutexGuard};
use tokio::sync::{Mutex, MutexGuard};

pub struct MyMutexProvider;

pub struct MyMutex<T>(Mutex<T>);

pub struct MyMutexGuard<'a, T>(MutexGuard<'a, T>);

impl MutexProvider for MyMutexProvider {
    type Mutex<T: Send + Sync> = MyMutex<T>;
}

impl<T: Send + Sync> TMutex<T> for MyMutex<T> {
    type Guard<'a>  = MyMutexGuard<'a,T> where Self:'a;

    type Future<'a>  = impl futures::Future<Output = Self::Guard<'a>> + Send where Self:'a;

    fn new(value: T) -> Self {
        Self(Mutex::new(value))
    }

    fn lock(&self) -> Self::Future<'_> {
        async move {
            let g = self.0.lock().await;
            MyMutexGuard(g)
        }
    }
}

impl<'a, T> Deref for MyMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.deref()
    }
}
impl<'a, T> DerefMut for MyMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0.deref_mut()
    }
}

impl<'a, T> TMutexGuard<'a, T> for MyMutexGuard<'a, T> {}
