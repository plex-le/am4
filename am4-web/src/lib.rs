mod components;
mod console;
mod db;

use am4::user::{GameMode, Settings};
use components::aircraft::{ACDetails, ACSearch};
use components::airport::{APDetails, APSearch};
use components::console::ConsoleView;
use components::help::Help;
use components::nav::{Header, Page};
use components::settings::SettingsPanel;

use console::{ConsoleState, UserLogger};
use db::{Idb, LoadDbProgress};
use leptos::prelude::*;
use leptos::web_sys;

const SETTINGS_KEY: &str = "am4_settings";
const GAME_MODE_KEY: &str = "am4_game_mode";

#[component]
pub fn App() -> impl IntoView {
    let database = StoredValue::<Option<db::Data>>::new(None);
    let (console, set_console) = signal(ConsoleState::default());
    let logger = UserLogger(set_console);

    let (progress, set_progress) = signal(db::LoadDbProgress::Starting);
    let page = RwSignal::new(Page::Calculator);

    let initial_settings = web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .and_then(|ls| ls.get_item(SETTINGS_KEY).unwrap())
        .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
        .unwrap_or_default();
    let settings = RwSignal::new(initial_settings);

    Effect::new(move |_| {
        let s = settings.get();
        if let Some(ls) = web_sys::window().unwrap().local_storage().unwrap() {
            let _ = ls.set_item(SETTINGS_KEY, &serde_json::to_string(&s).unwrap());
        }
    });

    let initial_game_mode = web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .and_then(|ls| ls.get_item(GAME_MODE_KEY).unwrap())
        .and_then(|s| serde_json::from_str::<GameMode>(&s).ok())
        .unwrap_or_default();
    let game_mode = RwSignal::new(initial_game_mode);

    Effect::new(move |_| {
        let gm = game_mode.get();
        if let Some(ls) = web_sys::window().unwrap().local_storage().unwrap() {
            let _ = ls.set_item(GAME_MODE_KEY, &serde_json::to_string(&gm).unwrap());
        }
    });

    provide_context(database);
    provide_context(set_console);
    provide_context(console);
    provide_context(logger);
    provide_context(settings);
    provide_context(game_mode);
    provide_context(page);

    LocalResource::new(move || async move {
        logger.info(format!(
            "initialising am4help {}",
            env!("CARGO_PKG_VERSION")
        ));
        match Idb::connect()
            .await
            .unwrap()
            .init_db(|msg| logger.info(msg))
            .await
        {
            Ok(db) => {
                database.set_value(Some(db));
                set_progress.set(LoadDbProgress::Loaded);
                logger.success("initialised database");
            }
            Err(e) => {
                logger.error(format!("database error: {e}"));
                set_progress.set(LoadDbProgress::Err);
            }
        }
    });

    view! {
        <div id="app">
            <Header />
            <main>
                <Show when=move || page.get() == Page::Help>
                    <Help />
                </Show>

                <Show when=move || page.get() == Page::Calculator>
                    <Show
                        when=move || progress.get() == LoadDbProgress::Loaded
                        fallback=|| {
                            view! { <div class="padded">"Loading data..."</div> }
                        }
                    >
                        <ConsoleView />
                        <SettingsPanel />
                        <div id="search-layout">
                            <div id="input-group">
                                <ACSearch />
                                <APSearch />
                            </div>
                            <div id="details-pane">
                                <ACDetails />
                                <APDetails />
                            </div>
                        </div>
                    </Show>
                </Show>
            </main>
        </div>
    }
}
