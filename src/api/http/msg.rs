use async_trait::async_trait;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::{api::message::MsgApi, pb::message::Msg};

use super::RespStatus;

pub struct MsgHttp {
    token: String,
    auth_header: String,
}

impl MsgHttp {
    pub fn new(token: String, auth_header: String) -> Self {
        Self { token, auth_header }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PullOfflineMsgReq {
    pub user_id: String,
    pub start: i64,
    pub end: i64,
}

#[async_trait(?Send)]
impl MsgApi for MsgHttp {
    async fn pull_offline_msg(
        &self,
        user_id: &str,
        start: i64,
        end: i64,
    ) -> Result<Vec<Msg>, JsValue> {
        let requset = PullOfflineMsgReq {
            user_id: user_id.to_string(),
            start,
            end,
        };
        let messages = Request::post("/api/message")
            .header(&self.auth_header, &self.token)
            .json(&requset)
            .map_err(|err| JsValue::from(err.to_string()))?
            .send()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?
            .success()?
            .json()
            .await
            .map_err(|e| JsValue::from(e.to_string()))?;
        Ok(messages)
    }
}
