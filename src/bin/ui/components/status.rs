use dioxus::prelude::*;

#[component]
pub fn StatusLine(is_loading: bool) -> Element {
    rsx! {
        if is_loading {
            span { class: "status", "Streaming en direct..." }
        }
    }
}
