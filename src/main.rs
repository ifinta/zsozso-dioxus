mod db;
mod i18n;
mod ledger;
mod ui;
mod store;

use dioxus::prelude::*;
use std::sync::OnceLock;

/// App version string extracted from the in-app log buffer.
static APP_VERSION: OnceLock<String> = OnceLock::new();

pub fn app_version() -> &'static str {
    APP_VERSION.get().map(|s| s.as_str()).unwrap_or("")
}

/// Try to read the version from the log bridge.
/// The SW log lines contain the CACHE_NAME (e.g. "zsozso-v2").
pub fn try_read_version() {
    if APP_VERSION.get().is_some() { return; }
    let ver = js_sys::eval("window.__zsozso_log ? window.__zsozso_log.version() : ''")
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    if !ver.is_empty() {
        let _ = APP_VERSION.set(ver);
    }
}

fn main() {
    LaunchBuilder::web().launch(ui::app);
}