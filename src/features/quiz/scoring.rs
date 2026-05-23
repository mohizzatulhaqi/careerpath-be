use std::collections::HashMap;
use uuid::Uuid;

// ── Input types (built by service from DB rows) ───────────────────────────────

#[derive(Debug, Clone)]
pub struct RoleWeight {
    pub role_id: Uuid,
    pub role_code: String,
    pub role_name: String,
    pub weight: i32,
}

#[derive(Debug, Clone)]
pub struct AnswerInput {
    pub question_id: Uuid,
    pub option_id: Uuid,
    pub question_text: String,
    pub option_text: String,
    pub weights_per_role: Vec<RoleWeight>,
}

#[derive(Debug, Clone)]
pub struct MaxPossiblePerRole {
    pub role_id: Uuid,
    pub role_code: String,
    pub role_name: String,
    pub max_total: i32,
}

// ── Output types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RoleScore {
    pub role_id: Uuid,
    pub role_code: String,
    pub role_name: String,
    pub score: i32,
    pub max_possible: i32,
}

#[derive(Debug, Clone)]
pub struct Contribution {
    pub question_text: String,
    pub option_text: String,
    pub contributed_weight: i32,
}

#[derive(Debug)]
pub struct ScoringResult {
    pub winning_role_id: Uuid,
    pub match_percentage: f64,
    pub scores: Vec<RoleScore>,
    pub top_contributions: Vec<Contribution>,
}

// ── Pure scoring function (no DB, no async) ───────────────────────────────────

