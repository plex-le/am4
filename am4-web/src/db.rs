use am4::aircraft::db::Aircrafts;
use am4::airport::{db::Airports, Airport};
use am4::route::db::DistanceMatrix;
use am4::{AC_FILENAME, AP_FILENAME, DIST_FILENAME};
use indexed_db_futures::database::Database;
use indexed_db_futures::error::OpenDbError;
use indexed_db_futures::prelude::*;
use indexed_db_futures::transaction::TransactionMode;
use leptos::{
    wasm_bindgen::{prelude::*, JsValue},
    web_sys,
};
use thiserror::Error;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    js_sys::{Array, Uint8Array},
    window, Blob, BlobPropertyBag, Response,
};

#[derive(Debug, Clone)]
pub struct Idb {
    database: Database,
}

impl Idb {
    const NAME_DB: &str = "am4help";
    const NAME_STORE: &str = "data";

    /// connect to the database and ensure that `am4help/data` object store exists
    pub async fn connect() -> Result<Self, OpenDbError> {
        let database = Database::open(Self::NAME_DB)
            .with_on_upgrade_needed(|_event, db| {
                if !db.object_store_names().any(|n| n == Self::NAME_STORE) {
                    db.create_object_store(Self::NAME_STORE).build()?;
                }
                Ok(())
            })
            .await?;
        Ok(Self { database })
    }

    pub async fn clear(&self) -> Result<(), indexed_db_futures::error::Error> {
        let tx = self
            .database
            .transaction(Self::NAME_STORE)
            .with_mode(TransactionMode::Readwrite)
            .build()?;
        tx.object_store(Self::NAME_STORE)?.clear()?;
        tx.commit().await?;
        Ok(())
    }

    /// Load a binary file from the IndexedDb. If the blob is not found*,
    /// fetch it from the server and cache it in the IndexedDb.
    /// not found: `Response`* -> `Blob`* -> IndexedDb -> `Blob` -> `ArrayBuffer` -> `Vec<u8>`
    pub async fn get_blob<F>(&self, k: &str, url: &str, log: &F) -> Result<Vec<u8>, DatabaseError>
    where
        F: Fn(String),
    {
        {
            let tx = self
                .database
                .transaction(Self::NAME_STORE)
                .with_mode(TransactionMode::Readonly)
                .build()?;
            let store = tx.object_store(Self::NAME_STORE)?;
            if let Some(b) = store.get::<JsValue, _, _>(k).primitive()?.await? {
                log(format!("idb hit: {k}"));
                let ab = JsFuture::from(b.dyn_into::<Blob>()?.array_buffer()).await?;
                return Ok(Uint8Array::new(&ab).to_vec());
            }
        }

        log(format!("downloading: {k}"));
        let b = fetch_bytes(url).await?;

        {
            let tx = self
                .database
                .transaction(Self::NAME_STORE)
                .with_mode(TransactionMode::Readwrite)
                .build()?;
            let store = tx.object_store(Self::NAME_STORE)?;
            store.put(&b).with_key(k).await?;
            tx.commit().await?;
        }

        let ab = JsFuture::from(b.dyn_into::<Blob>()?.array_buffer()).await?;
        Ok(Uint8Array::new(&ab).to_vec())
    }

    /// Load the flat distances from the indexeddb. If the blob is not found*,
    /// generate it from the slice of airports and cache it in the indexeddb.
    /// not found: `&[Airport]`* -> `Distances`* (return this) -> `Blob`* -> IndexedDb
    /// found: IndexedDb -> `Blob` -> `ArrayBuffer` -> `Distances`
    async fn get_distances<F>(
        &self,
        aps: &[Airport],
        log: &F,
    ) -> Result<DistanceMatrix, DatabaseError>
    where
        F: Fn(String),
    {
        {
            let tx = self
                .database
                .transaction(Self::NAME_STORE)
                .with_mode(TransactionMode::Readonly)
                .build()?;
            let store = tx.object_store(Self::NAME_STORE)?;
            if let Some(b) = store
                .get::<JsValue, _, _>(DIST_FILENAME)
                .primitive()?
                .await?
            {
                log(format!("idb hit: {DIST_FILENAME}"));
                let ab = JsFuture::from(b.dyn_into::<Blob>()?.array_buffer()).await?;
                let bytes = Uint8Array::new(&ab).to_vec();
                return Ok(DistanceMatrix::from_bytes(&bytes).unwrap());
            }
        }

        log("calculating distances...".to_string());
        let distances = DistanceMatrix::from_airports(aps);
        let b = distances.to_bytes().unwrap();

        // https://github.com/rustwasm/wasm-bindgen/issues/1693
        // effectively, this is `new Blob([new Uint8Array(b)], {type: 'application/octet-stream'})`
        let ja = Array::new();
        ja.push(&Uint8Array::from(b.as_slice()).buffer());
        let opts = BlobPropertyBag::new();
        opts.set_type("application/octet-stream");
        let blob = Blob::new_with_u8_array_sequence_and_options(&ja, &opts)?;
        let val: JsValue = blob.into();

        {
            let tx = self
                .database
                .transaction(Self::NAME_STORE)
                .with_mode(TransactionMode::Readwrite)
                .build()?;
            let store = tx.object_store(Self::NAME_STORE)?;
            store.put(&val).with_key(DIST_FILENAME).await?;
            tx.commit().await?;
        }
        Ok(distances)
    }

    pub async fn init_db<F>(&self, log: F) -> Result<Data, DatabaseError>
    where
        F: Fn(String),
    {
        let bytes = self
            .get_blob(AP_FILENAME, &format!("assets/{AP_FILENAME}"), &log)
            .await?;
        let airports = Airports::from_bytes(&bytes).unwrap();
        log(format!("loaded {} airports", airports.data().len()));

        let bytes = self
            .get_blob(AC_FILENAME, &format!("assets/{AC_FILENAME}"), &log)
            .await?;
        let aircrafts = Aircrafts::from_bytes(&bytes).unwrap();
        log(format!("loaded {} aircraft", aircrafts.data().len()));

        let distances = self.get_distances(airports.data(), &log).await?;
        log(format!("loaded {} distances", distances.data().len()));

        Ok(Data {
            aircrafts,
            airports,
        })
    }
}

async fn fetch_bytes(path: &str) -> Result<JsValue, DatabaseError> {
    let window = window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_str(path)).await?;
    let resp = resp_value.dyn_into::<Response>()?;
    let jsb = JsFuture::from(resp.blob()?).await?;
    Ok(jsb)
}

pub struct Data {
    pub aircrafts: Aircrafts,
    pub airports: Airports,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadDbProgress {
    Starting,
    Loaded,
    Err,
}

#[derive(Error, Debug, PartialEq)]
pub enum DatabaseError {
    #[error("IDB error: {:?}", self)]
    Idb(#[from] indexed_db_futures::error::Error),
    #[error("DOM exception: {:?}", self)]
    Dom(web_sys::DomException),
    #[error("JavaScript exception: {:?}", self)]
    Js(JsValue),
}

impl From<web_sys::DomException> for DatabaseError {
    fn from(e: web_sys::DomException) -> Self {
        Self::Dom(e)
    }
}

impl From<JsValue> for DatabaseError {
    fn from(v: JsValue) -> Self {
        Self::Js(v)
    }
}
