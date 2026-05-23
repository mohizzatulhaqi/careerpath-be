use std::{collections::HashMap, sync::Arc};

use uuid::Uuid;

use crate::{
    features::quiz::{
        dto::{
            AnswerSavedResponse, AttemptCreatedResponse, HistoryItem, QuizResultResponse,
            RoleDto, RoleScoreDto, TopReason,
        },
        error::QuizError,
        repository::{self, AnswerWeightRow, MaxPossibleRow},
        scoring::{self, AnswerInput, MaxPossiblePerRole, RoleWeight},
    },
    shared::pagination::{PaginatedResponse, PaginationQuery},
    state::AppState,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn group_answer_rows(rows: Vec<AnswerWeightRow>) -> Vec<AnswerInput> {
    let mut map: HashMap<Uuid, AnswerInput> = HashMap::new();
    for row in rows {
        let entry = map.entry(row.question_id).or_insert_with(|| AnswerInput {
            question_id: row.question_id,
            option_id: row.option_id,
            question_text: row.question_text.clone(),
            option_text: row.option_text.clone(),
            weights_per_role: Vec::new(),
        });
        entry.weights_per_role.push(RoleWeight {
            role_id: row.role_id,
            role_code: row.role_code,
            role_name: row.role_name,
            weight: row.weight,
        });
    }
    map.into_values().collect()
}

fn to_max_possible(rows: Vec<MaxPossibleRow>) -> Vec<MaxPossiblePerRole> {
    rows.into_iter()
        .map(|r| MaxPossiblePerRole {
            role_id: r.role_id,
            role_code: r.role_code,
            role_name: r.role_name,
            max_total: r.max_total,
        })
        .collect()
}

async fn build_result(
    state: &Arc<AppState>,
    attempt_id: Uuid,
    winning_role_id: Uuid,
    match_percentage: f64,
    submitted_at: chrono::DateTime<chrono::Utc>,
) -> Result<QuizResultResponse, QuizError> {
    let role = repository::find_role_by_id(&state.db, winning_role_id)
        .await?
        .ok_or(QuizError::AttemptNotFound)?;

    let raw_rows = repository::get_answers_with_weights(&state.db, attempt_id).await?;
    let answers = group_answer_rows(raw_rows);
    let max_rows = repository::get_max_possible_per_role(&state.db).await?;
    let max_possible = to_max_possible(max_rows);

    let scored = scoring::calculate_scores(&answers, &max_possible);

    Ok(QuizResultResponse {
        attempt_id,
        submitted_at,
        role: RoleDto {
            id: role.id,
            code: role.code,
            name: role.name,
            description: role.description,
        },
        match_percentage,
        top_reasons: scored
            .top_contributions
            .into_iter()
            .map(|c| TopReason {
                question_text: c.question_text,
                option_text: c.option_text,
                contributed_weight: c.contributed_weight,
            })
            .collect(),
        all_scores: scored
            .scores
            .into_iter()
            .map(|s| RoleScoreDto {
                role_code: s.role_code,
                role_name: s.role_name,
                score: s.score,
                max_possible: s.max_possible,
            })
            .collect(),
    })
}

// ── Public service functions ──────────────────────────────────────────────────

/// GET /api/quiz/questions
pub async fn get_questions(
    state: &Arc<AppState>,
) -> Result<Vec<crate::features::quiz::dto::QuestionDto>, QuizError> {
    use crate::features::quiz::dto::{OptionDto, QuestionDto};
    use std::collections::BTreeMap;

    let rows = repository::get_active_question_option_rows(&state.db).await?;

    // Group: BTreeMap preserves insertion order; rows are already sorted by question_order
    let mut grouped: BTreeMap<(i32, Uuid), QuestionDto> = BTreeMap::new();
    for row in rows {
        let entry = grouped
            .entry((row.question_order, row.question_id))
            .or_insert_with(|| QuestionDto {
                id: row.question_id,
                question_text: row.question_text.clone(),
                order_index: row.question_order,
                options: Vec::new(),
            });
        entry.options.push(OptionDto {
            id: row.option_id,
            option_text: row.option_text,
            order_index: row.option_order,
        });
    }
    Ok(grouped.into_values().collect())
}

/// POST /api/quiz/attempts
pub async fn create_or_resume_attempt(
    state: &Arc<AppState>,
    user_id: Uuid,
) -> Result<AttemptCreatedResponse, QuizError> {
    let attempt = match repository::find_in_progress_attempt(&state.db, user_id).await? {
        Some(existing) => existing,
        None => repository::create_attempt(&state.db, user_id).await?,
    };

    Ok(AttemptCreatedResponse {
        attempt_id: attempt.id,
        status: attempt.status,
        started_at: attempt.started_at,
    })
}

/// POST /api/quiz/attempts/:id/answers
pub async fn save_answer(
    state: &Arc<AppState>,
    user_id: Uuid,
    attempt_id: Uuid,
    question_id: Uuid,
    option_id: Uuid,
) -> Result<AnswerSavedResponse, QuizError> {
    let attempt = repository::find_attempt_by_id(&state.db, attempt_id)
        .await?
        .ok_or(QuizError::AttemptNotFound)?;

    if attempt.user_id != user_id {
        return Err(QuizError::AttemptNotOwned);
    }
    if attempt.status != "in_progress" {
        return Err(QuizError::AttemptAlreadySubmitted);
    }

    let valid = repository::option_belongs_to_question(&state.db, option_id, question_id).await?;
    if !valid {
        return Err(QuizError::InvalidOption);
    }

    repository::upsert_answer(&state.db, attempt_id, question_id, option_id).await?;

    Ok(AnswerSavedResponse { saved: true, question_id, option_id })
}

/// POST /api/quiz/attempts/:id/submit
pub async fn submit_attempt(
    state: &Arc<AppState>,
    user_id: Uuid,
    attempt_id: Uuid,
) -> Result<QuizResultResponse, QuizError> {
    let attempt = repository::find_attempt_by_id(&state.db, attempt_id)
        .await?
        .ok_or(QuizError::AttemptNotFound)?;

    if attempt.user_id != user_id {
        return Err(QuizError::AttemptNotOwned);
    }
    if attempt.status != "in_progress" {
        return Err(QuizError::AttemptAlreadySubmitted);
    }

    // Completeness check
    let required = repository::count_active_questions(&state.db).await? as usize;
    let answered = repository::count_attempt_answers(&state.db, attempt_id).await? as usize;
    if answered < required {
        return Err(QuizError::IncompleteAnswers { answered, required });
    }

    // Compute scores
    let raw_rows = repository::get_answers_with_weights(&state.db, attempt_id).await?;
    let answers = group_answer_rows(raw_rows);
    let max_rows = repository::get_max_possible_per_role(&state.db).await?;
    let max_possible = to_max_possible(max_rows);
    let scored = scoring::calculate_scores(&answers, &max_possible);

    // Persist result inside a transaction
    let mut tx = state.db.begin().await?;
    repository::update_attempt_submitted(
        &mut tx,
        attempt_id,
        scored.winning_role_id,
        scored.match_percentage,
    )
    .await?;
    tx.commit().await?;

    // Build response (re-uses already-computed scored data)
    let role = repository::find_role_by_id(&state.db, scored.winning_role_id)
        .await?
        .ok_or(QuizError::AttemptNotFound)?;

    Ok(QuizResultResponse {
        attempt_id,
        submitted_at: chrono::Utc::now(),
        role: RoleDto {
            id: role.id,
            code: role.code,
            name: role.name,
            description: role.description,
        },
        match_percentage: scored.match_percentage,
        top_reasons: scored
            .top_contributions
            .into_iter()
            .map(|c| TopReason {
                question_text: c.question_text,
                option_text: c.option_text,
                contributed_weight: c.contributed_weight,
            })
            .collect(),
        all_scores: scored
            .scores
            .into_iter()
            .map(|s| RoleScoreDto {
                role_code: s.role_code,
                role_name: s.role_name,
                score: s.score,
                max_possible: s.max_possible,
            })
            .collect(),
    })
}

/// GET /api/quiz/attempts/:id/result
pub async fn get_result(
    state: &Arc<AppState>,
    user_id: Uuid,
    attempt_id: Uuid,
) -> Result<QuizResultResponse, QuizError> {
    let attempt = repository::find_attempt_by_id(&state.db, attempt_id)
        .await?
        .ok_or(QuizError::AttemptNotFound)?;

    if attempt.user_id != user_id {
        return Err(QuizError::AttemptNotOwned);
    }
    if attempt.status != "submitted" {
        return Err(QuizError::AttemptNotSubmitted);
    }

    let winning_role_id = attempt.result_role_id.ok_or(QuizError::AttemptNotFound)?;
    let match_percentage = attempt.match_percentage.unwrap_or(0.0);
    let submitted_at = attempt.submitted_at.ok_or(QuizError::AttemptNotFound)?;

    build_result(state, attempt_id, winning_role_id, match_percentage, submitted_at).await
}

/// GET /api/quiz/history
pub async fn get_history(
    state: &Arc<AppState>,
    user_id: Uuid,
    pagination: &PaginationQuery,
) -> Result<PaginatedResponse<HistoryItem>, QuizError> {
    let rows =
        repository::get_history(&state.db, user_id, pagination.limit(), pagination.offset())
            .await?;
    let total = repository::count_history(&state.db, user_id).await?;

    let items = rows
        .into_iter()
        .map(|r| HistoryItem {
            attempt_id: r.attempt_id,
            submitted_at: r.submitted_at,
            role_name: r.role_name,
            role_code: r.role_code,
            match_percentage: r.match_percentage,
        })
        .collect();

    Ok(PaginatedResponse::new(items, total, pagination))
}
