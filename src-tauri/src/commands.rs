use crate::clipboard::AppState;
use crate::models::Clip;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn get_clips(state: State<'_, Arc<AppState>>) -> Vec<Clip> {
    state.clips.lock().unwrap().clone()
}

#[tauri::command]
pub fn delete_clip(state: State<'_, Arc<AppState>>, id: String) {
    state.clips.lock().unwrap().retain(|c| c.id != id);
}

#[tauri::command]
pub fn clear_clips(state: State<'_, Arc<AppState>>) {
    state.clips.lock().unwrap().clear();
}

#[tauri::command]
pub fn set_capture_enabled(state: State<'_, Arc<AppState>>, enabled: bool) {
    state.capture_enabled.store(enabled, Ordering::Relaxed);
}

#[tauri::command]
pub fn get_capture_enabled(state: State<'_, Arc<AppState>>) -> bool {
    state.capture_enabled.load(Ordering::Relaxed)
}

#[tauri::command]
pub fn set_workspace(state: State<'_, Arc<AppState>>, name: String) {
    *state.workspace.lock().unwrap() = name;
}

#[tauri::command]
pub fn get_workspace(state: State<'_, Arc<AppState>>) -> String {
    state.workspace.lock().unwrap().clone()
}

/// Write text back to the OS clipboard (used by Power Paste).
#[tauri::command]
pub fn write_clipboard(text: String) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(text).map_err(|e| e.to_string())
}

/// Apply a Power-Paste transform to text before pasting.
#[tauri::command]
pub fn transform_text(text: String, op: String) -> Result<String, String> {
    let out = match op.as_str() {
        "uppercase" => text.to_uppercase(),
        "lowercase" => text.to_lowercase(),
        "trim" => text.trim().to_string(),
        "url_encode" => url_encode(&text),
        "json_pretty" => {
            let value: serde_json::Value =
                serde_json::from_str(text.trim()).map_err(|e| format!("Invalid JSON: {e}"))?;
            serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?
        }
        other => return Err(format!("Unknown transform: {other}")),
    };
    Ok(out)
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}
