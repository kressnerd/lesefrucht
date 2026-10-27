//! Chapter-aware text extraction.

pub mod epub;
pub mod pdf;
pub mod unsupported;

use crate::domain::Chapter;

#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    #[error("unsupported format")]
    Unsupported,
    #[error("extraction failed: {0}")]
    Failed(String),
    #[error("the edition has no text")]
    EmptyText,
}

pub trait Extractor {
    fn extract(&self, bytes: &[u8]) -> Result<Vec<Chapter>, ExtractError>;
}

pub fn extract(format: &str, bytes: &[u8]) -> Result<Vec<Chapter>, ExtractError> {
    match format.to_ascii_lowercase().as_str() {
        "epub" => epub::EpubExtractor.extract(bytes),
        "pdf" => pdf::PdfExtractor.extract(bytes),
        _ => Err(ExtractError::Unsupported),
    }
}

/// Minimal HTML-to-text: drop markup, keep block boundaries as newlines.
pub fn html_to_text(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut chars = html.chars().peekable();
    let mut in_tag = false;
    let mut tag = String::new();
    let mut skip_until: Option<&str> = None;
    while let Some(character) = chars.next() {
        if let Some(end) = skip_until {
            if character == '<' {
                let mut lookahead = String::new();
                for next in chars.by_ref() {
                    lookahead.push(next);
                    if lookahead.ends_with(end) {
                        break;
                    }
                }
                skip_until = None;
            }
            continue;
        }
        if in_tag {
            if character == '>' {
                in_tag = false;
                let lower = tag.to_ascii_lowercase();
                if lower.starts_with("script") {
                    skip_until = Some("</script>");
                } else if lower.starts_with("style") {
                    skip_until = Some("</style>");
                } else if lower.starts_with("br")
                    || lower.starts_with("/p")
                    || lower.starts_with("/div")
                    || lower.starts_with("/h")
                    || lower.starts_with("/li")
                    || lower.starts_with("/tr")
                {
                    output.push('\n');
                }
                tag.clear();
            } else {
                tag.push(character);
            }
            continue;
        }
        if character == '<' {
            in_tag = true;
            continue;
        }
        output.push(character);
    }
    decode_entities(&output)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn decode_entities(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(position) = rest.find('&') {
        output.push_str(&rest[..position]);
        rest = &rest[position..];
        let Some(end) = rest.find(';') else {
            output.push_str(rest);
            return output;
        };
        let entity = &rest[1..end];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" | "#39" => Some('\''),
            "nbsp" => Some(' '),
            other => other
                .strip_prefix("#x")
                .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                .or_else(|| other.strip_prefix('#').and_then(|digits| digits.parse().ok()))
                .and_then(char::from_u32),
        };
        match decoded {
            Some(character) => output.push(character),
            None => output.push_str(&rest[..=end]),
        }
        rest = &rest[end + 1..];
    }
    output.push_str(rest);
    output
}
