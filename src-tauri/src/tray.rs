//! System tray: close-to-tray, unread blink, show/quit menu.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

pub struct TrayState {
    pub unread: Arc<AtomicBool>,
    pub normal_icon: Image<'static>,
}

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        // hide()/show() do not produce WebView2 focus events; tell the UI explicitly
        let _ = w.emit("window-shown", ());
    }
}

pub fn init_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "显示 / Show", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出 / Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;
    let icon = app
        .default_window_icon()
        .cloned()
        .map(|i| i.to_owned())
        .unwrap_or_else(|| Image::new_owned(vec![0u8; 4], 1, 1));

    let tray = TrayIconBuilder::with_id("ircmontis-main")
        .icon(icon.clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;

    let unread = Arc::new(AtomicBool::new(false));
    app.manage(TrayState {
        unread: unread.clone(),
        normal_icon: icon,
    });

    let tray_id = tray.id().clone();
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut blank = false;
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let state = app2.state::<TrayState>();
            let on = state.unread.load(Ordering::Relaxed);
            if let Some(t) = app2.tray_by_id(&tray_id) {
                if on {
                    blank = !blank;
                    let img = if blank {
                        Image::new_owned(vec![0u8; 32 * 32 * 4], 32, 32)
                    } else {
                        state.normal_icon.clone()
                    };
                    let _ = t.set_icon(Some(img));
                } else if blank {
                    blank = false;
                    let _ = t.set_icon(Some(state.normal_icon.clone()));
                }
            }
        }
    });
    Ok(())
}

pub fn set_labels(app: &AppHandle, show: &str, quit: &str) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id("ircmontis-main") {
        let show_item = MenuItem::with_id(app, "show", show, true, None::<&str>)?;
        let quit_item = MenuItem::with_id(app, "quit", quit, true, None::<&str>)?;
        let menu = Menu::with_items(app, &[&show_item, &quit_item])?;
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}
