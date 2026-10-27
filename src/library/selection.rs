//! Exactly one selected edition per work, by format preference.

pub fn select_format(formats: &[String], preference: &[String]) -> Option<String> {
    preference
        .iter()
        .map(|format| format.to_ascii_lowercase())
        .find(|format| formats.contains(format))
        .or_else(|| formats.first().cloned())
}
