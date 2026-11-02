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
const FAVICON_SVG: Asset = asset!("/assets/favicon.svg");
const FAVICON_ICO: Asset = asset!("/assets/favicon.ico");
const APPLE_TOUCH_ICON: Asset = asset!("/assets/apple-touch-icon.png");

fn main() {
    dioxus::launch(app);
}

fn app() -> Element {
    rsx! {
        document::Title { "Bannerbay: make banners and social cards" }
        document::Link { rel: "icon", href: FAVICON_SVG, r#type: "image/svg+xml" }
        document::Link { rel: "alternate icon", href: FAVICON_ICO, sizes: "32x32" }
        document::Link { rel: "apple-touch-icon", href: APPLE_TOUCH_ICON }
        document::Stylesheet { href: MAIN_CSS }
        ui::App {}
    }
}
