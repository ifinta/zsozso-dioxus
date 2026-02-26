use dioxus::prelude::*;
use crate::ui::i18n::UiI18n;

pub fn render_home_tab(i18n: &dyn UiI18n) -> Element {
    rsx! {
        div { style: "text-align: center; margin-top: 60px; color: #888;",
            p { style: "font-size: 2em;", "🏠" }
            p { "{i18n.tab_home()}" }
        }
    }
}
