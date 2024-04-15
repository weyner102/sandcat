pub mod emoji;
pub mod friend_card;
pub mod friendship_list;
pub mod msg_item;
pub mod msg_list;
pub mod postcard;
pub mod sender;
pub mod set_drawer;
pub mod set_window;

use std::rc::Rc;

use fluent::{FluentBundle, FluentResource};
use yew::platform::spawn_local;
use yew::prelude::*;

use crate::components::left::select_friends::SelectFriendList;
use crate::components::right::friendship_list::FriendShipList;
use crate::components::right::set_window::SetWindow;
use crate::i18n::{en_us, zh_cn, LanguageType};
use crate::icons::{CloseIcon, MaxIcon};
use crate::model::RightContentType;
use crate::model::{ComponentType, ItemInfo};
use crate::pages::{ConvState, CreateConvState, FriendListState, I18nState};
use crate::{
    components::right::{msg_list::MessageList, postcard::PostCard},
    pages::AppState,
};
use crate::{db, tr, utils};

pub struct Right {
    show_setting: bool,
    show_friend_list: bool,
    i18n: FluentBundle<FluentResource>,
    state: Rc<AppState>,
    _ctx_listener: ContextHandle<Rc<AppState>>,
    conv_state: Rc<ConvState>,
    _conv_listener: ContextHandle<Rc<ConvState>>,
    cur_conv_info: Option<Box<dyn ItemInfo>>,
    friend_list_state: Rc<FriendListState>,
    _friend_list_listener: ContextHandle<Rc<FriendListState>>,
    create_conv: Rc<CreateConvState>,
    _create_conv_listener: ContextHandle<Rc<CreateConvState>>,
    lang_state: Rc<I18nState>,
    _lang_state_listener: ContextHandle<Rc<I18nState>>,
}

pub enum RightMsg {
    StateChanged(Rc<AppState>),
    ConvStateChanged(Rc<ConvState>),
    ContentChange(Option<Box<dyn ItemInfo>>),
    FriendListStateChanged(Rc<FriendListState>),
    ShowSetting,
    ShowSelectFriendList,
    CreateGroup(Vec<String>),
    None,
    SwitchLang(Rc<I18nState>),
}

