pub mod add_friend;
pub mod contacts;
pub mod conv_com;
pub mod list_item;
pub mod right_click_panel;
pub mod top;
pub mod user_info;

use std::rc::Rc;

use yew::prelude::*;
use yewdux::Dispatch;

use crate::components::left::contacts::Contacts;
use crate::components::left::conv_com::Chats;
use crate::components::left::top::Top;
use crate::model::ComponentType;
use crate::state::{AppState, ComponentTypeState};

#[derive(Properties, PartialEq, Debug)]
pub struct LeftProps {
    pub user_id: AttrValue,
}

pub enum LeftMsg {
    StateChanged(Rc<AppState>),
    ComStateChanged(Rc<ComponentTypeState>),
}
pub struct Left {
    _dispatch: Dispatch<AppState>,
    app_state: Rc<AppState>,
    com_type: Rc<ComponentTypeState>,
    _com_dis: Dispatch<ComponentTypeState>,
}

impl Component for Left {
    type Message = LeftMsg;
    type Properties = LeftProps;

    fn create(ctx: &Context<Self>) -> Self {
        let dispatch = Dispatch::global().subscribe(ctx.link().callback(LeftMsg::StateChanged));
        let com_dis = Dispatch::global().subscribe(ctx.link().callback(LeftMsg::ComStateChanged));
        Self {
            app_state: dispatch.get(),
            _dispatch: dispatch,
            com_type: com_dis.get(),
            _com_dis: com_dis,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LeftMsg::StateChanged(state) => {
                self.app_state = state;
                true
            }
            LeftMsg::ComStateChanged(state) => {
                self.com_type = state;
                true
            }
        }
    }

    // 左侧面板总布局,包含顶部选项栏、左侧列表、
    // 点击顶部选项栏切换左侧列表
    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut classes = "slider";
        match self.com_type.component_type {
            ComponentType::Contacts => {
                classes = "slider move-left";
            }
            ComponentType::Messages => {
                classes = "slider ";
                // <Messages />
            }
            ComponentType::Setting => {}
        };
        html! {
            <div class="left-container">
                // 左侧顶部组件：包含头像以及功能切换
                <Top />
                <div class="left-down">
                    <div class={classes}>
                    <Chats user_id={ctx.props().user_id.clone()}
                            avatar={self.app_state.login_user.avatar.clone()} />
                    <Contacts user_id={ctx.props().user_id.clone()}/>
                    </div>
                </div>
            </div>
        }
    }
}
