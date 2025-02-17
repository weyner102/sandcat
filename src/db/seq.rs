use wasm_bindgen::JsValue;

use crate::model::seq::Seq;

/// seq's id is always 1
#[async_trait::async_trait(?Send)]
pub trait SeqInterface {
    async fn put(&self, seq: &Seq) -> Result<(), JsValue>;

    async fn get(&self) -> Result<Seq, JsValue>;
}
