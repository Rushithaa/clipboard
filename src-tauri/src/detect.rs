use crate::models::ClipType;
use regex::Regex;
use serde_json::json;
use std::sync::OnceLock;

fn url_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\s*https?://[^\s]+\s*$").unwrap())
}

fn hex_color_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\s*#?([0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})\s*$").unwrap())
}

/// Heuristically classify a piece of text and produce associated metadata.
pub fn detect_kind(text: &str) -> (ClipType, serde_json::Value) {
    let trimmed = text.trim();

    // Treat a letterless value (e.g. "845213") as a color only when it is
    // explicitly prefixed with '#', so OTP-like digit runs aren't misread.
    let looks_like_color = hex_color_re().is_match(trimmed)
        && (trimmed.starts_with('#') || trimmed.chars().any(|c| c.is_ascii_alphabetic()));
    if looks_like_color {
        let hex = if trimmed.starts_with('#') {
            trimmed.to_string()
        } else {
            format!("#{}", trimmed)
        };
        return (ClipType::Color, json!({ "hex": hex }));
    }

    if url_re().is_match(trimmed) {
        let host = trimmed.split('/').nth(2).unwrap_or("").to_string();
        // Title/favicon are fetched by the metadata enricher in a later phase.
        return (ClipType::Url, json!({ "host": host, "title": null, "favicon": null }));
    }

    if (trimmed.starts_with('{') || trimmed.starts_with('['))
        && serde_json::from_str::<serde_json::Value>(trimmed).is_ok()
    {
        return (ClipType::Code, json!({ "language": "json" }));
    }

    if let Some(lang) = detect_language(trimmed) {
        return (ClipType::Code, json!({ "language": lang }));
    }

    (ClipType::Text, json!({ "chars": trimmed.chars().count() }))
}

/// Very small language guesser. Returns `None` if the text does not look like code.
fn detect_language(text: &str) -> Option<&'static str> {
    let multiline = text.contains('\n');
    let symbols = text.matches(['{', '}', ';', '(', ')', '=', '<', '>']).count();
    let looks_codey = symbols >= 3 && (multiline || text.len() < 200);

    if !looks_codey {
        return None;
    }

    let t = text;
    if t.contains("def ") || t.contains("import ") && t.contains(":") {
        Some("python")
    } else if t.contains("=>") || t.contains("const ") || t.contains("function ") || t.contains("useState") || t.contains("useEffect") {
        Some("javascript")
    } else if t.contains("fn ") || t.contains("let mut ") || t.contains("impl ") {
        Some("rust")
    } else if t.contains("public class") || t.contains("System.out") {
        Some("java")
    } else if t.contains("#include") || t.contains("std::") {
        Some("cpp")
    } else if t.contains("SELECT ") || t.contains("select ") && t.contains(" from ") {
        Some("sql")
    } else if t.trim_start().starts_with('{') || t.trim_start().starts_with('[') {
        Some("json")
    } else if symbols >= 3 {
        Some("plaintext-code")
    } else {
        None
    }
}

/// Detect content that looks sensitive (passwords, OTPs, card numbers, keys).
pub fn is_sensitive(text: &str) -> bool {
    let t = text.trim();

    // One-time codes: a standalone 6-8 digit number.
    let otp = Regex::new(r"^\d{6,8}$").unwrap();
    if otp.is_match(t) {
        return true;
    }

    // Credit-card-like: 13-19 digits, optionally grouped.
    let digits: String = t.chars().filter(|c| c.is_ascii_digit()).collect();
    let only_card_chars = t.chars().all(|c| c.is_ascii_digit() || c == ' ' || c == '-');
    if only_card_chars && (13..=19).contains(&digits.len()) && luhn_valid(&digits) {
        return true;
    }

    // Common secret-bearing tokens.
    let lower = t.to_lowercase();
    let key_markers = ["password", "passwd", "secret", "api_key", "apikey", "bearer ", "ssh-rsa", "-----begin"];
    if key_markers.iter().any(|m| lower.contains(m)) {
        return true;
    }

    false
}

fn luhn_valid(digits: &str) -> bool {
    let mut sum = 0u32;
    let mut alt = false;
    for c in digits.chars().rev() {
        let mut d = c.to_digit(10).unwrap_or(0);
        if alt {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
        alt = !alt;
    }
    sum.is_multiple_of(10)
}

/// Build a short single-line preview for list rendering.
pub fn make_preview(text: &str, max: usize) -> String {
    let one_line: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if one_line.chars().count() > max {
        let truncated: String = one_line.chars().take(max).collect();
        format!("{}…", truncated)
    } else {
        one_line
    }
}
