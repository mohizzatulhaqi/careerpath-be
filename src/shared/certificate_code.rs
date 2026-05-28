use rand::Rng;

// Exclude visually ambiguous characters: 0/O, 1/I
const CODE_CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

pub fn generate_code(year: i32) -> String {
    let mut rng = rand::thread_rng();
    let suffix: String = (0..8)
        .map(|_| {
            let idx = rng.gen_range(0..CODE_CHARS.len());
            CODE_CHARS[idx] as char
        })
        .collect();
    format!("CERT-{}-{}", year, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn format_is_correct() {
        let code = generate_code(2026);
        // CERT-YYYY-XXXXXXXX
        assert!(code.starts_with("CERT-2026-"), "got: {code}");
        assert_eq!(code.len(), 18, "got: {code}");
        let suffix = &code[10..];
        assert_eq!(suffix.len(), 8);
        for ch in suffix.chars() {
            assert!(CODE_CHARS.contains(&(ch as u8)), "ambiguous char {ch} in code {code}");
        }
    }

    #[test]
    fn no_ambiguous_chars() {
        for _ in 0..200 {
            let code = generate_code(2026);
            let suffix = &code[10..];
            assert!(!suffix.contains('0'), "found 0 in {code}");
            assert!(!suffix.contains('O'), "found O in {code}");
            assert!(!suffix.contains('1'), "found 1 in {code}");
            assert!(!suffix.contains('I'), "found I in {code}");
        }
    }

    #[test]
    fn generates_distinct_codes() {
        let codes: HashSet<String> = (0..100).map(|_| generate_code(2026)).collect();
        // With 32^8 ≈ 10^12 possibilities, 100 codes should all be unique
        assert_eq!(codes.len(), 100);
    }
}
