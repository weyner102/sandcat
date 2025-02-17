use std::rc::Rc;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use fluent::{FluentBundle, FluentResource};
use gloo::utils::window;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::scope_ext::RouterScopeExt;
use yewdux::Dispatch;

use crate::db::{self, TOKEN, WS_ADDR};
use crate::i18n::{en_us, zh_cn, LanguageType};
use crate::model::user::User;
use crate::state::{I18nState, ThemeState};
use crate::{api, db::DB_NAME};
use crate::{tr, utils};

use super::Page;

pub struct Login {
    account_ref: NodeRef,
    pwd_ref: NodeRef,
    login_state: LoginState,
    show_error: bool,
    i18n: FluentBundle<FluentResource>,
    lang: LanguageType,
    theme: Dispatch<ThemeState>,
}

pub enum LoginMsg {
    Login,
    Success(AttrValue),
    Failed,
    OnEnterKeyDown(SubmitEvent),
    SwitchLanguage(Event),
    SwitchTheme(Rc<ThemeState>),
}

pub enum LoginState {
    Logining,
    Nothing,
}

async fn login_simulate(account: String, mut password: String) -> Result<Response, JsValue> {
    password = STANDARD.encode(password);
    let body = serde_json::to_string(&LoginRequest { account, password }).unwrap();
    let res = reqwasm::http::Request::post("/api/user/login")
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|err| {
            log::error!("send request error: {:?}", err);
            JsValue::from(err.to_string())
        })?
        .json()
        .await
        .map_err(|err| {
            log::error!("send request error: {:?}", err);
            JsValue::from(err.to_string())
        })?;
    Ok(res)
}

#[derive(Deserialize)]
pub struct Response {
    user: User,
    token: String,
    ws_addr: String,
}

#[derive(Serialize)]
pub struct LoginRequest {
    pub account: String,
    pub password: String,
}

// 模拟输入写入数据库
async fn init_db(id: AttrValue) {
    // 拉取联系人
    // 查询是否需要更新联系人
    match api::friends().get_friend_list_by_id(id.to_string()).await {
        Ok(res) => {
            // 写入数据库
            db::friends().await.put_friend_list(&res).await;
        }
        Err(e) => {
            log::error!("获取联系人列表错误: {:?}", e)
        }
    }
}

impl Component for Login {
    type Message = LoginMsg;

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        // load theme
        let theme =
            Dispatch::<ThemeState>::global().subscribe(_ctx.link().callback(LoginMsg::SwitchTheme));
        // load the i18n bundle
        let lang = Dispatch::<I18nState>::global().get().lang;
        let res = match lang {
            LanguageType::ZhCN => zh_cn::LOGIN,
            LanguageType::EnUS => en_us::LOGIN,
        };
        let i18n = utils::create_bundle(res);

