use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub tabs: Vec<TabSession>,
    pub show_sidebar: bool,
    #[serde(default)]
    pub show_preview: bool,
}

#[derive(Serialize, Deserialize)]
pub struct TabSession {
    pub path: String,
}

fn session_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("xplorer")
        .join("session.json")
}

pub fn save(session: &Session) {
    let path = session_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(session) {
        let _ = std::fs::write(&path, json);
    }
}

pub fn load() -> Option<Session> {
    let path = session_path();
    let data = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}
