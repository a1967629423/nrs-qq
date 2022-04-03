use crate::MyClientProvider;
use futures::Future;
use nrs_qq::{msg::elem::RQElem, Handler, QEvent};
use std::pin::Pin;

use super::make_message;
use super::{GROUP_CODE, MASTER_UIN};
type MyQEvent = QEvent<MyClientProvider>;
pub struct MyHandler {
    start: std::time::Instant,
}

impl MyHandler {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}
impl Handler<MyClientProvider> for MyHandler {
    //type Future = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
    type Future = impl Future<Output = ()> + Send + 'static;
    fn handle(&self, event: MyQEvent) -> Self::Future {
        let start = self.start;
        println!("handle");
        async move {
            match event {
                QEvent::GroupMessage(e) => {
                    if e.message.group_code != GROUP_CODE {
                        println!("group code {}", e.message.group_code);
                        return;
                    }
                    if e.message.from_uin != MASTER_UIN {
                        println!("from uin {}", e.message.from_uin);
                        return;
                    }
                    let mut content: Option<String> = None;
                    for el in e.message.elements {
                        match el {
                            RQElem::Text(t) => {
                                content.replace(t.content);
                                break;
                            }
                            _ => {}
                        }
                    }
                    if content.is_none() {
                        return;
                    }
                    let content = content.unwrap_or_default();
                    // log::info!("content {}",content);
                    if content == "?" || content == "1" {
                        e.client
                            .send_group_message(e.message.group_code, make_message("?".to_string()))
                            .await
                            .ok();
                        return;
                    }
                }
                _ => {}
            }
        }
    }
}
