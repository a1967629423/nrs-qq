use crate::client::event::FriendRequestEvent;
use crate::handler::QEvent;
use crate::provider::*;
use crate::Client;
use crate::Handler;
use alloc::sync::Arc;
use nrq_engine::command::profile_service::FriendSystemMessages;
impl<P> Client<P>
where
    P: ClientProvider,
{
    pub(crate) async fn process_friend_system_messages(
        self: &Arc<Self>,
        msgs: FriendSystemMessages,
    ) {
        for request in msgs.requests {
            self.handler
                .handle(QEvent::FriendRequest(FriendRequestEvent {
                    client: self.clone(),
                    request,
                }))
                .await;
        }
    }
}
