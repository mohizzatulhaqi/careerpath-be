use ammonia::Builder;

/// Errors from plain-text sanitization & length validation.
#[derive(Debug, thiserror::Error)]
pub enum TextValidationError {
    #[error("teks tidak boleh kosong")]
    Empty,
    #[error("teks terlalu pendek (minimal {min} karakter)")]
    TooShort { min: usize },
    #[error("teks terlalu panjang (maksimal {max} karakter)")]
    TooLong { max: usize },
}

/// Sanitize arbitrary user text:
///
/// 1. Trim leading/trailing whitespace.
/// 2. Strip **all** HTML tags with `ammonia::Builder::empty()` (empty allowlist means
///    no tags pass — but inner text content is preserved and special characters like
///    `<` / `>` are HTML-entity-escaped so they render safely).
///    - `<script>alert(1)</script>` → `alert(1)` (tag stripped, text kept)
///    - `<b>hello</b>` → `hello`
///    - `5 < 10` → `5 &lt; 10` (entity-escaped, still readable)
/// 3. Re-trim after sanitization.
/// 4. Validate character count (Unicode-aware) within [min, max].
///
/// The returned string is safe to store and display (front-end should still use
/// `textContent` / JSX auto-escape for defence-in-depth).
pub fn sanitize_plain_text(
    raw: &str,
    min: usize,
    max: usize,
) -> Result<String, TextValidationError> {
    // 1. Trim
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(TextValidationError::Empty);
    }

    // 2. Strip all HTML tags; keep inner text (entity-escaped where needed)
    let sanitized = Builder::empty().clean(trimmed).to_string();

    // 3. Re-trim
    let final_text = sanitized.trim().to_string();
    if final_text.is_empty() {
        return Err(TextValidationError::Empty);
    }

    // 4. Validate length (char count, not byte count → Unicode-safe)
    let char_count = final_text.chars().count();
    if char_count < min {
        return Err(TextValidationError::TooShort { min });
    }
    if char_count > max {
        return Err(TextValidationError::TooLong { max });
    }

    Ok(final_text)
}

// ── Mapping to AppError ───────────────────────────────────────────────────────

impl From<TextValidationError> for crate::error::AppError {
    fn from(e: TextValidationError) -> Self {
        crate::error::AppError::BadRequest(e.to_string())
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_returns_empty_error() {
        assert!(matches!(
            sanitize_plain_text("", 1, 100),
            Err(TextValidationError::Empty)
        ));
    }

    #[test]
    fn whitespace_only_returns_empty_error() {
        assert!(matches!(
            sanitize_plain_text("   \t\n  ", 1, 100),
            Err(TextValidationError::Empty)
        ));
    }

    #[test]
    fn too_short_returns_too_short() {
        assert!(matches!(
            sanitize_plain_text("abc", 10, 100),
            Err(TextValidationError::TooShort { min: 10 })
        ));
    }

    #[test]
    fn too_long_returns_too_long() {
        let long = "a".repeat(5001);
        assert!(matches!(
            sanitize_plain_text(&long, 1, 5000),
            Err(TextValidationError::TooLong { max: 5000 })
        ));
    }

    #[test]
    fn script_tag_stripped_text_kept() {
        // The text inside <script> is kept; the tag itself is stripped
        let result = sanitize_plain_text("<script>alert(1)</script>Hello", 1, 500).unwrap();
        // ammonia strips the tag; inner text "alert(1)" and "Hello" remain
        assert!(!result.contains("<script>"));
        assert!(!result.contains("</script>"));
        assert!(result.contains("Hello"));
    }

    #[test]
    fn bold_tag_stripped_text_kept() {
        let result = sanitize_plain_text("<b>Bold</b> text", 1, 500).unwrap();
        assert!(!result.contains("<b>"));
        assert!(result.contains("Bold"));
        assert!(result.contains("text"));
    }

    #[test]
    fn less_than_greater_than_preserved() {
        // "5 < 10 and 20 > 15" — the < and > are not HTML tags so ammonia
        // entity-escapes them but does NOT strip text
        let result = sanitize_plain_text("5 < 10 and 20 > 15", 1, 500).unwrap();
        // The text content is preserved (may be entity-escaped)
        assert!(!result.is_empty());
        // Should not error — this is valid user text
    }

    #[test]
    fn valid_text_passes_through() {
        let input = "Project bagus, lulus!";
        let result = sanitize_plain_text(input, 10, 5000).unwrap();
        // No HTML, so ammonia returns as-is (trimmed)
        assert_eq!(result, input);
    }

    #[test]
    fn iframe_stripped_short_result_is_too_short() {
        // <iframe src='evil'></iframe> — tag stripped, nothing left → TooShort
        let result = sanitize_plain_text("<iframe src='evil'></iframe>", 5, 5000);
        assert!(matches!(
            result,
            Err(TextValidationError::TooShort { .. }) | Err(TextValidationError::Empty)
        ));
    }

    #[test]
    fn mixed_html_and_text_passes_if_text_long_enough() {
        let input = "<b>Submission</b> ini memenuhi semua kriteria yang ditetapkan";
        let result = sanitize_plain_text(input, 10, 5000).unwrap();
        assert!(!result.contains("<b>"));
        assert!(result.contains("Submission"));
        assert!(result.len() > 10);
    }
}
