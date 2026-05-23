pub const MINI_QUIZ_PASSING_SCORE: f64 = 100.0;

/// Returns (score_percentage, passed).
/// Passed when all questions answered correctly (exact match).
pub fn score_mini_quiz(correct_count: usize, total: usize) -> (f64, bool) {
    if total == 0 {
        return (0.0, false);
    }
    let score = (correct_count as f64 / total as f64) * 100.0;
    let passed = correct_count == total;
    (score, passed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_correct_passes() {
        let (score, passed) = score_mini_quiz(3, 3);
        assert_eq!(score, 100.0);
        assert!(passed);
    }

    #[test]
    fn partial_correct_fails() {
        let (score, passed) = score_mini_quiz(2, 3);
        assert!((score - 66.66).abs() < 1.0);
        assert!(!passed);
    }

    #[test]
    fn zero_correct_fails() {
        let (score, passed) = score_mini_quiz(0, 3);
        assert_eq!(score, 0.0);
        assert!(!passed);
    }

    #[test]
    fn empty_quiz_fails() {
        let (score, passed) = score_mini_quiz(0, 0);
        assert_eq!(score, 0.0);
        assert!(!passed);
    }
}
