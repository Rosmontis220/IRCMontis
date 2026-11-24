mod background;
mod commands;
mod store;
mod tray;

use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_background_service::init_with_service(|| {
            background::KeepAlive
        }));

    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        tray::show_main(app);
    }));

    builder
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let store = store::Store::open(&data_dir.join("ircmontis.db"))?;
            app.manage(store);
            commands::spawn_bridge(app.handle());

            #[cfg(desktop)]
            tray::init_tray(app.handle())?;

            #[cfg(desktop)]
            let config = app.config().app.windows[0].clone();
            #[cfg(desktop)]
            let window =
                tauri::WebviewWindowBuilder::from_config(app.handle(), &config)?.build()?;

            #[cfg(desktop)]
            {
                let handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let store = handle.state::<store::Store>();
                        let to_tray = store
                            .setting_get("close_to_tray")
                            .ok()
                            .flatten()
                            .map(|v| v != "false")
                            .unwrap_or(true);
                        if to_tray {
                            if let Some(w) = handle.get_webview_window("main") {
                                let _ = w.hide();
                                let _ = w.emit("window-hidden", ());
                            }
                        } else {
                            handle.exit(0);
                        }
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::irc_connect,
            commands::irc_disconnect,
            commands::irc_send,
            commands::irc_join,
            commands::irc_part,
            commands::irc_set_nick,
            commands::message_append,
            commands::history_load,
            commands::history_search,
            commands::sessions_list,
            commands::session_set_muted,
            commands::session_delete,
            commands::setting_get,
            commands::setting_set,
            commands::tray_set_unread,
            commands::tray_set_labels,
            commands::save_background
        ])
        .run(tauri::generate_context!())
        .expect("error while running IRCMontis");
}
