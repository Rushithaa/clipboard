use crate::detect::{detect_kind, is_sensitive, make_preview};
use crate::models::{Clip, ClipType};
use base64::Engine;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const MAX_CLIPS: usize = 500;
const POLL_INTERVAL_MS: u64 = 700;

/// Shared application state, managed by Tauri and shared with the watcher thread.
pub struct AppState {
    pub clips: Mutex<Vec<Clip>>,
    pub capture_enabled: AtomicBool,
    pub workspace: Mutex<String>,
    last_hash: Mutex<Option<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            clips: Mutex::new(Vec::new()),
            capture_enabled: AtomicBool::new(true),
            workspace: Mutex::new("default".to_string()),
            last_hash: Mutex::new(None),
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn now_id(hash: &str) -> String {
    format!("{}-{}", Utc::now().timestamp_millis(), &hash[..hash.len().min(8)])
}

/// Spawn the background clipboard monitor. Polls the OS clipboard and emits a
/// `clip-added` event whenever new content appears.
pub fn start_watcher(app: AppHandle) {
    std::thread::spawn(move || {
        let mut clipboard = match arboard::Clipboard::new() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[clipboard] failed to open clipboard: {e}");
                return;
            }
        };

        loop {
            std::thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));

            let state = app.state::<std::sync::Arc<AppState>>();
            if !state.capture_enabled.load(Ordering::Relaxed) {
                continue;
            }

            if let Ok(text) = clipboard.get_text() {
                if !text.is_empty() {
                    let hash = sha256_hex(text.as_bytes());
                    if !is_duplicate(&state, &hash) {
                        let (kind, meta) = detect_kind(&text);
                        let clip = build_text_clip(&state, text, kind, meta, hash);
                        push_clip(&app, &state, clip);
                    }
                    continue;
                }
            }

            if let Ok(img) = clipboard.get_image() {
                let raw = img.bytes.as_ref();
                let hash = sha256_hex(raw);
                if !is_duplicate(&state, &hash) {
                    if let Some(clip) = build_image_clip(&state, &img, hash) {
                        push_clip(&app, &state, clip);
                    }
                }
            }
        }
    });
}

fn is_duplicate(state: &AppState, hash: &str) -> bool {
    let mut last = state.last_hash.lock().unwrap();
    if last.as_deref() == Some(hash) {
        return true;
    }
    *last = Some(hash.to_string());
    false
}

fn build_text_clip(
    state: &AppState,
    text: String,
    kind: ClipType,
    meta: serde_json::Value,
    hash: String,
) -> Clip {
    let sensitive = is_sensitive(&text);
    let preview = make_preview(&text, 120);
    Clip {
        id: now_id(&hash),
        kind,
        content: text,
        preview,
        meta,
        sensitive,
        hash,
        created_at: Utc::now().to_rfc3339(),
        workspace: state.workspace.lock().unwrap().clone(),
        tags: Vec::new(),
    }
}

fn build_image_clip(state: &AppState, img: &arboard::ImageData, hash: String) -> Option<Clip> {
    let rgba = image::RgbaImage::from_raw(
        img.width as u32,
        img.height as u32,
        img.bytes.as_ref().to_vec(),
    )?;
    let mut buf = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(rgba)
        .write_to(&mut buf, image::ImageFormat::Png)
        .ok()?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(buf.get_ref());
    let data_url = format!("data:image/png;base64,{}", b64);

    Some(Clip {
        id: now_id(&hash),
        kind: ClipType::Image,
        content: data_url,
        preview: format!("Image {}×{}", img.width, img.height),
        meta: serde_json::json!({ "width": img.width, "height": img.height }),
        sensitive: false,
        hash,
        created_at: Utc::now().to_rfc3339(),
        workspace: state.workspace.lock().unwrap().clone(),
        tags: Vec::new(),
    })
}

fn push_clip(app: &AppHandle, state: &AppState, clip: Clip) {
    {
        let mut clips = state.clips.lock().unwrap();
        clips.insert(0, clip.clone());
        if clips.len() > MAX_CLIPS {
            clips.truncate(MAX_CLIPS);
        }
    }
    let _ = app.emit("clip-added", &clip);
}
