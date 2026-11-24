//! Tauri commands: IRC control, history, sessions, settings, tray, background image.

use ircmontis_irc::client::{self, Command, ConnectConfig, Event, Handle};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::store::{SessionRow, Store, StoredMessage};
use crate::tray::TrayState;

pub struct IrcState(pub Handle);

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn spawn_bridge(app: &AppHandle) {
    // setup runs on the main thread; enter the global runtime so client::spawn's tokio::spawn has a reactor
    let _guard = tauri::async_runtime::handle().inner().enter();
    let (handle, mut ev_rx) = client::spawn();
    app.manage(IrcState(handle));
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(ev) = ev_rx.recv().await {
            let store = app2.state::<Store>();
            match &ev {
                Event::Message {
                    session,
                    sender,
                    text,
                    kind,
                    ts,
                    msgid,
                } => {
                    let _ =
                        store.append_message(session, sender, text, kind, *ts, msgid.as_deref());
                    let session_kind = if session.starts_with('#') || session.starts_with('&') {
                        "channel"
                    } else {
                        "pm"
                    };
                    let _ = store.touch_session(session, session_kind, *ts);
                }
                Event::Joined { channel, .. } => {
                    let _ = store.touch_session(channel, "channel", now_millis());
                }
                _ => {}
            }
            let _ = app2.emit("irc-event", &ev);
        }
    });
}

#[tauri::command]
pub fn irc_connect(state: State<IrcState>, config: ConnectConfig) {
    state.0.send(Command::Connect(config));
}

#[tauri::command]
pub fn irc_disconnect(state: State<IrcState>) {
    state.0.send(Command::Disconnect);
}

#[tauri::command]
pub fn irc_send(state: State<IrcState>, target: String, text: String) {
    state.0.send(Command::Send { target, text });
}

#[tauri::command]
pub fn irc_join(state: State<IrcState>, channel: String) {
    state.0.send(Command::Join(channel));
}

#[tauri::command]
pub fn irc_part(state: State<IrcState>, channel: String, reason: Option<String>) {
    state.0.send(Command::Part { channel, reason });
}

#[tauri::command]
pub fn irc_set_nick(state: State<IrcState>, nick: String) {
    state.0.send(Command::SetNick(nick));
}

#[tauri::command]
pub fn message_append(
    store: State<Store>,
    session: String,
    sender: String,
    text: String,
    kind: String,
    ts: i64,
) -> Result<i64, String> {
    store
        .append_message(&session, &sender, &text, &kind, ts, None)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn history_load(
    store: State<Store>,
    session: String,
    before_ts: Option<i64>,
    limit: Option<u32>,
) -> Result<Vec<StoredMessage>, String> {
    store
        .history(&session, before_ts, limit.unwrap_or(100))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn history_search(
    store: State<Store>,
    session: String,
    query: String,
    limit: Option<u32>,
) -> Result<Vec<StoredMessage>, String> {
    store
        .search(&session, &query, limit.unwrap_or(200))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn sessions_list(store: State<Store>) -> Result<Vec<SessionRow>, String> {
    store.sessions().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn session_set_muted(store: State<Store>, name: String, muted: bool) -> Result<(), String> {
    store.set_muted(&name, muted).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn session_delete(store: State<Store>, name: String) -> Result<(), String> {
    store.delete_session(&name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn setting_get(store: State<Store>, key: String) -> Result<Option<String>, String> {
    store.setting_get(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn setting_set(store: State<Store>, key: String, value: String) -> Result<(), String> {
    store.setting_set(&key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn tray_set_unread(state: State<TrayState>, on: bool) {
    state.unread.store(on, std::sync::atomic::Ordering::Relaxed);
}

#[tauri::command]
pub fn tray_set_labels(app: AppHandle, show: String, quit: String) -> Result<(), String> {
    crate::tray::set_labels(&app, &show, &quit).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_background(app: AppHandle, path: String) -> Result<String, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("backgrounds");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    let dest = dir.join(format!("current.{ext}"));
    std::fs::copy(&path, &dest).map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().to_string())
}
