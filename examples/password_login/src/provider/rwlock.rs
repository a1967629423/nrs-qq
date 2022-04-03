use std::ops::{Deref, DerefMut};

use nrs_qq::provider::{RwLockProvider, TRwLock, TRwLockReadGuard, TRwLockWriteGuard};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
pub struct MyRwLockProvider;
#[derive(Debug)]
pub struct MyRwLock<T>(RwLock<T>);
#[derive(Debug)]
pub struct MyRwLockReadGuard<'a, T>(RwLockReadGuard<'a, T>);

#[derive(Debug)]
pub struct MyRwLockWriteGuard<'a, T>(RwLockWriteGuard<'a, T>);

impl<'a, T> Deref for MyRwLockReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.deref()
    }
}

impl<'a, T> TRwLockReadGuard<'a, T> for MyRwLockReadGuard<'a, T> {}

impl<'a, T> Deref for MyRwLockWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.deref()
    }
}

impl<'a, T> DerefMut for MyRwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0.deref_mut()
    }
}

impl<'a, T> TRwLockWriteGuard<'a, T> for MyRwLockWriteGuard<'a, T> {}

impl<T> TRwLock<T> for MyRwLock<T>
where
    T: Send + Sync,
{
    type ReadGuard<'a>  = MyRwLockReadGuard<'a,T> where Self:'a;

    type WriteGuard<'a> = MyRwLockWriteGuard<'a,T>  where Self:'a;

    type ReadFuture<'a>  = impl futures::Future<Output = Self::ReadGuard<'a>> + Send where Self:'a;

    type WriteFuture<'a>  = impl futures::Future<Output = Self::WriteGuard<'a>> + Send where Self:'a;

    fn new(value: T) -> Self {
        Self(RwLock::new(value))
    }

    fn read(&self) -> Self::ReadFuture<'_> {
        async move {
            let g = self.0.read().await;
            MyRwLockReadGuard(g)
        }
    }

    fn write(&self) -> Self::WriteFuture<'_> {
        async move {
            let g = self.0.write().await;
            MyRwLockWriteGuard(g)
        }
    }
}

impl RwLockProvider for MyRwLockProvider {
    type RwLock<T>  = MyRwLock<T> where T: Send + Sync;
}
