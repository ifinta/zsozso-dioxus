mod db;
mod i18n;
mod ledger;
mod ui;
mod store;

use dioxus::prelude::*;
use std::sync::OnceLock;

/// App version string read from the service worker's CACHE_NAME at startup.
static APP_VERSION: OnceLock<String> = OnceLock::new();

pub fn app_version() -> &'static str {
    APP_VERSION.get().map(|s| s.as_str()).unwrap_or("")
}

/// Synchronously read the SW CACHE_NAME by fetching sw.js as text and
/// extracting the constant. Runs before any UI code.
fn read_sw_version() -> String {
    let result = js_sys::eval(r#"
        (function() {
            try {
                var xhr = new XMLHttpRequest();
                xhr.open('GET', 'sw.js', false);  // synchronous
                xhr.send();
                if (xhr.status === 200) {
                    var m = xhr.responseText.match(/const\s+CACHE_NAME\s*=\s*['"]([^'"]+)['"]/);
                    return m ? m[1] : '';
                }
            } catch(e) {}
            return '';
        })()
    "#);
    result
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

fn main() {
    let version = read_sw_version();
    if !version.is_empty() {
        let _ = APP_VERSION.set(version);
    }
    LaunchBuilder::web().launch(ui::app);
}