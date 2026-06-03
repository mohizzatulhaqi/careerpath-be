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


pub fn sanitize_plain_text(
    raw: &str,
    min: usize,
    max: usize,
) -> Result<String, TextValidationError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(TextValidationError::Empty);
    }

    let sanitized = Builder::empty().clean(trimmed).to_string();

    let final_text = sanitized.trim().to_string();
    if final_text.is_empty() {
        return Err(TextValidationError::Empty);
    }

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
        let result = sanitize_plain_text("<script>alert(1)</script>Hello", 1, 500).unwrap();
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
        // entity-escapes them but does NOT strip text
        let result = sanitize_plain_text("5 < 10 and 20 > 15", 1, 500).unwrap();
        // The text content is preserved (may be entity-escaped)
        assert!(!result.is_empty());
        // Should not error  this is valid user text
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
