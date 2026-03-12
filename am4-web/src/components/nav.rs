use crate::console::UserLogger;
use crate::db::Idb;
use am4::user::Settings;
use leptos::prelude::*;
use leptos::web_sys;

#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Calculator,
    Help,
    ResultsGrid,
}

#[component]
pub fn Header() -> impl IntoView {
    let logger = expect_context::<UserLogger>();
    let settings = expect_context::<RwSignal<Settings>>();
    let page = expect_context::<RwSignal<Page>>();

    let clear_db = Action::new_local(move |_: &()| async move {
        Idb::connect().await.unwrap().clear().await.unwrap();
        if let Some(ls) = web_sys::window().unwrap().local_storage().unwrap() {
            let _ = ls.remove_item("am4_settings");
        }
        settings.set(Settings::default());
        logger.info("cleared IndexedDB and settings");
    });

    view! {
        <header>
            <div id="global-nav">
                <a href="https://github.com/abc8747/am4" target="_blank">
                    <img src="/assets/img/icons/logo-196.png" alt="logo" height="32" width="32" />
                </a>
                <div>
                    <span id="name">"AM4Help"</span>
                </div>
            </div>
            <div id="local-bar">
                <nav>
                    <ul>
                        <li
                            class:active=move || page.get() == Page::Calculator
                            on:click=move |_| page.set(Page::Calculator)
                        >
                            "Calculator"
                        </li>
                        <li
                            class:active=move || page.get() == Page::Help
                            on:click=move |_| page.set(Page::Help)
                        >
                            "Help"
                        </li>
                        <li
                            id="reset"
                            on:click=move |_| {
                                clear_db.dispatch(());
                            }
                        >
                            "Reset"
                        </li>
                    </ul>
                </nav>
            </div>
        </header>
    }
}
