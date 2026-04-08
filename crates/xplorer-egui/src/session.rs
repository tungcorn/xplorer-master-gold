use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub tabs: Vec<TabSession>,
    pub show_sidebar: bool,
    #[serde(default)]
    pub show_preview: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TabSession {
    pub path: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Workspace {
    pub name: String,
    pub tabs: Vec<TabSession>,
    pub show_sidebar: bool,
    pub show_preview: bool,
}

#[derive(Serialize, Deserialize, Default)]
pub struct WorkspaceStore {
    pub workspaces: Vec<Workspace>,
}

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("xplorer")
}

fn session_path() -> PathBuf {
    config_dir().join("session.json")
}

fn workspaces_path() -> PathBuf {
    config_dir().join("workspaces.json")
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

pub fn load_workspaces() -> WorkspaceStore {
    let path = workspaces_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default()
}

pub fn save_workspaces(store: &WorkspaceStore) {
    let path = workspaces_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(store) {
        let _ = std::fs::write(&path, json);
    }
}

pub fn save_workspace(name: &str, tabs: Vec<TabSession>, show_sidebar: bool, show_preview: bool) {
    let mut store = load_workspaces();
    if let Some(existing) = store.workspaces.iter_mut().find(|w| w.name == name) {
        existing.tabs = tabs;
        existing.show_sidebar = show_sidebar;
        existing.show_preview = show_preview;
    } else {
        store.workspaces.push(Workspace {
            name: name.to_string(),
            tabs,
            show_sidebar,
            show_preview,
        });
    }
    save_workspaces(&store);
}

#[allow(dead_code)]
pub fn delete_workspace(name: &str) {
    let mut store = load_workspaces();
    store.workspaces.retain(|w| w.name != name);
    save_workspaces(&store);
}