impl Right {
    fn match_content(&mut self, ctx: &Context<Self>) {
        let id = self.conv_state.conv.item_id.clone();
        if id.is_empty() {
            self.cur_conv_info = None;
            return;
        }
        match self.state.component_type {
            ComponentType::Messages => {
                log::debug!(
                    "right conv content type:{:?}",
                    self.conv_state.conv.content_type
                );
                match self.conv_state.conv.content_type {
                    RightContentType::Default => {}
                    RightContentType::Friend => {
                        ctx.link().send_future(async move {
                            let friend = db::friends().await.get_friend(id.as_str()).await;
                            RightMsg::ContentChange(Some(Box::new(friend)))
                        });
                    }
                    RightContentType::Group => {
                        let ctx = ctx.link().clone();
                        spawn_local(async move {
                            if let Ok(Some(group)) = db::groups().await.get(id.as_str()).await {
                                ctx.send_message(RightMsg::ContentChange(Some(Box::new(group))));
                            }
                        });
                    }

                    _ => {}
                }
            }

            _ => self.cur_conv_info = None,
        }
    }
}
impl Component for Right {
    type Message = RightMsg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (state, _ctx_listener) = ctx
            .link()
            .context(ctx.link().callback(RightMsg::StateChanged))
            .expect("expect state");
        let (conv_state, _conv_listener) = ctx
            .link()
            .context(ctx.link().callback(RightMsg::ConvStateChanged))
            .expect("expect state");
        let (friend_list_state, _friend_list_listener) = ctx
            .link()
            .context(ctx.link().callback(RightMsg::FriendListStateChanged))
            .expect("expect state");
        let (create_conv, _create_conv_listener) = ctx
            .link()
            .context(ctx.link().callback(|_| RightMsg::None))
            .expect("expect state");
        let (lang_state, _lang_state_listener) = ctx
            .link()
            .context(ctx.link().callback(RightMsg::SwitchLang))
            .expect("expect state");
        let cur_conv_info = None;
        let res = match lang_state.lang {
            LanguageType::ZhCN => zh_cn::RIGHT_PANEL,
            LanguageType::EnUS => en_us::RIGHT_PANEL,
        };
        let i18n = utils::create_bundle(res);
        Self {
            show_setting: false,
            show_friend_list: false,
            i18n,
            state,
            _ctx_listener,
            conv_state,
            _conv_listener,
            cur_conv_info,
            friend_list_state,
            _friend_list_listener,
            create_conv,
            _create_conv_listener,
            lang_state,
            _lang_state_listener,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            RightMsg::StateChanged(state) => {
                // 根据state中的不同数据变化，渲染右侧页面
                self.state = state;
                // 为了标题栏展示好友名字以及当前会话设置
                self.match_content(ctx);
                true
            }
            RightMsg::ContentChange(item) => {
                self.cur_conv_info = item;
                true
            }
            RightMsg::FriendListStateChanged(state) => {
                self.friend_list_state = state;
                true
            }
            RightMsg::ConvStateChanged(state) => {
                self.conv_state = state;
                self.match_content(ctx);
                true
            }
            RightMsg::ShowSetting => {
                if self.show_friend_list {
                    return false;
                }
                self.show_setting = !self.show_setting;
                true
            }
            RightMsg::ShowSelectFriendList => {
                self.show_friend_list = !self.show_friend_list;
                true
            }
            RightMsg::CreateGroup(nodes) => {
                self.show_friend_list = false;
                if nodes.is_empty() {
                    return true;
                }
                // create group conversation and send 'create group' message
                self.create_conv
                    .create_group
                    .emit((RightContentType::Group, nodes));
                self.show_friend_list = false;
                self.show_setting = false;
                true
            }
            RightMsg::None => false,
            RightMsg::SwitchLang(state) => {
                self.lang_state = state;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut top_bar_info = html!();
        let mut setting = html!();
        let mut friend_list = html!();
        if let Some(info) = &self.cur_conv_info {
            let onclick = ctx.link().callback(|_| RightMsg::ShowSetting);
            let close = ctx.link().callback(|_| RightMsg::ShowSelectFriendList);
            let submit_back = ctx.link().callback(RightMsg::CreateGroup);

            if self.show_setting {
                setting = html! (
                    <SetWindow
                        id={info.id()}
                        conv_type={info.get_type()}
                        close={ctx.link().callback(|_| RightMsg::ShowSetting)}
                        plus_click={close.clone()}
                        lang={self.lang_state.lang} />);
            }
            if self.show_friend_list {
                friend_list = html!(<SelectFriendList
                            except={info.id()}
                            close_back={close}
                            {submit_back}
                            lang={self.lang_state.lang} />);
            }
            top_bar_info = html! {
                <div class="right-top-bar-friend">
                    <span>
                        {info.name()}
                    </span>
                    <span class="pointer" {onclick}>
                        {"···"}
                    </span>
                </div>
            }
        }
        let content = match self.state.component_type {
            ComponentType::Messages => {
                // 处理没有选中会话的情况
                if self.conv_state.conv.item_id.is_empty() {
                    html! {
                        <h2 class="choose-conv">{tr!(self.i18n, "hello")}</h2>
                    }
                } else {
                    html! {
                    <MessageList
                        friend_id={&self.conv_state.conv.item_id.clone()}
                        cur_user_avatar={self.state.login_user.avatar.clone()}
                        conv_type={self.conv_state.conv.content_type.clone()}
                        cur_user_id={self.state.login_user.id.clone()}
                        lang={self.lang_state.lang}/>
                    }
                }
            }
            ComponentType::Contacts => {
                // 要根据右部内容类型绘制页面
                match self.friend_list_state.friend.content_type {
                    RightContentType::Friend
                    | RightContentType::Group
                    | RightContentType::UserInfo => {
                        html! {
                            <PostCard user_id={&self.state.login_user.id.clone()}
                            id={&self.friend_list_state.friend.item_id.clone()}
                            conv_type={self.friend_list_state.friend.content_type.clone()}
                            lang={self.lang_state.lang}/>
                        }
                    }
                    RightContentType::FriendShipList => {
                        log::debug!("right msg container");
                        html! {
                            // self.friendships.iter().map(|item|
                            //
                            // )
                            <FriendShipList lang={self.lang_state.lang}/>
                        }
                    }
                    _ => {
                        html!()
                    }
                }
            }
            ComponentType::Setting => html! {},
        };
        html! {
            <div class="right-container">
                <div class="right-top-bar">
                    <div class="close-bar">
                        <span></span>
                        <MaxIcon/>
                        <CloseIcon/>
                    </div>
                    {top_bar_info}
                </div>
                    {setting}
                    {friend_list}
                <div class="msg-container">
                    {content}
                </div>
            </div>
        }
    }
}