pub fn calculate_scores(
    answers: &[AnswerInput],
    max_possible: &[MaxPossiblePerRole],
) -> ScoringResult {
    // Accumulate total score per role across all answers
    let mut score_map: HashMap<Uuid, i32> = max_possible
        .iter()
        .map(|m| (m.role_id, 0))
        .collect();

    for answer in answers {
        for rw in &answer.weights_per_role {
            *score_map.entry(rw.role_id).or_insert(0) += rw.weight;
        }
    }

    // Sort: higher score first; on tie, smaller role_code wins (deterministic)
    let mut ranked: Vec<(&MaxPossiblePerRole, i32)> = max_possible
        .iter()
        .map(|m| (m, score_map.get(&m.role_id).copied().unwrap_or(0)))
        .collect();

    ranked.sort_by(|(a_m, a_s), (b_m, b_s)| {
        b_s.cmp(a_s)
            .then_with(|| a_m.role_code.cmp(&b_m.role_code))
    });

    let (winner_meta, winning_score) = ranked.first().copied().unwrap_or_else(|| {
        // Should never happen if max_possible is non-empty
        panic!("calculate_scores called with empty max_possible")
    });

    let match_percentage = if winner_meta.max_total > 0 {
        (winning_score as f64 / winner_meta.max_total as f64 * 100.0).min(100.0)
    } else {
        0.0
    };

    let scores: Vec<RoleScore> = ranked
        .iter()
        .map(|(m, s)| RoleScore {
            role_id: m.role_id,
            role_code: m.role_code.clone(),
            role_name: m.role_name.clone(),
            score: *s,
            max_possible: m.max_total,
        })
        .collect();

    // Top-3 contributions for the winning role
    let winning_role_id = winner_meta.role_id;
    let mut contributions: Vec<Contribution> = answers
        .iter()
        .filter_map(|a| {
            let w = a.weights_per_role
                .iter()
                .find(|rw| rw.role_id == winning_role_id)
                .map_or(0, |rw| rw.weight);
            (w > 0).then(|| Contribution {
                question_text: a.question_text.clone(),
                option_text: a.option_text.clone(),
                contributed_weight: w,
            })
        })
        .collect();

    contributions.sort_by(|a, b| b.contributed_weight.cmp(&a.contributed_weight));
    contributions.truncate(3);

    ScoringResult {
        winning_role_id,
        match_percentage,
        scores,
        top_contributions: contributions,
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn rw(role_id: Uuid, code: &str, name: &str, w: i32) -> RoleWeight {
        RoleWeight { role_id, role_code: code.into(), role_name: name.into(), weight: w }
    }

    fn mp(role_id: Uuid, code: &str, name: &str, max: i32) -> MaxPossiblePerRole {
        MaxPossiblePerRole { role_id, role_code: code.into(), role_name: name.into(), max_total: max }
    }

    fn answer(q: Uuid, o: Uuid, q_txt: &str, o_txt: &str, weights: Vec<RoleWeight>) -> AnswerInput {
        AnswerInput {
            question_id: q,
            option_id: o,
            question_text: q_txt.into(),
            option_text: o_txt.into(),
            weights_per_role: weights,
        }
    }

    #[test]
    fn test_happy_path_winner() {
        let fe = Uuid::new_v4();
        let be = Uuid::new_v4();
        let da = Uuid::new_v4();

        let answers = vec![
            answer(Uuid::new_v4(), Uuid::new_v4(), "Q1", "fe option",
                vec![rw(fe, "frontend", "Frontend Dev", 5), rw(be, "backend", "Backend Dev", 0), rw(da, "data_analyst", "DA", 1)]),
            answer(Uuid::new_v4(), Uuid::new_v4(), "Q2", "fe option",
                vec![rw(fe, "frontend", "Frontend Dev", 4), rw(be, "backend", "Backend Dev", 1), rw(da, "data_analyst", "DA", 0)]),
            answer(Uuid::new_v4(), Uuid::new_v4(), "Q3", "fe option",
                vec![rw(fe, "frontend", "Frontend Dev", 5), rw(be, "backend", "Backend Dev", 0), rw(da, "data_analyst", "DA", 2)]),
        ];
        let max_possible = vec![
            mp(fe, "frontend",     "Frontend Dev", 14), // 5+4+5
            mp(be, "backend",      "Backend Dev",  5),
            mp(da, "data_analyst", "DA",           5),
        ];

        let result = calculate_scores(&answers, &max_possible);

        assert_eq!(result.winning_role_id, fe);
        assert_eq!(result.scores[0].score, 14); // fe total
        assert!((result.match_percentage - 100.0).abs() < 0.01);
        assert!(!result.top_contributions.is_empty());
    }

    #[test]
    fn test_tie_breaker_alphabetical() {
        let be = Uuid::new_v4();
        let fe = Uuid::new_v4();

        let answers = vec![
            answer(Uuid::new_v4(), Uuid::new_v4(), "Q1", "tie option",
                vec![rw(fe, "frontend", "Frontend Dev", 3), rw(be, "backend", "Backend Dev", 3)]),
        ];
        let max_possible = vec![
            mp(fe, "frontend", "Frontend Dev", 3),
            mp(be, "backend",  "Backend Dev",  3),
        ];

        let result = calculate_scores(&answers, &max_possible);

        // "backend" < "frontend" alphabetically → backend wins the tie
        assert_eq!(result.winning_role_id, be);
    }

    #[test]
    fn test_all_zero_weights_no_panic() {
        let fe = Uuid::new_v4();
        let be = Uuid::new_v4();

        let answers = vec![
            answer(Uuid::new_v4(), Uuid::new_v4(), "Q1", "zero option",
                vec![rw(fe, "frontend", "Frontend Dev", 0), rw(be, "backend", "Backend Dev", 0)]),
        ];
        let max_possible = vec![
            mp(fe, "frontend", "Frontend Dev", 0),
            mp(be, "backend",  "Backend Dev",  0),
        ];

        let result = calculate_scores(&answers, &max_possible);

        // Must not panic; percentage is 0
        assert_eq!(result.match_percentage, 0.0);
        assert!(result.scores.iter().all(|s| s.score == 0));
    }

    #[test]
    fn test_top_contributions_sorted_and_capped() {
        let fe = Uuid::new_v4();
        let be = Uuid::new_v4();

        // 5 questions, picking frontend-heavy options
        let mut answers = Vec::new();
        let weights = [5i32, 3, 4, 1, 2];
        for (i, &w) in weights.iter().enumerate() {
            answers.push(answer(
                Uuid::new_v4(), Uuid::new_v4(),
                &format!("Q{}", i + 1), &format!("opt {}", i + 1),
                vec![rw(fe, "frontend", "Frontend Dev", w), rw(be, "backend", "Backend Dev", 0)],
            ));
        }
        let max_possible = vec![
            mp(fe, "frontend", "Frontend Dev", 15),
            mp(be, "backend",  "Backend Dev",  5),
        ];

        let result = calculate_scores(&answers, &max_possible);

        assert_eq!(result.winning_role_id, fe);
        // Should only have 3 contributions max
        assert!(result.top_contributions.len() <= 3);
        // Should be sorted descending
        let contrib_weights: Vec<i32> = result.top_contributions.iter()
            .map(|c| c.contributed_weight)
            .collect();
        assert!(contrib_weights.windows(2).all(|w| w[0] >= w[1]),
            "top_contributions not sorted descending: {:?}", contrib_weights);
        // Top 3 should be weights 5, 4, 3
        assert_eq!(contrib_weights, vec![5, 4, 3]);
    }
}
