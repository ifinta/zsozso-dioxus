mod clipboard;
pub mod actions;
pub mod i18n;
pub mod status;
pub mod tabs;
pub mod view;
pub mod state;
pub mod controller;
pub mod qr_scanner;
pub mod toast;

use dioxus::prelude::*;
use state::use_wallet_state;
use controller::AppController;
use toast::UpdateNotification;

pub fn app() -> Element {
    let state = use_wallet_state();
    let ctrl = AppController::new(state);

    // Clear clipboard when the tab/browser is closed
    use_hook(|| {
        clipboard::register_beforeunload_cleanup();
    });

    rsx! {
        {view::render_app(state, ctrl)}
        UpdateNotification {}
    }
}

pub fn log(msg: &str) { web_sys::console::log_1(&msg.into()); }
