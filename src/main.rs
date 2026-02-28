mod db;
mod i18n;
mod ledger;
mod ui;
mod store;

use dioxus::prelude::*;

fn main() {
    LaunchBuilder::web().launch(ui::app);
}