        Self {
            account_ref: NodeRef::default(),
            pwd_ref: NodeRef::default(),
            login_state: LoginState::Nothing,
            show_error: false,
            i18n,
            lang,
            theme,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LoginMsg::Login => {
                // use ref to get the account and password
                let account = self.account_ref.cast::<HtmlInputElement>().unwrap().value();
                let pwd = self.pwd_ref.cast::<HtmlInputElement>().unwrap().value();

                ctx.link().send_future(async move {
                    let res = login_simulate(account, pwd).await;
                    if res.is_err() {
                        return LoginMsg::Failed;
                    }
                    let res = res.unwrap();
                    let user = res.user;
                    // user.login = true;

                    let id = user.id.clone();
                    // 初始化数据库名称,
                    // 这里一定要将所有权传过去，否则会提示将当前域变量转移的问题，因为当前函数结束会将该变量销毁
                    DB_NAME.get_or_init(|| format!("im-{}", id.clone()));
                    // 将用户id写入本地文件
                    //登录成功，初始化数据库

                    window()
                        .local_storage()
                        .unwrap()
                        .unwrap()
                        .set(WS_ADDR, res.ws_addr.as_str())
                        .unwrap();
                    window()
                        .local_storage()
                        .unwrap()
                        .unwrap()
                        .set(TOKEN, res.token.as_str())
                        .unwrap();
                    // 初始化数据库
                    init_db(id.clone()).await;
                    // 将用户信息存入数据库
                    // 先查询是否登录过
                    let user_repo = db::users().await;
                    // let user_former = user_repo.get(id.clone()).await;
                    user_repo.add(&user).await;
                    // if user_former.is_ok() && user_former.unwrap().id != AttrValue::default() {
                    //     // 已经存在，更新数据库
                    // } else {
                    //     user_repo.add(&user).await;
                    // }
                    LoginMsg::Success(id)
                });
                self.login_state = LoginState::Logining;
                true
            }
            LoginMsg::Success(id) => {
                ctx.link().navigator().unwrap().push(&Page::Home { id });
                true
            }
            LoginMsg::Failed => {
                self.show_error = true;
                true
            }
            LoginMsg::OnEnterKeyDown(event) => {
                event.prevent_default();
                false
            }
            LoginMsg::SwitchLanguage(event) => {
                log::debug!("switch language: {:?}", event);
                let input = event
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap();
                let value = input.value();
                if value == "zh_cn" {
                    self.i18n = utils::create_bundle(zh_cn::LOGIN);
                    // save language type with yewdux
                    Dispatch::<I18nState>::global().reduce_mut(|s| s.lang = LanguageType::ZhCN);
                    self.lang = LanguageType::ZhCN;
                } else if value == "en_us" {
                    self.i18n = utils::create_bundle(en_us::LOGIN);
                    Dispatch::<I18nState>::global().reduce_mut(|s| s.lang = LanguageType::EnUS);
                    self.lang = LanguageType::EnUS;
                }
                true
            }
            LoginMsg::SwitchTheme(state) => {
                utils::set_theme(&state.to_string());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut info = html!();
        if self.show_error {
            info = html!(
                <div class="error">
                    {"用户名或密码错误"}
                </div>)
        }

        // i18n
        let login_title = tr!(self.i18n, "login_text");
        let email = tr!(self.i18n, "email");
        let onchange = ctx.link().callback(LoginMsg::SwitchLanguage);

        let mut switch = classes!("switch", "pointer");
        let mut slider = classes!("slider");
        if *self.theme.get() == ThemeState::Dark {
            switch.push("background-change");
            slider.push("right");
        } else {
            slider.push("left");
        }

        let theme = html! {
            <span class={switch} onclick={self.theme.reduce_mut_callback(|s| {if *s == ThemeState::Dark{
                *s = ThemeState::Light;
            } else {
                *s = ThemeState::Dark;
            }})}>
                <span class={slider}></span>
            </span>
        };
        html! {
            <div class="login-container">
                {info}
                <form class="login-wrapper" onsubmit={ctx.link().callback(LoginMsg::OnEnterKeyDown)}>
                    <div class="sign">
                        {login_title.clone()}
                    </div>

                    <div class="email">
                        <input type="text" ref={self.account_ref.clone()} required={true} autocomplete="current-password"  placeholder={email}/>
                    </div>
                    <div class="pwd">
                        <input type="password" ref={self.pwd_ref.clone()} required={true} autocomplete="current-password"   placeholder={tr!(self.i18n, "password")}/>
                    </div>
                    <div class="language">
                        <label for="en_us">
                            <input type="radio" name="language" id="en_us" value="en_us" onchange={onchange.clone()} checked={self.lang==LanguageType::EnUS}/>{"\tENG"}
                        </label>
                        <label for="zh_cn">
                            <input type="radio" name="language" id="zh_cn" value="zh_cn" {onchange} checked={self.lang==LanguageType::ZhCN}/>{"\t中文"}
                        </label>
                    </div>
                    <input type="submit" class="submit" onclick={ctx.link().callback(|_| LoginMsg::Login)} value={login_title}/>
                    <div class="login-register">
                        {tr!(self.i18n, "to_register_prefix")}
                        <a href="/register">{tr!(self.i18n, "to_register")}</a>
                    </div>
                </form>
                <div class="theme-login">
                    {theme}
                </div>
            </div>
        }
    }
}
