use uuid::Uuid;

pub struct EligibilityInput {
    pub has_submitted_quiz: bool,
    pub role_id: Option<Uuid>,
    pub total_modules: usize,
    pub completed_modules: usize,
    pub final_project_approved: bool,
    pub final_project_submission_id: Option<Uuid>,
}

pub enum EligibilityResult {
    Eligible { role_id: Uuid, submission_id: Uuid },
    NotEligible { reason: String },
}

pub fn check_eligibility(input: EligibilityInput) -> EligibilityResult {
    if !input.has_submitted_quiz {
        return EligibilityResult::NotEligible {
            reason: "belum submit quiz role".into(),
        };
    }
    let role_id = match input.role_id {
        Some(id) => id,
        None => {
            return EligibilityResult::NotEligible {
                reason: "role belum di-resolve".into(),
            }
        }
    };
    if input.total_modules == 0 {
        return EligibilityResult::NotEligible {
            reason: "role tidak punya modul".into(),
        };
    }
    if input.completed_modules < input.total_modules {
        return EligibilityResult::NotEligible {
            reason: format!(
                "baru selesai {}/{} modul",
                input.completed_modules, input.total_modules
            ),
        };
    }
    if !input.final_project_approved {
        return EligibilityResult::NotEligible {
            reason: "final project belum approved".into(),
        };
    }
    let submission_id = match input.final_project_submission_id {
        Some(id) => id,
        None => {
            return EligibilityResult::NotEligible {
                reason: "submission_id missing".into(),
            }
        }
    };
    EligibilityResult::Eligible {
        role_id,
        submission_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eligible_input() -> EligibilityInput {
        EligibilityInput {
            has_submitted_quiz: true,
            role_id: Some(Uuid::new_v4()),
            total_modules: 3,
            completed_modules: 3,
            final_project_approved: true,
            final_project_submission_id: Some(Uuid::new_v4()),
        }
    }

    #[test]
    fn no_quiz_not_eligible() {
        let input = EligibilityInput {
            has_submitted_quiz: false,
            ..eligible_input()
        };
        assert!(matches!(
            check_eligibility(input),
            EligibilityResult::NotEligible { .. }
        ));
    }

    #[test]
    fn no_modules_not_eligible() {
        let input = EligibilityInput {
            total_modules: 0,
            completed_modules: 0,
            ..eligible_input()
        };
        assert!(matches!(
            check_eligibility(input),
            EligibilityResult::NotEligible { .. }
        ));
    }

    #[test]
    fn partial_modules_not_eligible() {
        let input = EligibilityInput {
            total_modules: 3,
            completed_modules: 2,
            ..eligible_input()
        };
        match check_eligibility(input) {
            EligibilityResult::NotEligible { reason } => {
                assert!(reason.contains("2/3"), "reason: {reason}");
            }
            EligibilityResult::Eligible { .. } => panic!("should be not eligible"),
        }
    }

    #[test]
    fn project_not_approved_not_eligible() {
        let input = EligibilityInput {
            final_project_approved: false,
            ..eligible_input()
        };
        assert!(matches!(
            check_eligibility(input),
            EligibilityResult::NotEligible { .. }
        ));
    }

    #[test]
    fn all_conditions_met_eligible() {
        let input = eligible_input();
        assert!(matches!(
            check_eligibility(input),
            EligibilityResult::Eligible { .. }
        ));
    }
}
