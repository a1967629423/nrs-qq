use anyhow::Result;
use chrono::Utc;
use nrs_qq::{
    device::Device,
    engine::{
        command::wtlogin::{LoginDeviceLocked, LoginResponse, LoginSuccess},
        init_random_provider, init_timer_provider,
    },
    ext::common::after_login,
    version::{get_version, Protocol},
    Client, ClientProvider, TRwLock,
};
use password_login::provider::{
    channel::MyChannelProvider,
    mutex::MyMutexProvider,
    oneshot::MyOneShotProvider,
    rwlock::MyRwLockProvider,
    task::MyTaskProvider,
    tcp::{addrs_conver, MyTcpStreamProvider},
    MyRandomProvidr, MyTimeProvider,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tracing::Level;
use tracing_subscriber::prelude::*;
pub struct MyClientProvider;
impl ClientProvider for MyClientProvider {
    type CP = MyChannelProvider;
    type OSCP = MyOneShotProvider;
    type RLP = MyRwLockProvider;
    type MP = MyMutexProvider;
    type TCP = MyTcpStreamProvider;
    type TP = MyTaskProvider;
}
type MyClient = Client<MyClientProvider>;

async fn timer(client: Arc<MyClient>, group_code: i64) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    loop {
        interval.tick().await;
        let time = Utc::now()
            .with_timezone(&chrono::FixedOffset::east(8 * 3600))
            .to_rfc3339();

        client
            .send_group_message(
                group_code,
                password_login::make_message(format!("当前时间 {}", time)),
            )
            .await
            .ok();
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    init_random_provider(Box::new(MyRandomProvidr));
    init_timer_provider(Box::new(MyTimeProvider));
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .without_time(),
        )
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("rs_qq", Level::DEBUG)
                .with_target("password_login", Level::DEBUG),
        )
        .init();
    // load uin and password from env
    let uin: i64 = std::env::var("UIN")
        .expect("failed to read UIN from env")
        .parse()
        .expect("failed to parse UIN");

    let password = std::env::var("PASSWORD").expect("failed to read PASSWORD from env");
    let group_code: i64 = std::env::var("GROUP_CODE")
        .expect("failed to read GROUP_CODE from env")
        .parse()
        .unwrap();
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
    let client = Arc::new(MyClient::new(device, get_version(Protocol::IPad), ()));
    let stream = MyTcpStreamProvider::new(
        TcpStream::connect(addrs_conver(client.get_address()).as_slice()).await?,
    );
    let c = client.clone();
    let handle = tokio::spawn(async move { c.start(stream).await });
    tokio::task::yield_now().await; // 等一下，确保连上了

    tracing::info!("准备登录");
    let mut resp = client
        .password_login(uin, &password)
        .await
        .expect("failed to login with password");
    let cc = client.clone();

    loop {
        match resp {
            LoginResponse::Success(LoginSuccess {
                ref account_info, ..
            }) => {
                tracing::info!("login success: {:?}", account_info);
                break;
            }
            LoginResponse::DeviceLocked(LoginDeviceLocked {
                ref sms_phone,
                ref verify_url,
                ref message,
                ..
            }) => {
                tracing::info!("device locked: {:?}", message);
                tracing::info!("sms_phone: {:?}", sms_phone);
                tracing::info!("verify_url: {:?}", verify_url);
                tracing::info!("手机打开url，处理完成后重启程序");
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
        tracing::info!("{:?}", client.friends.read().await);

        client
            .reload_groups()
            .await
            .expect("failed to reload group list");
        let group_list = client.groups.read().await;
        tracing::info!("{:?}", group_list);
    }
    let d = client.get_allowed_clients().await;
    tracing::info!("{:?}", d);
    client
        .send_group_message(
            group_code,
            password_login::make_message("我是一个小闹钟".into()),
        )
        .await
        .unwrap();

    tokio::spawn(async move {
        timer(cc, group_code).await;
    });
    handle.await.unwrap();
    Ok(())
}
