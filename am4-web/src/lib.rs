mod components;
mod console;
mod db;

use am4::aircraft::custom::CustomAircraft;
use am4::airport::Airport;
use am4::user::{GameMode, Settings};
use components::aircraft::{ACDetails, ACSearch, ACSelection};
use components::airport::{APDetails, APSearch};
use components::console::ConsoleView;
use components::help::Help;
use components::icons::DownloadIcon;
use components::nav::{Header, Page};
use components::route::{RouteList, RouteOptions, RouteStats, WebScheduledRoute};
use components::settings::SettingsPanel;

use console::{ConsoleState, UserLogger};
use db::{Idb, LoadDbProgress};
use leptos::prelude::*;
use leptos::web_sys;

const SETTINGS_KEY: &str = "am4_settings";
const GAME_MODE_KEY: &str = "am4_game_mode";

#[derive(Clone, Copy, PartialEq)]
pub enum DemandsState {
    Unknown,
    Checking,
    Present,
    Missing,
}

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

    let ac_selected = RwSignal::new(Vec::<CustomAircraft>::new());
    let ap_selected = RwSignal::new(Vec::<Airport>::new());
    let ac_active = RwSignal::new(None::<ACSelection>);
    let ap_active = RwSignal::new(None::<Airport>);

    let demands_state = RwSignal::new(DemandsState::Unknown);
    let loading_demands = RwSignal::new(false);

    let (routes, set_routes) = signal(Vec::<WebScheduledRoute>::new());
    let (stats, set_stats) = signal(RouteStats::default());
    let show_origin = Memo::new(move |_| ap_selected.get().len() > 1);

    let load_demands = Action::new_local(move |_: &()| async move {
        loading_demands.set(true);
        match Idb::connect()
            .await
            .unwrap()
            .load_demands(|msg| logger.info(msg))
            .await
        {
            Ok(demands) => {
                database.update_value(|db| {
                    if let Some(data) = db.as_mut() {
                        data.demands = Some(demands);
                    }
                });
                demands_state.set(DemandsState::Present);
                logger.success("demands loaded");
            }
            Err(e) => {
                logger.error(format!("failed to load demands: {e}"));
            }
        }
        loading_demands.set(false);
    });

    provide_context(database);
    provide_context(set_console);
    provide_context(console);
    provide_context(logger);
    provide_context(settings);
    provide_context(game_mode);
    provide_context(page);
    provide_context(ac_active);
    provide_context(ap_active);

    LocalResource::new(move || async move {
        logger.info(format!(
            "initialising am4help {}",
            env!("CARGO_PKG_VERSION")
        ));
        let idb = Idb::connect().await.unwrap();
        demands_state.set(DemandsState::Checking);
        let has_demands = idb.has_demands().await.unwrap_or(false);

        match idb.init_db(|msg| logger.info(msg)).await {
            Ok(db) => {
                database.set_value(Some(db));
                if has_demands {
                    load_demands.dispatch(());
                } else {
                    demands_state.set(DemandsState::Missing);
                }
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
                    <ConsoleView />
                    <SettingsPanel />
                    <Show
                        when=move || progress.get() == LoadDbProgress::Loaded
                        fallback=|| {
                            view! { <div class="padded">"Loading data..."</div> }
                        }
                    >
                        <div id="search-layout">
                            <div id="input-group">
                                <ACSearch selected=ac_selected active=ac_active />
                                <APSearch selected=ap_selected active=ap_active />

                                <Show
                                    when=move || demands_state.get() == DemandsState::Present
                                    fallback=move || {
                                        view! {
                                            <Show when=move || {
                                                demands_state.get() == DemandsState::Missing
                                            }>
                                                <div class="demand-prompt">
                                                    <p>
                                                        "am4help needs to download a 46 MB database for offline route searching."
                                                    </p>
                                                    <button
                                                        class="download-btn"
                                                        disabled=move || loading_demands.get()
                                                        on:click=move |_| {
                                                            load_demands.dispatch(());
                                                        }
                                                    >
                                                        <DownloadIcon />
                                                        {move || {
                                                            if loading_demands.get() {
                                                                "Downloading..."
                                                            } else {
                                                                "Start Download"
                                                            }
                                                        }}

                                                    </button>
                                                </div>
                                            </Show>
                                        }
                                    }
                                >
                                    <RouteOptions
                                        ac_selected=ac_selected
                                        ap_selected=ap_selected
                                        set_routes=set_routes
                                        set_stats=set_stats
                                    />
                                </Show>
                            </div>
                            <div id="details-pane">
                                <ACDetails />
                                <APDetails />
                                <RouteList routes=routes stats=stats show_origin=show_origin />
                            </div>
                        </div>
                    </Show>
                </Show>
            </main>
        </div>
    }
}
