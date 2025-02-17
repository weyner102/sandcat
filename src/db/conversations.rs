use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use yew::AttrValue;

use crate::model::conversation::Conversation;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Test {
    id: String,
    name: String,
}

#[async_trait::async_trait(?Send)]
pub trait Conversations {
    async fn mute(&self, conv: &Conversation) -> Result<(), JsValue>;

    async fn put_conv(
        &self,
        conv: &Conversation,
        is_clean_unread_count: bool,
    ) -> Result<(), JsValue>;

    async fn get_convs2(&self) -> Result<IndexMap<AttrValue, Conversation>, JsValue>;

    async fn get_by_frined_id(&self, friend_id: &str) -> Conversation;

    async fn delete(&self, friend_id: &str) -> Result<(), JsValue>;
}
