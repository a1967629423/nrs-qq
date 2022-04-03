use crate::provider::{
    ChannelProvider, ClientProvider, MutexProvider, OneShotChannelProvider, RwLockProvider,
};
use crate::structs::{AccountInfo, AddressInfo, FriendInfo, Group, OtherClientInfo};
use alloc::collections::BTreeMap as HashMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::AtomicI64;
use nrq_engine::command::online_push::GroupMessagePart;
use nrq_engine::command::profile_service::GroupSystemMessages;
use nrq_engine::{protocol::packet::Packet, Engine};

mod api;
mod client;
pub mod event;
pub(crate) mod ext;
pub mod handler;
mod net;
mod processor;
pub use ext::read_buf::ReadBuf;

/// ## TODO: 使用泛型确定handler的future
///
/// 目前为了省事直接Box::pin了
pub struct Client<P>
where
    P: ClientProvider,
{
    // 一堆幽灵
    __p_data: PhantomData<P>,

    handler: P::Handler,

    engine: <P::RLP as RwLockProvider>::RwLock<Engine>,

    // 是否正在运行（是否需要快速重连）
    pub running: AtomicBool,
    // 是否在线（是否可以快速重连）
    pub online: AtomicBool,
    // 停止网络
    disconnect_signal: <P::CP as ChannelProvider>::Sender<()>,
    pub heartbeat_enabled: AtomicBool,

    out_pkt_sender: <P::CP as ChannelProvider>::Sender<bytes::Bytes>,
    packet_promises: <P::RLP as RwLockProvider>::RwLock<
        HashMap<i32, <P::OSCP as OneShotChannelProvider>::Sender<Packet>>,
    >,
    packet_waiters: <P::RLP as RwLockProvider>::RwLock<
        HashMap<String, <P::OSCP as OneShotChannelProvider>::Sender<Packet>>,
    >,
    receipt_waiters: <P::MP as MutexProvider>::Mutex<
        HashMap<i32, <P::OSCP as OneShotChannelProvider>::Sender<i32>>,
    >,

    // account info
    pub account_info: <P::RLP as RwLockProvider>::RwLock<AccountInfo>,

    // address
    pub address: <P::RLP as RwLockProvider>::RwLock<AddressInfo>,
    pub friends: <P::RLP as RwLockProvider>::RwLock<HashMap<i64, Arc<FriendInfo>>>,
    pub groups: <P::RLP as RwLockProvider>::RwLock<HashMap<i64, Arc<Group<P::RLP>>>>,
    pub online_clients: <P::RLP as RwLockProvider>::RwLock<Vec<OtherClientInfo>>,

    // statics
    pub last_message_time: AtomicI64,
    pub start_time: i32,

    // /// 群消息 builder 寄存 <div_seq, parts> : parts is sorted by pkg_index
    // group_message_builder: RwLock<cached::TimedCache<i32, Vec<GroupMessagePart>>>,
    // /// 每个 28 Byte
    // c2c_cache: RwLock<cached::TimedCache<(i64, i64, i32, i64), ()>>,
    // push_req_cache: RwLock<cached::TimedCache<(i16, i64), ()>>,
    // push_trans_cache: RwLock<cached::TimedCache<(i32, i64), ()>>,
    // group_sys_message_cache: RwLock<GroupSystemMessages>,
    /// 群消息 builder 寄存 <div_seq, parts> : parts is sorted by pkg_index
    // group_message_builder:RP::RwLock<caches::RawLRU<i32,Vec<GroupMessagePart>>>,
    // c2c_cache:RP::RwLock<caches::RawLRU<(i64,i64,i32,i64),()>>,
    // push_trans_cache:RP::RwLock<caches::RawLRU<(i32,i64),()>>,
    // group_sys_message_cache:RP::RwLock<GroupSystemMessages>,

    /// 群消息 builder 寄存 <div_seq, parts> : parts is sorted by pkg_index
    group_message_builder: <P::RLP as RwLockProvider>::RwLock<P::Cache<i32, Vec<GroupMessagePart>>>,
    c2c_cache: <P::RLP as RwLockProvider>::RwLock<P::Cache<(i64, i64, i32, i64), ()>>,
    push_req_cache: <P::RLP as RwLockProvider>::RwLock<P::Cache<(i16, i64), ()>>,
    push_trans_cache: <P::RLP as RwLockProvider>::RwLock<P::Cache<(i32, i64), ()>>,
    group_sys_message_cache: <P::RLP as RwLockProvider>::RwLock<GroupSystemMessages>,

    highway_session: <P::RLP as RwLockProvider>::RwLock<nrq_engine::highway::Session>,
    highway_addrs: <P::RLP as RwLockProvider>::RwLock<Vec<no_std_net::SocketAddr>>,
}
unsafe impl<P> Send for crate::Client<P> where P: ClientProvider {}
unsafe impl<P> Sync for crate::Client<P> where P: ClientProvider {}
