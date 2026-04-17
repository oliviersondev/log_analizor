use dioxus::prelude::*;

#[component]
pub fn OutputPanel(output: String, error_message: Option<String>) -> Element {
    rsx! {
        if let Some(err) = error_message {
            div { class: "error", "Erreur: {err}" }
        }

        pre { class: "output", "{output}" }
    }
}
