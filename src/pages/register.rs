use fluent::{FluentBundle, FluentResource};
use gloo::timers::callback::{Interval, Timeout};
use regex::Regex;
use wasm_bindgen::JsValue;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::prelude::RouterScopeExt;
use yewdux::Dispatch;
use zxcvbn::zxcvbn;

use crate::i18n::{en_us, zh_cn, LanguageType};
use crate::model::user::UserRegister;
use crate::pages::Page;
use crate::state::I18nState;
use crate::{api, tr, utils};

#[derive(Default)]
pub struct Register {
    name_node: NodeRef,
    email_node: NodeRef,
    pwd_node: NodeRef,
    code_node: NodeRef,
    re_pwd_is_modify: bool,
    pwd_is_same: bool,
    /// 邮箱格式状态
    email_format: bool,
    /// 邮箱是否修改
    email_is_modify: bool,
    /// 密码强度
    pwd_strength: u8,
    /// 验证码是否发送
    is_code_send: bool,
    /// 验证码倒计时
    code_timer: Option<Interval>,
    time: u8,
    req_status: RequestStatus,
    avatars: Vec<AttrValue>,
    avatar: AttrValue,
    pwd: AttrValue,
    i18n: FluentBundle<FluentResource>,
}

pub enum RegisterMsg {
    Register,
    // OnEnterKeyDown(SubmitEvent),
    // OnFormKeyDown(KeyboardEvent),
    OnEmailChange,
    SendCode,
    SendCodeSuccess,
    SendCodeFailed(JsValue),
    UpdateTime,
    Request(RequestStatus),
    OnAvatarClick(usize),
    OnPwdInput(InputEvent),
    OnRePwdInput(InputEvent),
}

#[derive(Default, Debug)]
pub enum RequestStatus {
    #[default]
    Default,
    Pendding,
    Success,
    Failed,
}

impl Component for Register {
    type Message = RegisterMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let avatars = vec![
            AttrValue::from("./images/avatars/avatar1.png"),
            AttrValue::from("./images/avatars/avatar2.png"),
            AttrValue::from("./images/avatars/avatar3.png"),
            // "./images/avatars/avatar4.png".into(),
            // "./images/avatars/avatar5.png".into(),
            // "./images/avatars/avatar6.png".into(),
        ];
        let avatar = avatars[0].clone();

