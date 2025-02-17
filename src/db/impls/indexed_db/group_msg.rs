use std::ops::Deref;

use futures_channel::oneshot;
use indexmap::IndexMap;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{IdbKeyRange, IdbRequest};
use yew::{AttrValue, Event};

use crate::{
    db::group_msg::GroupMessages,
    model::message::{Message, ServerResponse},
};

use super::{
    repository::Repository, GROUP_MSG_TABLE_NAME, MESSAGE_FRIEND_ID_INDEX, MESSAGE_ID_INDEX,
};

pub struct GroupMsgRepo(Repository);
impl Deref for GroupMsgRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl GroupMsgRepo {
    pub async fn new() -> Self {
        Self(Repository::new().await)
    }
}
#[async_trait::async_trait(?Send)]
impl GroupMessages for GroupMsgRepo {
    async fn put(&self, group: &Message) -> Result<(), JsValue> {
        let store = self.store(GROUP_MSG_TABLE_NAME).await?;
        let value = serde_wasm_bindgen::to_value(group)?;
        store.put(&value)?;
        Ok(())
    }

    /// friend id is group id
    /// send id is group member id
    async fn get_messages(
        &self,
        friend_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<IndexMap<AttrValue, Message>, JsValue> {
        let mut counter = 0;
        let mut advanced = true;
        // use channel to get messages
        let (tx, rx) = oneshot::channel::<IndexMap<AttrValue, Message>>();

        let store = self.store(GROUP_MSG_TABLE_NAME).await.unwrap();
        let rang = IdbKeyRange::only(&JsValue::from(friend_id));

        // use friend id as group id
        let index = store.index(MESSAGE_FRIEND_ID_INDEX).unwrap();
        let request = index
            .open_cursor_with_range_and_direction(
                &JsValue::from(&rang.unwrap()),
                web_sys::IdbCursorDirection::Prev,
            )
            .unwrap();
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("读取数据失败").into());
            web_sys::console::log_1(&event.into());
        });

        let messages = std::rc::Rc::new(std::cell::RefCell::new(IndexMap::new()));
        let messages = messages.clone();
        let mut tx = Some(tx);
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        let success = Closure::wrap(Box::new(move |event: &Event| {
            let target = event.target().expect("msg");
            let req = target
                .dyn_ref::<IdbRequest>()
                .expect("Event target is IdbRequest; qed");
            let result = req.result().unwrap_or_else(|_err| JsValue::null());

            if !result.is_null() {
                let cursor = result
                    .dyn_ref::<web_sys::IdbCursorWithValue>()
                    .expect("result is IdbCursorWithValue; qed");
                if page > 1 && advanced {
                    advanced = false;
                    cursor.advance((page - 1) * page_size).unwrap();
                    return;
                }
                let value = cursor.value().unwrap();
                // 反序列化
                if let Ok(msg) = serde_wasm_bindgen::from_value::<Message>(value) {
                    let id = msg.local_id.clone();
                    messages.borrow_mut().insert(id, msg);
                }
                counter += 1;
                if counter >= page_size {
                    let _ = tx.take().unwrap().send(messages.borrow().clone());
                    return;
                }
                let _ = cursor.continue_();
            } else {
                // 如果为null说明已经遍历完成
                //将总的结果发送出来
                let _ = tx.take().unwrap().send(messages.borrow().clone());
            }
        }) as Box<dyn FnMut(&Event)>);

        request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
        success.forget();
        Ok(rx.await.unwrap())
    }

    async fn get_last_msg(&self, group_id: &str) -> Result<Message, JsValue> {
        // 使用channel异步获取数据
        let (tx, rx) = oneshot::channel::<Message>();
        let store = self.store(GROUP_MSG_TABLE_NAME).await.unwrap();

        // let rang = IdbKeyRange::bound(&JsValue::from(0), &JsValue::from(100));
        let rang = IdbKeyRange::only(&JsValue::from(group_id));
        let index = store.index(MESSAGE_FRIEND_ID_INDEX).unwrap();

        let request = index
            .open_cursor_with_range_and_direction(
                &JsValue::from(&rang.unwrap()),
                web_sys::IdbCursorDirection::Prev,
            )
            .unwrap();
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("读取数据失败").into());
            web_sys::console::log_1(&event.into());
        });

        let mut tx = Some(tx);
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        let success = Closure::wrap(Box::new(move |event: &Event| {
            let target = event.target().expect("msg");
            let req = target
                .dyn_ref::<IdbRequest>()
                .expect("Event target is IdbRequest; qed");
            let result = req.result().unwrap_or_else(|_err| JsValue::null());

            if !result.is_null() {
                let cursor = result
                    .dyn_ref::<web_sys::IdbCursorWithValue>()
                    .expect("result is IdbCursorWithValue; qed");

                let value = cursor.value().unwrap();
                // 反序列化
                let msg: Message = serde_wasm_bindgen::from_value(value).unwrap();

                let _ = tx.take().unwrap().send(msg);
            } else {
                let _ = tx.take().unwrap().send(Message::default());
            }
        }) as Box<dyn FnMut(&Event)>);

        request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
        success.forget();
        Ok(rx.await.unwrap())
    }

    async fn update_msg_status(&self, msg: &ServerResponse) -> Result<(), JsValue> {
        let store = self.store(GROUP_MSG_TABLE_NAME).await.unwrap();
        let index = store.index(MESSAGE_ID_INDEX).unwrap();
        let req = index.get(&JsValue::from(msg.local_id.as_str()))?;
        let store = store.clone();
        let send_status = msg.send_status.clone();
        let server_id = msg.server_id.clone();
        let onsuccess = Closure::once(move |event: &Event| {
            let value = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            if !value.is_undefined() && !value.is_null() {
                let mut result: Message = serde_wasm_bindgen::from_value(value).unwrap();
                result.send_status = send_status;
                result.server_id = server_id;
                store
                    .put(&serde_wasm_bindgen::to_value(&result).unwrap())
                    .unwrap();
            }
        });
        req.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        onsuccess.forget();
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("消息存储失败").into());
            web_sys::console::log_1(&event.into());
        });
        req.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        on_add_error.forget();
        Ok(())
    }
}
