mod clipboard;
mod commands;
mod detect;
mod models;

use std::sync::atomic::Ordering;
use std::sync::Arc;

use clipboard::{start_watcher, AppState};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// Show a window if hidden, hide it if visible.
fn toggle_window(app: &AppHandle, label: &str) {
    if let Some(win) = app.get_webview_window(label) {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
        } else {
            let _ = win.show();
            let _ = win.set_focus();
        }
    }
}

fn show_window(app: &AppHandle, label: &str) {
    if let Some(win) = app.get_webview_window(label) {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Ctrl+Shift+V opens the Power Paste overlay (Win+V is reserved by Windows).
    let overlay_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed && shortcut == &overlay_shortcut {
                        toggle_window(app, "overlay");
                    }
                })
                .build(),
        )
        .manage(Arc::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            commands::get_clips,
            commands::delete_clip,
            commands::clear_clips,
            commands::set_capture_enabled,
            commands::get_capture_enabled,
            commands::set_workspace,
            commands::get_workspace,
            commands::write_clipboard,
            commands::transform_text,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();

            // Register the global overlay shortcut.
            if let Err(e) = app.global_shortcut().register(overlay_shortcut) {
                eprintln!("[shortcut] failed to register overlay shortcut: {e}");
            }

            // System tray with a small menu.
            let show_i = MenuItem::with_id(app, "show", "Show Clipboard", true, None::<&str>)?;
            let capture_i =
                MenuItem::with_id(app, "capture", "Pause / Resume Capture", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &capture_i, &quit_i])?;

            TrayIconBuilder::with_id("main-tray")
                .tooltip("Smart Clipboard")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_window(app, "main"),
                    "capture" => {
                        let state = app.state::<Arc<AppState>>();
                        let now = !state.capture_enabled.load(Ordering::Relaxed);
                        state.capture_enabled.store(now, Ordering::Relaxed);
                    }
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
                        toggle_window(tray.app_handle(), "main");
                    }
                })
                .build(app)?;

            start_watcher(handle);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
