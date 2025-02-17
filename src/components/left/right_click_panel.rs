use fluent::{FluentBundle, FluentResource};
use web_sys::HtmlDivElement;
use yew::prelude::*;
use yew::{Component, Properties};

use crate::i18n::{en_us, zh_cn, LanguageType};
use crate::{tr, utils};
pub struct RightClickPanel {
    node: NodeRef,
    pub i18n: FluentBundle<FluentResource>,
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct RightClickPanelProps {
    pub x: i32,
    pub y: i32,
    pub close: Callback<()>,
    pub delete: Callback<()>,
    pub mute: Callback<()>,
    pub is_mute: bool,
    pub lang: LanguageType,
}

pub enum RightClickPanelMsg {}

impl Component for RightClickPanel {
    type Message = RightClickPanelMsg;

    type Properties = RightClickPanelProps;

    fn create(ctx: &Context<Self>) -> Self {
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::RIGHT_CLICK_PANEL,
            LanguageType::EnUS => en_us::RIGHT_CLICK_PANEL,
        };
        let i18n = utils::create_bundle(res);
        Self {
            node: NodeRef::default(),
            i18n,
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        let node: HtmlDivElement = self.node.cast().unwrap();
        node.focus().unwrap();
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let style = format!("left: {}px; top: {}px;", ctx.props().x, ctx.props().y);
        let mute_str = if ctx.props().is_mute {
            tr!(self.i18n, "un_mute")
        } else {
            tr!(self.i18n, "mute")
        };
        html! {
            <div ref={self.node.clone()}
                {style}
                class="right-click-panel box-shadow" tabindex="0"
                onblur={ctx.props().close.reform(|_|())}
                >
                <div class="right-click-panel-item hover" onclick={ctx.props().delete.reform(|_|())}>
                    {tr!(self.i18n, "delete")}
                </div>
                <div class="right-click-panel-item hover" onclick={ctx.props().mute.reform(|_|())}>
                    {mute_str}
                </div>
            </div>
        }
    }
}
