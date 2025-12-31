use crate::console::{ConsoleState, Level};
use leptos::html::Div;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsValue;
use web_sys::js_sys::Date;

#[component]
pub fn ConsoleView() -> impl IntoView {
    let state = expect_context::<ReadSignal<ConsoleState>>();
    let container_ref = NodeRef::<Div>::new();

    Effect::new(move |_| {
        let _ = state.get().history.len();
        request_animation_frame(move || {
            if let Some(div) = container_ref.get() {
                div.set_scroll_top(div.scroll_height());
            }
        });
    });

    view! {
        <div class="settings-panel">
            <details>
                <summary>"Console"</summary>
                <div id="console" node_ref=container_ref>
                    <For
                        each=move || state.get().history
                        key=|e| e.id
                        children=move |entry| {
                            let date = Date::new(&JsValue::from_f64(entry.timestamp as f64));
                            let ts_str = format!(
                                "{:02}:{:02}:{:02}.{:03}",
                                date.get_hours(),
                                date.get_minutes(),
                                date.get_seconds(),
                                date.get_milliseconds(),
                            );
                            let lvl_class = match entry.level {
                                Level::Info => "info",
                                Level::Success => "success",
                                Level::Error => "error",
                            };

                            view! {
                                <div class=format!("entry {lvl_class}")>
                                    <span class="ts">{ts_str}</span>
                                    <span class="lvl">{entry.level.to_string()}</span>
                                    <span class="msg">{entry.message}</span>
                                </div>
                            }
                        }
                    />
                </div>
            </details>
        </div>
    }
}
