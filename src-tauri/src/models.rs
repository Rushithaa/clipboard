use serde::{Deserialize, Serialize};

/// The detected kind of a captured clip.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClipType {
    Text,
    Url,
    Code,
    Color,
    Image,
}

/// A single captured clipboard entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: String,
    pub kind: ClipType,
    /// For text-like clips this is the raw text. For images this is a
    /// `data:image/png;base64,...` preview URL.
    pub content: String,
    /// Short preview used in list views (never contains full image data).
    pub preview: String,
    /// Free-form metadata: detected language, url host, dimensions, etc.
    pub meta: serde_json::Value,
    /// Whether the clip looks like a secret (password / OTP / card number).
    pub sensitive: bool,
    /// SHA-256 of the source content, used for de-duplication and copy-diffing.
    pub hash: String,
    /// RFC3339 timestamp of when the clip was captured.
    pub created_at: String,
    /// Workspace the clip belongs to (defaults to "default").
    pub workspace: String,
    /// AI / heuristic tags. Populated by the semantic layer in a later phase.
    pub tags: Vec<String>,
}
