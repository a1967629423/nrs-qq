use alloc::string::String;
use alloc::sync::Arc;
use core::fmt::Debug;
use core::time::Duration;

use bytes::Bytes;

use nrq_engine::command::wtlogin::LoginResponse;

use crate::ext::common::after_login;
use crate::{Client, RQError, RQResult};

use crate::provider::*;

pub trait Connector<P>
where
    P: ClientProvider,
{
    type ConnectError: Debug = ();
    type ConnectFuture: futures::Future<Output = Result<P::TCP, Self::ConnectError>> + Send;
    fn connect(&self, client: &Arc<Client<P>>) -> Self::ConnectFuture;
}

pub struct DefaultConnector;

impl<P> Connector<P> for DefaultConnector
where
    P: ClientProvider,
{
    type ConnectError = <P::TCP as TcpStreamProvider>::IOError;
    type ConnectFuture = impl futures::Future<Output = Result<P::TCP, Self::ConnectError>> + Send;
    fn connect(&self, client: &Arc<Client<P>>) -> Self::ConnectFuture {
        let c = client.clone();
        async move { P::TCP::connect(c.get_address()).await }
    }
}

/// 自动重连，在掉线后使用，会阻塞到重连结束
pub async fn auto_reconnect<P, C: Connector<P> + Sync>(
    client: Arc<Client<P>>,
    credential: Credential,
    interval: Duration,
    max: usize,
    connector: C,
) where
    P: ClientProvider + 'static,
{
    let mut count = 0;
    loop {
        client.stop();
        tracing::error!("client will reconnect after {} seconds", interval.as_secs());
        P::TP::sleep(interval).await;
        let stream = if let Ok(stream) = connector.connect(&client).await {
            count = 0;
            stream
        } else {
            count += 1;
            if count > max {
                tracing::error!("reconnect_count: {}, break!", count);
                break;
            }
            continue;
        };
        let c = client.clone();
        let handle = P::TP::spawn(async move { c.start(stream).await });
        P::TP::yield_now().await; // 等一下，确保连上了
        if let Err(err) = fast_login(client.clone(), credential.clone()).await {
            // token 可能过期了
            tracing::error!("failed to fast_login: {}, break!", err);
            break;
        }
        tracing::info!("succeed to reconnect");
        after_login(&client).await;
        handle.await.ok();
    }
}

pub trait FastLogin<P>
where
    P: ClientProvider,
{
    type LoginFuture: futures::Future<Output = RQResult<()>> + Send;
    fn fast_login(&self, client: Arc<Client<P>>) -> Self::LoginFuture;
}
#[derive(Clone)]
pub struct Token(pub Bytes);
#[derive(Clone)]
pub struct Password {
    pub uin: i64,
    pub password: String,
}
#[derive(Clone)]
pub enum Credential {
    Token(Token),
    Password(Password),
    Both(Token, Password),
}

impl<P> FastLogin<P> for Token
where
    P: ClientProvider + 'static,
{
    type LoginFuture = impl futures::Future<Output = RQResult<()>> + Send;
    fn fast_login(&self, client: Arc<Client<P>>) -> Self::LoginFuture {
        let s = self.clone();
        async move { client.token_login(s.0.clone()).await }
    }
}
impl<P> FastLogin<P> for Password
where
    P: ClientProvider + 'static,
{
    type LoginFuture = impl futures::Future<Output = RQResult<()>> + Send;
    fn fast_login(&self, client: Arc<Client<P>>) -> Self::LoginFuture {
        let s = self.clone();
        async move {
            let resp = client.password_login(s.uin, &s.password).await?;
            match resp {
                LoginResponse::Success { .. } => return Ok(()),
                LoginResponse::DeviceLockLogin { .. } => {
                    return if let LoginResponse::Success { .. } = client.device_lock_login().await?
                    {
                        Ok(())
                    } else {
                        Err(RQError::Other("failed to login".into()))
                    };
                }
                _ => return Err(RQError::Other("failed to login".into())),
            }
        }
    }
}

pub async fn fast_login<P>(client: Arc<Client<P>>, credential: Credential) -> RQResult<()>
where
    P: ClientProvider + 'static,
{
    return match credential {
        Credential::Token(token) => token.fast_login(client).await,
        Credential::Password(password) => password.fast_login(client).await,
        Credential::Both(token, password) => match token.fast_login(client.clone()).await {
            Ok(_) => Ok(()),
            Err(_) => password.fast_login(client).await,
        },
    };
}
