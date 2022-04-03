use crate::{
    client::event::{
        DeleteFriendEvent, FriendMessageRecallEvent, FriendPokeEvent, FriendRequestEvent,
        GroupLeaveEvent, GroupMessageEvent, GroupMessageRecallEvent, GroupMuteEvent,
        GroupNameUpdateEvent, GroupRequestEvent, MemberPermissionChangeEvent, NewFriendEvent,
        NewMemberEvent, PrivateMessageEvent, SelfInvitedEvent, TempMessageEvent,
    },
    ClientProvider,
};
use core::future::Future;
#[derive(Clone, derivative::Derivative)]
#[derivative(Debug)]
pub enum QEvent<P: ClientProvider> {
    TcpConnect,
    TcpDisconnect,
    /// 登录成功事件
    Login(i64),
    /// 群消息
    GroupMessage(GroupMessageEvent<P>),
    /// 群自身消息
    SelfGroupMessage(GroupMessageEvent<P>),
    /// 私聊消息
    PrivateMessage(PrivateMessageEvent<P>),
    /// 私聊消息
    TempMessage(TempMessageEvent<P>),
    /// 加群申请
    GroupRequest(GroupRequestEvent<P>),
    /// 加群申请
    SelfInvited(SelfInvitedEvent<P>),
    /// 加好友申请
    FriendRequest(FriendRequestEvent<P>),
    /// 新成员入群
    NewMember(NewMemberEvent<P>),
    /// 成员被禁言
    GroupMute(GroupMuteEvent<P>),
    /// 好友消息撤回
    FriendMessageRecall(FriendMessageRecallEvent<P>),
    /// 群消息撤回
    GroupMessageRecall(GroupMessageRecallEvent<P>),
    /// 新好友
    NewFriend(NewFriendEvent<P>),
    /// 退群/被踢
    GroupLeave(GroupLeaveEvent<P>),
    /// 好友戳一戳
    FriendPoke(FriendPokeEvent<P>),
    /// 群名称修改
    GroupNameUpdate(GroupNameUpdateEvent<P>),
    /// 好友删除
    DeleteFriend(DeleteFriendEvent<P>),
    /// 群成员权限变更
    MemberPermissionChange(MemberPermissionChangeEvent<P>),
    // FriendList(decoder::friendlist::FriendListResponse),
    // GroupMemberInfo(structs::GroupMemberInfo),

    // 群消息发送成功事件 内部处理
    // GroupMessageReceipt(GroupMessageReceiptEvent)
}

pub trait Handler<P: ClientProvider>: Send + Sync {
    type Future: Future<Output = ()> + Send;
    fn handle(&self, _event: QEvent<P>) -> Self::Future;
}

/// 一个默认 Handler，只是把信息打印出来
pub struct DefaultHandler;

impl<P> Handler<P> for DefaultHandler
where
    P: ClientProvider,
{
    type Future = core::pin::Pin<alloc::boxed::Box<dyn Future<Output = ()> + Send>>;
    fn handle(&self, e: QEvent<P>) -> Self::Future {
        match e {
            QEvent::GroupMessage(m) => {
                tracing::info!(
                    target = "rs_qq",
                    "MESSAGE (GROUP={}): {}",
                    m.message.group_code,
                    m.message.elements
                )
            }
            QEvent::PrivateMessage(m) => {
                tracing::info!(
                    target = "rs_qq",
                    "MESSAGE (FRIEND={}): {}",
                    m.message.from_uin,
                    m.message.elements
                )
            }
            QEvent::TempMessage(m) => {
                tracing::info!(
                    target = "rs_qq",
                    "MESSAGE (TEMP={}): {}",
                    m.message.from_uin,
                    m.message.elements
                )
            }
            QEvent::GroupRequest(m) => {
                tracing::info!(
                    target = "rs_qq",
                    "REQUEST (GROUP={}, UIN={}): {}",
                    m.request.group_code,
                    m.request.req_uin,
                    m.request.message
                )
            }
            QEvent::FriendRequest(m) => {
                tracing::info!(
                    target = "rs_qq",
                    "REQUEST (UIN={}): {}",
                    m.request.req_uin,
                    m.request.message
                )
            }
            _ => tracing::info!(target = "rs_qq", "unknown"),
        }

        alloc::boxed::Box::pin(futures::future::ready(()))
    }
}
