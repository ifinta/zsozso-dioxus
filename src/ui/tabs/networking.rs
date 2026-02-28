use dioxus::prelude::*;
use crate::ui::state::WalletState;
use crate::ui::controller::AppController;
use crate::ui::i18n::UiI18n;

pub fn render_networking_tab(s: WalletState, ctrl: AppController, i18n: &dyn UiI18n) -> Element {
    let ping_display = match &*s.ping_status.read() {
        Some(msg) => msg.clone(),
        None => String::new(),
    };

    rsx! {
        div { style: "text-align: center; margin-top: 40px;",
            p { style: "font-size: 2em;", "\u{1F310}" }
            p { style: "color: #888; margin-bottom: 30px;", "{i18n.tab_networking()}" }

            // Ping button
            button {
                style: "padding: 14px 48px; background: #007bff; color: white; border: none; border-radius: 8px; font-weight: bold; cursor: pointer; font-size: 1.1em;",
                onclick: move |_| ctrl.ping_contract_action(),
                "{i18n.btn_ping()}"
            }

            // Ping result display
            if !ping_display.is_empty() {
                p { style: "margin-top: 20px; font-size: 0.95em; color: #495057; font-style: italic; padding: 0 20px;",
                    "{ping_display}"
                }
            }
        }
    }
}
