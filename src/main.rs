#[allow(dead_code)] // wired into the renderer by T30
mod emphasis;
mod export;
mod model;
mod render;
mod storage;
mod themes;
mod ui;

use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(app);
}

fn app() -> Element {
    rsx! {
        document::Title { "Bannerbay: make banners and social cards" }
        document::Stylesheet { href: MAIN_CSS }
        ui::App {}
    }
}
