mod components;
mod console;
mod db;

use components::aircraft::ACSearch;
use components::airport::APSearch;
use components::nav::Header;

use console::{Entry, Level};
use db::{Idb, LoadDbProgress};
use leptos::{logging::log, prelude::*};

#[component]
#[allow(non_snake_case)]
pub fn App() -> impl IntoView {
    let database = StoredValue::<Option<db::Data>>::new(None);
    let console = RwSignal::new(console::Console { history: vec![] });
    let (progress, set_progress) = signal(db::LoadDbProgress::Starting);

    provide_context(database);
    provide_context(console);

    LocalResource::new(move || async move {
        console.update(|c| {
            c.history.push(Entry {
                time: 0,
                level: Level::Debug,
                user: "system".to_string(),
                message: "start".to_string(),
            })
        });
        let history = console
            .get()
            .history
            .iter()
            .map(|m| m.message.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        log!("{history}");
        // let db = Idb::connect().await;
        match Idb::connect().await.unwrap().init_db().await {
            Ok(db) => {
                database.set_value(Some(db));
                set_progress.set(LoadDbProgress::Loaded);
            }
            Err(e) => {
                log!("{e}");
                set_progress.set(LoadDbProgress::Err);
            }
        }
    });

    view! {
        <div id="app">
            <Header />
            <Show when=move || progress.get() == db::LoadDbProgress::Loaded>
                <div id="search-container">
                    <ACSearch />
                    <APSearch />
                </div>
            </Show>
            <main></main>
        </div>
    }
}
