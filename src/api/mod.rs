use gloo::utils::window;
use std::sync::OnceLock;

use crate::db::TOKEN;

use self::{
    file::FileApi,
    friend::FriendApi,
    group::GroupApi,
    http::{FileHttp, FriendHttp, GroupHttp, MsgHttp, SeqHttp, UserHttp},
    message::MsgApi,
    seq::SeqApi,
    user::UserApi,
};

mod file;
mod friend;
mod group;
mod http;
mod message;
mod seq;
mod user;

pub const AUTHORIZE_HEADER: &str = "Authorization";
pub static TOKEN_VALUE: OnceLock<String> = OnceLock::new();

pub fn token() -> String {
    let token = TOKEN_VALUE
        .get_or_init(|| {
            window()
                .local_storage()
                .unwrap()
                .unwrap()
                .get(TOKEN)
                .unwrap()
                .unwrap()
        })
        .to_string();
    format!("Bearer {}", token)
}

pub fn users() -> Box<dyn UserApi> {
    Box::new(UserHttp::new(token, AUTHORIZE_HEADER.to_string()))
}

pub fn groups() -> Box<dyn GroupApi> {
    Box::new(GroupHttp::new(token(), AUTHORIZE_HEADER.to_string()))
}

pub fn friends() -> Box<dyn FriendApi> {
    Box::new(FriendHttp::new(token(), AUTHORIZE_HEADER.to_string()))
}

pub fn messages() -> Box<dyn MsgApi> {
    Box::new(MsgHttp::new(token(), AUTHORIZE_HEADER.to_string()))
}

pub fn seq() -> Box<dyn SeqApi> {
    Box::new(SeqHttp::new(token(), AUTHORIZE_HEADER.to_string()))
}

pub fn file() -> Box<dyn FileApi> {
    Box::new(FileHttp::new(token(), AUTHORIZE_HEADER.to_string()))
}
