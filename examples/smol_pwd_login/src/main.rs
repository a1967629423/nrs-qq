#![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]
use nrs_qq::{
    device::Device,
    engine::{
        command::wtlogin::{LoginDeviceLocked, LoginResponse, LoginSuccess},
        get_timer_provider, init_random_provider, init_timer_provider,
    },
    ext::common::after_login,
    handler::DefaultHandler,
    version::{get_version, Protocol},
    Client, ClientProvider, TRwLock, TcpStreamProvider,
};
use provider::{
    channel::MyChannelProvider,
    engine::{MyRandomProvidr, MyTimeProvider},
    mutex::MyMutexProvider,
    oneshot::MyOneShotProvider,
    rwlock::MyRwLockProvider,
    task::MyTaskProvider,
    tcp::{MyTcpStreamProvider, MyTcpStreamSyncProvider},
};
use smol::{future::yield_now, stream::StreamExt, Timer};
use std::sync::Arc;
use tracing::Level;
use tracing_subscriber::prelude::*;
mod handler;
mod provider;
use handler::MyHandler;
pub struct MyClientProvider;
impl ClientProvider for MyClientProvider {
    type CP = MyChannelProvider;
    type OSCP = MyOneShotProvider;
    type RLP = MyRwLockProvider;
    type MP = MyMutexProvider;
    type TCP = MyTcpStreamSyncProvider;
    type TP = MyTaskProvider;
    type Handler = MyHandler;
}
type MyClient = Client<MyClientProvider>;
use nrs_qq::msg::MessageChain;
pub fn make_message(msg: String) -> MessageChain {
    use nrs_qq::engine::pb::msg::{elem::Elem as EElem, Text};
    MessageChain::new(vec![EElem::Text(Text {
        str: Some(msg),
        ..Default::default()
    })])
}
static UIN: i64 = 1000;
static PASSWORD: &str = "PASSWORD";
static TIME_HOST: &str = "TIME_HOST:7954";
pub static GROUP_CODE: i64 = 1000;
pub static MASTER_UIN: i64 = 1000;

fn app() {
    //let my_timer = MyTimeProvider::new_form_net_sync("192.168.43.229:7000");
    let my_timer = MyTimeProvider::new_form_net_sync(TIME_HOST);
    init_timer_provider(Box::new(my_timer));
    init_random_provider(Box::new(MyRandomProvidr));
    // let password = std::env::var("PASSWORD").expect("failed to read PASSWORD from env");
    let my_device: Device = Device {
        display: "GMC.274685.001".into(),
        product: "iarim".into(),
        device: "sagit".into(),
        board: "eomam".into(),
        model: "MI 6".into(),
        finger_print: "xiaomi/iarim/sagit:10/eomam.200122.001/9285333:user/release-keys".into(),
        boot_id: "6fc5a573-d976-013c-20b4-4c00b3a199e1".into(),
        proc_version: "Linux 5.4.0-54-generic-9AKjXfjq (android-build@google.com)".into(),
        imei: "259136341828576".into(),
        brand: "Xiaomi".into(),
        bootloader: "U-boot".into(),
        base_band: "".into(),
        version: nrs_qq::device::OSVersion {
            incremental: "5891938".into(),
            release: "10".into(),
            codename: "REL".into(),
            sdk: 29,
        },
        sim_info: "T-Mobile".into(),
        os_type: "android".into(),
        mac_address: "00:50:56:C0:00:08".into(),
        ip_address: vec![10, 0, 1, 3],
        wifi_bssid: "00:50:56:C0:00:08".into(),
        wifi_ssid: "<unknown ssid>".into(),
        imsi_md5: vec![
            160, 148, 68, 243, 199, 78, 44, 171, 87, 226, 130, 80, 163, 39, 126, 140,
        ],
        android_id: "d8d603f0a7f4d8d2".into(),
        apn: "wifi".into(),
        vendor_name: "MIUI".into(),
        vendor_os_name: "gmc".into(),
    };
    let device = my_device;
    // let device = match Path::new("device.json").exists() {
    //     true => serde_json::from_str(
    //         &tokio::fs::read_to_string("device.json")
    //             .await
    //             .expect("failed to read device.json"),
    //     )
    //     .expect("failed to parse device info"),
    //     false => {
    //         let d = Device::random();
    //         tokio::fs::write("device.json", serde_json::to_string(&d).unwrap())
    //             .await
    //             .expect("failed to write device info to file");
    //         d
    //     }
    // };
    let client = Arc::new(MyClient::new(
        device,
        get_version(Protocol::IPad),
        MyHandler::new(),
    ));

    // load uin and
    smol::block_on(async move {
        let stream = MyTcpStreamSyncProvider::connect(client.get_address())
            .await
            .unwrap();
        let c = client.clone();
        let handler = smol::spawn(async move {
            c.start(stream).await;
        });
        yield_now().await;
        // tracing::info!("准备登录");
        println!("准备登录");
        let mut resp = client
            .password_login(UIN, PASSWORD)
            .await
            .expect("failed to login with password");
        loop {
            match resp {
                LoginResponse::Success(LoginSuccess {
                    ref account_info, ..
                }) => {
                    // tracing::info!("login success: {:?}", account_info);
                    println!("login success: {:?}", account_info);
                    break;
                }
                LoginResponse::DeviceLocked(LoginDeviceLocked {
                    ref sms_phone,
                    ref verify_url,
                    ref message,
                    ..
                }) => {
                    // tracing::info!("device locked: {:?}", message);
                    // tracing::info!("sms_phone: {:?}", sms_phone);
                    // tracing::info!("verify_url: {:?}", verify_url);
                    // tracing::info!("手机打开url，处理完成后重启程序");
                    std::process::exit(0);
                    //也可以走短信验证
                    // resp = client.request_sms().await.expect("failed to request sms");
                }
                LoginResponse::DeviceLockLogin { .. } => {
                    resp = client
                        .device_lock_login()
                        .await
                        .expect("failed to login with device lock");
                }
                LoginResponse::AccountFrozen => {
                    panic!("account frozen");
                }
                LoginResponse::TooManySMSRequest => {
                    panic!("too many sms request");
                }
                _ => {
                    panic!("unknown login status: ");
                }
            }
        }
        tracing::info!("{:?}", resp);

        after_login(&client).await;
        {
            client
                .reload_friends()
                .await
                .expect("failed to reload friend list");
            println!("{:?}", client.friends.read().await);

            client
                .reload_groups()
                .await
                .expect("failed to reload group list");
            let group_list = client.groups.read().await;
            println!("{:?}", group_list);
        }
        let d = client.get_allowed_clients().await;
        println!("{:?}", d);
        //timer(client, group_code).await;
        // client
        //     .send_group_message(
        //         group_code,
        //         password_login::make_message("我是一个小闹钟".into()),
        //     )
        //     .await
        //     .unwrap();

        // tokio::spawn(async move {
        //     timer(cc, group_code).await;
        // });

        // client.delete_essence_message(1095020555, 8114, 2107692422).await
        // let mem_info = client.get_group_member_info(335783090, 875543543).await;
        // println!("{:?}", mem_info);
        // let mem_list = client.get_group_member_list(335783090).await;
        // println!("{:?}", mem_list);
        handler.await;
    });
}
fn main() {
    app();
}
