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
        div { style: "text-align: center; margin-top: 40px; padding: 0 20px;",
            // Ping button
            button {
                style: "width: 100%; padding: 14px 0; background: #007bff; color: white; border: none; border-radius: 8px; font-weight: bold; cursor: pointer; font-size: 1.1em;",
                onclick: move |_| ctrl.ping_contract_action(),
                "{i18n.btn_ping()}"
            }

            // Scan QR button
            button {
                style: "width: 100%; margin-top: 16px; padding: 14px 0; background: #28a745; color: white; border: none; border-radius: 8px; font-weight: bold; cursor: pointer; font-size: 1.1em;",
                onclick: move |_| ctrl.scan_qr_action(),
                "{i18n.btn_scan_qr()}"
            }

            // Status display
            if !ping_display.is_empty() {
                p { style: "margin-top: 20px; font-size: 0.95em; color: #495057; font-style: italic; padding: 0;",
                    "{ping_display}"
                }
            }
        }
    }
}
