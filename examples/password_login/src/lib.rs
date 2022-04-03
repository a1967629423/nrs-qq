#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use nrs_qq::msg::MessageChain;
pub mod provider;

pub fn make_message(msg: String) -> MessageChain {
    use nrs_qq::engine::pb::msg::{elem::Elem as EElem, Text};
    MessageChain::new(vec![EElem::Text(Text {
        str: Some(msg),
        ..Default::default()
    })])
}
