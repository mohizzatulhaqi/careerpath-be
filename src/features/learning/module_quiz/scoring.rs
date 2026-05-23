pub const FINAL_QUIZ_PASSING_SCORE: f64 = 70.0;

/// Returns (score_percentage, passed).
/// Passed when score >= FINAL_QUIZ_PASSING_SCORE.
pub fn score_final_quiz(correct_count: usize, total: usize) -> (f64, bool) {
    if total == 0 {
        return (0.0, false);
    }
    let score = (correct_count as f64 / total as f64) * 100.0;
    let passed = score >= FINAL_QUIZ_PASSING_SCORE;
    (score, passed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_correct_passes() {
        let (score, passed) = score_final_quiz(5, 5);
        assert_eq!(score, 100.0);
        assert!(passed);
    }

    #[test]
    fn exactly_seventy_passes() {
        // 7 out of 10
        let (score, passed) = score_final_quiz(7, 10);
        assert_eq!(score, 70.0);
        assert!(passed);
    }

    #[test]
    fn below_seventy_fails() {
        // 3 out of 5 = 60%
        let (score, passed) = score_final_quiz(3, 5);
        assert_eq!(score, 60.0);
        assert!(!passed);
    }

    #[test]
    fn zero_correct_fails() {
        let (score, passed) = score_final_quiz(0, 5);
        assert_eq!(score, 0.0);
        assert!(!passed);
    }
}
