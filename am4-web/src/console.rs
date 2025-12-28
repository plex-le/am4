use leptos::prelude::*;
use web_sys::js_sys::Date;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Level {
    #[default]
    Debug,
    Info,
    Success,
    Error,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Success => write!(f, "OK"),
            Self::Error => write!(f, "ERR"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub id: usize,
    pub timestamp: u64,
    pub level: Level,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct ConsoleState {
    pub history: Vec<Entry>,
}

#[derive(Clone, Copy)]
pub struct UserLogger(pub WriteSignal<ConsoleState>);

impl UserLogger {
    pub fn log(&self, level: Level, msg: impl Into<String>) {
        self.0.update(|s| {
            let entry = Entry {
                id: s.history.len(),
                timestamp: Date::now() as u64,
                level,
                message: msg.into(),
            };
            s.history.push(entry);
        });
    }

    pub fn debug(&self, msg: impl Into<String>) {
        self.log(Level::Debug, msg);
    }
    pub fn info(&self, msg: impl Into<String>) {
        self.log(Level::Info, msg);
    }
    pub fn success(&self, msg: impl Into<String>) {
        self.log(Level::Success, msg);
    }
    pub fn error(&self, msg: impl Into<String>) {
        self.log(Level::Error, msg);
    }
}
