//! A collapsible section: a header button that opens and closes its body with an animated height.
//! The body stays in the page (so it can animate) but is `inert` while closed, so it is skipped by
//! Tab and screen readers. Reduced motion makes the change instant (tokens.css).
use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::prelude::*;

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

#[component]
pub fn Collapsible(
    title: String,
    #[props(default)] open: bool,
    #[props(default)] on_reset: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let mut is_open = use_signal(|| open);
    let id = use_hook(|| format!("collapse-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed)));
    let expanded = is_open();
    rsx! {
        div { class: if expanded { "collapsible open" } else { "collapsible" },
            button {
                class: "collapsible-head",
                r#type: "button",
                aria_expanded: if expanded { "true" } else { "false" },
                aria_controls: "{id}",
                onclick: move |_| is_open.toggle(),
                span { class: "chevron", "aria-hidden": "true" }
                "{title}"
            }
            // Only present while closed: Dioxus would write `inert="false"`, which is still inert.
            div { class: "collapsible-body", id: "{id}", inert: (!expanded).then_some("true"),
                div { class: "collapsible-inner",
                    {children}
                    if let Some(on_reset) = on_reset {
                        button { class: "reset", onclick: move |_| on_reset.call(()), "Reset to default" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(open: bool) -> String {
        let mut dom = VirtualDom::new_with_props(
            |p: (bool,)| rsx! { Collapsible { title: "Style", open: p.0, p { "inside" } } },
            (open,),
        );
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn closed_section_is_collapsed_and_inert() {
        let html = render(false);
        assert!(html.contains(r#"aria-expanded="false""#), "{html}");
        assert!(html.contains("inert"), "{html}");
        assert!(html.contains("inside"), "the body stays in the page so it can animate: {html}");
    }

    /// Dioxus 0.7 does not treat `inert` as a boolean attribute in the browser, so `inert: false`
    /// becomes `inert="false"`, which still makes the section inert. The attribute must only be
    /// added, as a string, while the section is closed.
    #[test]
    fn inert_is_set_as_a_string_only_while_closed() {
        assert!(render(false).contains(r#"inert="true""#), "{}", render(false));
    }

    #[test]
    fn open_section_is_expanded_and_reachable() {
        let html = render(true);
        assert!(html.contains(r#"aria-expanded="true""#), "{html}");
        assert!(!html.contains("inert"), "{html}");
    }

    #[test]
    fn header_controls_the_body() {
        let html = render(false);
        let id = html.split(r#"aria-controls=""#).nth(1).and_then(|s| s.split('"').next()).expect("aria-controls");
        assert!(html.contains(&format!(r#"id="{id}""#)), "{html}");
    }
}
