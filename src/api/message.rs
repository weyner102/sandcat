use wasm_bindgen::JsValue;

use crate::pb::message::Msg;

#[async_trait::async_trait(?Send)]
pub trait MsgApi {
    async fn pull_offline_msg(
        &self,
        user_id: &str,
        start: i64,
        end: i64,
    ) -> Result<Vec<Msg>, JsValue>;
}
