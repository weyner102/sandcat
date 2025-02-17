use std::rc::Rc;

use web_sys::HtmlDivElement;
use yew::prelude::*;
use yewdux::Dispatch;

use crate::{
    components::self_info::SelfInfo,
    icons::{ContactsIcon, MessagesIcon},
    model::{user::User, ComponentType},
    state::{AppState, ComponentTypeState, UnreadState},
};

/// 增加双击切换置顶未读消息
pub struct Top {
    node: NodeRef,
    show_info: bool,
    app_state: Rc<AppState>,
    app_s_dis: Dispatch<AppState>,
    com_state: Rc<ComponentTypeState>,
    com_s_dis: Dispatch<ComponentTypeState>,
    unread_state: Rc<UnreadState>,
    _unread_dis: Dispatch<UnreadState>,
}

#[derive(Properties, PartialEq)]
pub struct TopProps {}

pub enum TopMsg {
    UnreadStateChanged(Rc<UnreadState>),
    EmptyCallback,
    ShowInfoPanel,
    SubmitInfo(Box<User>),
    AppStateChanged(Rc<AppState>),
    ComStateChanged(Rc<ComponentTypeState>),
}

impl Component for Top {
    type Message = TopMsg;

    type Properties = TopProps;

    fn create(ctx: &Context<Self>) -> Self {
        let dispatch = Dispatch::global().subscribe(ctx.link().callback(TopMsg::AppStateChanged));
        let com_s_dis = Dispatch::global().subscribe(ctx.link().callback(TopMsg::ComStateChanged));
        let unread_dis =
            Dispatch::global().subscribe(ctx.link().callback(TopMsg::UnreadStateChanged));
        Self {
            node: NodeRef::default(),
            show_info: false,
            app_state: dispatch.get(),
            app_s_dis: dispatch,
            unread_state: unread_dis.get(),
            _unread_dis: unread_dis,
            com_state: com_s_dis.get(),
            com_s_dis,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TopMsg::EmptyCallback => false,
            TopMsg::UnreadStateChanged(state) => {
                self.unread_state = state;
                true
            }
            TopMsg::ShowInfoPanel => {
                self.show_info = !self.show_info;
                true
            }
            TopMsg::SubmitInfo(user) => {
                self.show_info = !self.show_info;
                self.app_s_dis.reduce_mut(|s| s.login_user = *user);
                true
            }
            TopMsg::AppStateChanged(state) => {
                self.app_state = state;
                true
            }
            TopMsg::ComStateChanged(state) => {
                self.com_state = state;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut msg_class = "top-icon-selected";
        let msg_onclick = if self.com_state.component_type != ComponentType::Messages {
            msg_class = "hover";
            self.com_s_dis
                .reduce_mut_callback(|s| s.component_type = ComponentType::Messages)
        } else {
            ctx.link().callback(move |_| TopMsg::EmptyCallback)
        };
        let mut msg_class = classes!(msg_class);
        msg_class.push("msg-icon");
        let mut contact_class = "top-icon-selected";
        let contact_onclick = if self.com_state.component_type != ComponentType::Contacts {
            contact_class = "hover";
            self.com_s_dis
                .reduce_mut_callback(|s| s.component_type = ComponentType::Contacts)
        } else {
            ctx.link().callback(move |_| TopMsg::EmptyCallback)
        };
        // let mut setting_class = "top-icon-selected";
        // let setting_onclick = if self.state.component_type != ComponentType::Setting {
        //     setting_class = "hover";
        //     self.state
        //         .switch_com_event
        //         .reform(move |_| ComponentType::Setting)
        // } else {
        //     ctx.link().callback(move |_| TopMsg::EmptyCallback)
        // };
        let mut count = html!();
        if self.unread_state.msg_count > 0 {
            count = html! {
                <span class="unread-count">
                    {self.unread_state.msg_count}
                </span>
            };
        }

        let mut info_panel = html!();
        if self.show_info {
            let close = ctx.link().callback(|_| TopMsg::ShowInfoPanel);
            let submit = ctx.link().callback(TopMsg::SubmitInfo);
            info_panel =
                html!(<SelfInfo user={self.app_state.login_user.clone()} {close} {submit} />)
        }
        let onclick = ctx.link().callback(|_| TopMsg::ShowInfoPanel);
        html! {
            <div class="top" ref={self.node.clone()}>
                {info_panel}
                <div class="top-left pointer" {onclick}>
                    <img class="avatar" title={self.app_state.login_user.name.clone()} src={self.app_state.login_user.avatar.clone()} />
                </div>
                <div class="top-right">
                    <span class={msg_class} onclick={msg_onclick}>
                        <MessagesIcon />
                        {count}
                    </span>
                    <span class={contact_class} onclick={contact_onclick}>
                        <ContactsIcon/>
                    </span>
                    // <span class={setting_class} onclick={setting_onclick}>
                    //     <SettingIcon />
                    // </span>
                </div>

            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        let node: HtmlDivElement = self.node.cast().unwrap();
        node.set_attribute("data-tauri-drag-region", "").unwrap();
    }
}