        // load the i18n bundle
        let lang = Dispatch::<I18nState>::global().get().lang;
        let res = match lang {
            LanguageType::ZhCN => zh_cn::REGISTER,
            LanguageType::EnUS => en_us::REGISTER,
        };
        let i18n = utils::create_bundle(res);
        Self {
            i18n,
            avatars,
            avatar,
            ..Default::default()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            RegisterMsg::Register => {
                log::debug!("register1");
                if !self.pwd_is_same {
                    log::debug!("register2");
                    return false;
                }
                let name: HtmlInputElement = self.name_node.cast().unwrap();
                if name.value().is_empty() {
                    return false;
                }
                let email: HtmlInputElement = self.email_node.cast().unwrap();
                let pwd: HtmlInputElement = self.pwd_node.cast().unwrap();
                if email.value().is_empty() || pwd.value().is_empty() {
                    log::debug!("register3");

                    return false;
                }

                let code: HtmlInputElement = self.code_node.cast().unwrap();
                log::debug!("register4");
                let ctx = ctx.link().clone();
                let register = UserRegister {
                    name: name.value().into(),
                    password: pwd.value().into(),
                    email: email.value().into(),
                    avatar: self.avatar.clone(),
                    code: code.value().into(),
                };
                spawn_local(async move {
                    log::debug!("register5");
                    ctx.send_message(RegisterMsg::Request(RequestStatus::Pendding));
                    // 注册请求
                    match api::users().register(register).await {
                        Ok(_) => ctx.send_message(RegisterMsg::Request(RequestStatus::Success)),
                        Err(_) => ctx.send_message(RegisterMsg::Request(RequestStatus::Failed)),
                    }
                });
                true
            }
            /* RegisterMsg::OnEnterKeyDown(event) => {
                log::debug!("on submit");
                event.prevent_default();
                false
            } */
            RegisterMsg::OnEmailChange => {
                log::debug!("on email change");
                self.email_is_modify = true;
                let email: HtmlInputElement = self.email_node.cast().unwrap();
                let email_value = email.value();
                let regex =
                    Regex::new(r"^([a-zA-Z0-9_\-\.]+)@([a-zA-Z0-9_\-\.]+)\.([a-zA-Z]{2,})$")
                        .unwrap();
                self.email_format = regex.is_match(&email_value);
                true
            }
            RegisterMsg::SendCode => {
                log::debug!("send code");
                // 获取邮件
                let email: HtmlInputElement = self.email_node.cast().unwrap();
                if !self.email_format {
                    return false;
                }

                ctx.link().send_future(async move {
                    // 发送邮件
                    match api::users().send_mail(email.value()).await {
                        Ok(_) => RegisterMsg::SendCodeSuccess,
                        Err(e) => RegisterMsg::SendCodeFailed(e),
                    }
                });
                false
            }
            RegisterMsg::SendCodeSuccess => {
                log::debug!("send code success");
                // 初始化计时器
                let ctx = ctx.link().clone();
                self.code_timer = Some(Interval::new(1000, move || {
                    ctx.send_message(RegisterMsg::UpdateTime);
                }));
                self.time = 60;
                self.is_code_send = true;
                true
            }
            RegisterMsg::SendCodeFailed(e) => {
                log::error!("send code failed: {:?}", e);
                false
            }
            RegisterMsg::UpdateTime => {
                log::debug!("update time");
                self.time -= 1;
                if self.time == 0 {
                    self.code_timer.take().unwrap().cancel();
                    self.code_timer = None;
                    self.is_code_send = false;
                }
                true
            }
            RegisterMsg::Request(status) => {
                log::debug!("request: {:?}", status);
                match status {
                    RequestStatus::Pendding => self.req_status = RequestStatus::Pendding,
                    RequestStatus::Success => {
                        self.req_status = RequestStatus::Success;
                        let ctx = ctx.link().clone();
                        let timer =
                            Timeout::new(2000, move || ctx.navigator().unwrap().push(&Page::Login));
                        timer.forget();
                    }
                    RequestStatus::Failed => self.req_status = RequestStatus::Failed,
                    RequestStatus::Default => self.req_status = RequestStatus::Default,
                }
                true
            }
            RegisterMsg::OnAvatarClick(index) => {
                log::debug!("index: {}", index);
                self.avatar = self.avatars[index].clone();
                true
            }
            RegisterMsg::OnPwdInput(event) => {
                log::debug!("pwd: {}", &self.pwd);
                self.pwd = event
                    .target_dyn_into::<HtmlInputElement>()
                    .unwrap()
                    .value()
                    .into();
                if !self.pwd.is_empty() {
                    let estimate = zxcvbn(&self.pwd.clone(), &[]).unwrap();
                    self.pwd_strength = estimate.score() * 25;
                    log::debug!("strength: {}", estimate.score());
                } else {
                    self.pwd_strength = 0;
                }
                true
            }
            RegisterMsg::OnRePwdInput(event) => {
                log::debug!("re pwd");
                let re_pwd = event.target_dyn_into::<HtmlInputElement>().unwrap().value();
                self.re_pwd_is_modify = true;
                self.pwd_is_same = re_pwd == self.pwd;
                true
            } /*  RegisterMsg::OnFormKeyDown(event) => {
                  if event.key() == "Enter" {
                      event.prevent_default();
                  }
                  false
              } */
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let avatars = self.avatars.iter().enumerate().map(|(index,avatar)| {
        let on_avatar_click = ctx.link().callback(move |_| RegisterMsg::OnAvatarClick(index));
            let mut classes = classes!("register-avatar");
            if avatar == &self.avatar {
               classes.push( "avatar-active");
            }
            html! {
                <img src={avatar.to_owned()} class={classes} alt="avatar" onclick={on_avatar_click} />
            }
        }).collect::<Html>();

        let onsubmit = ctx.link().callback(|_| RegisterMsg::Register);

        let email_class = if self.email_is_modify {
            if self.email_format {
                "email-right"
            } else {
                "email-wrong"
            }
        } else {
            ""
        };
        let code_button = if self.is_code_send {
            format!("{}s {}", self.time, tr!(self.i18n, "re_send_code"))
        } else {
            tr!(self.i18n, "send_code")
        };
        let req_status = match self.req_status {
            RequestStatus::Default => html!(),
            RequestStatus::Pendding => html! {
                <div class="register-info box-shadow">
                    {tr!(self.i18n, "registering")}
                </div>
            },
            RequestStatus::Success => html!(
                <div class="register-success box-shadow">
                    {tr!(self.i18n, "register_success")}
                </div>
            ),
            RequestStatus::Failed => html! {
                <div class="register-error box-shadow">
                    {tr!(self.i18n, "register_failed")}
                </div>
            },
        };
        let pwd_strength = if self.pwd_strength > 0 {
            html! {
                <meter
                    max="100"
                    low="33"
                    high="66"
                    optimum="75"
                    value={self.pwd_strength.to_string()}>
                </meter>
            }
        } else {
            html!()
        };
        let pwd_is_same = if self.re_pwd_is_modify {
            if self.pwd_is_same {
                html!(<span style="color: green;">{"√"}</span>)
            } else {
                html!(<span style="color: red;">{"×"}</span>)
            }
        } else {
            html!()
        };
        html! {
            <div class="register-container">
                {req_status}
                <div class="register-wrapper"
                    // onkeydown={ctx.link().callback(RegisterMsg::OnFormKeyDown)}
                    // onsubmit={ctx.link().callback(RegisterMsg::OnEnterKeyDown)}>
                    >
                    <div >
                        <span>
                            {tr!(self.i18n, "avatar")}
                        </span>
                        <div class="register-avatar-wrapper">
                            {avatars}
                        </div>
                    </div>
                    <div class="nickname">
                        <label for="nickname">
                            {tr!(self.i18n, "nickname")}
                        </label>
                        <input
                            ref={self.name_node.clone()}
                            type="text"
                            id="nickname"
                            placeholder="nickname"
                            required={true}
                            autocomplete="nickname"
                            />
                    </div>
                    <div class="pwd">
                        <label for="pwd">
                            {tr!(self.i18n, "password")}
                        </label>
                        <input
                            ref={self.pwd_node.clone()}
                            type="password"
                            id="pwd"
                            required={true}
                            autocomplete="password"
                            placeholder={tr!(self.i18n, "pwd_hint")}
                            value={self.pwd.clone()}
                            oninput={ctx.link().callback(RegisterMsg::OnPwdInput)}
                            />
                        {pwd_strength}
                    </div>
                    <div class="re-pwd">
                        <label for="re-pwd">
                            {tr!(self.i18n, "confirm_pwd")}
                        </label>
                        <input
                            type="password"
                            id="re-pwd"
                            required={true}
                            autocomplete="current-password"
                            placeholder={tr!(self.i18n, "confirm_pwd_hint")}
                            oninput={ctx.link().callback(RegisterMsg::OnRePwdInput)}
                            />
                        {pwd_is_same}
                    </div>
                    <div class="email">
                        <label for="account">
                            {tr!(self.i18n, "email")}
                        </label>
                        <input ref={self.email_node.clone()}
                            class={email_class}
                            type="text"
                            id="email"
                            name="email"
                            placeholder={tr!(self.i18n, "email_hint")}
                            required={true}
                            autocomplete="current-password"
                            onchange={ctx.link().callback(|_|RegisterMsg::OnEmailChange)} />
                        <button
                            class="register-code-btn"
                            disabled={self.time != 0}
                            onclick={ctx.link().callback(|_| RegisterMsg::SendCode)}
                            >
                            {code_button}
                        </button>
                    </div>
                    <div class="code">
                        <label for="code">
                            {tr!(self.i18n, "code")}
                        </label>
                        <input
                            ref={self.code_node.clone()}
                            type="text"
                            id="code"
                            required={true}
                            autocomplete="current-password"
                            placeholder={tr!(self.i18n, "code")}/>
                    </div>

                    <div>
                        <span>
                        </span>
                        <input type="submit" class="register-button" onclick={onsubmit} value={tr!(self.i18n, "submit")}/>
                    </div>
                </div>
                    <div class="login-register">
                        {tr!(self.i18n, "to_login_prefix")}
                        <a href="/">{tr!(self.i18n, "to_login")}</a>
                    </div>
            </div>
        }
    }
}
