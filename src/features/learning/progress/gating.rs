use std::collections::HashSet;
use uuid::Uuid;

// ── Input types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub module_id: Uuid,
    pub order_index: i32,
}

#[derive(Debug, Clone)]
pub struct SubmaterialInfo {
    pub submaterial_id: Uuid,
    pub order_index: i32,
}

// ── Output types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleGateStatus {
    pub module_id: Uuid,
    pub is_unlocked: bool,
    pub is_completed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubmaterialGateStatus {
    pub submaterial_id: Uuid,
    pub is_unlocked: bool,
    pub is_completed: bool,
}

// ── Pure gating functions ─────────────────────────────────────────────────────

/// Determine which modules are unlocked based on sequential ordering and
/// completion of previous modules.
///
/// Rules:
/// - The first module (lowest order_index) is always unlocked.
/// - Module N is unlocked only if module N-1 is completed.
/// - A module is completed if its ID is in `completed_module_ids`.
pub fn compute_module_gates(
    modules_ordered: &[ModuleInfo],
    completed_module_ids: &HashSet<Uuid>,
) -> Vec<ModuleGateStatus> {
    let mut result = Vec::with_capacity(modules_ordered.len());
    let mut prev_completed = true; // first module is always unlocked

    for module in modules_ordered {
        let is_completed = completed_module_ids.contains(&module.module_id);
        let is_unlocked = prev_completed;

        result.push(ModuleGateStatus {
            module_id: module.module_id,
            is_unlocked,
            is_completed,
        });

        // Next module can only unlock if current one is completed AND unlocked
        prev_completed = is_completed && is_unlocked;
    }

    result
}

/// Determine which submaterials are unlocked within a single module.
///
/// Rules:
/// - If the module itself is locked, all submaterials are locked.
/// - The first submaterial (lowest order_index) is always unlocked if module is unlocked.
/// - Submaterial N is unlocked only if submaterial N-1 is completed.
pub fn compute_submaterial_gates(
    module_is_unlocked: bool,
    submaterials_ordered: &[SubmaterialInfo],
    completed_submaterial_ids: &HashSet<Uuid>,
) -> Vec<SubmaterialGateStatus> {
    let mut result = Vec::with_capacity(submaterials_ordered.len());

    if !module_is_unlocked {
        for sub in submaterials_ordered {
            result.push(SubmaterialGateStatus {
                submaterial_id: sub.submaterial_id,
                is_unlocked: false,
                is_completed: completed_submaterial_ids.contains(&sub.submaterial_id),
            });
        }
        return result;
    }

    let mut prev_completed = true; // first submaterial is always unlocked if module is unlocked

    for sub in submaterials_ordered {
        let is_completed = completed_submaterial_ids.contains(&sub.submaterial_id);
        let is_unlocked = prev_completed;

        result.push(SubmaterialGateStatus {
            submaterial_id: sub.submaterial_id,
            is_unlocked,
            is_completed,
        });

        prev_completed = is_completed && is_unlocked;
    }

    result
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn uuid(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    // ── compute_module_gates ──────────────────────────────────────────────

    #[test]
    fn module_first_always_unlocked() {
        let modules = vec![ModuleInfo {
            module_id: uuid(1),
            order_index: 1,
        }];
        let completed = HashSet::new();
        let gates = compute_module_gates(&modules, &completed);

        assert_eq!(gates.len(), 1);
        assert!(gates[0].is_unlocked);
        assert!(!gates[0].is_completed);
    }

    #[test]
    fn module_second_locked_when_first_not_completed() {
        let modules = vec![
            ModuleInfo { module_id: uuid(1), order_index: 1 },
            ModuleInfo { module_id: uuid(2), order_index: 2 },
        ];
        let completed = HashSet::new();
        let gates = compute_module_gates(&modules, &completed);

        assert!(gates[0].is_unlocked);
        assert!(!gates[0].is_completed);
        assert!(!gates[1].is_unlocked);
        assert!(!gates[1].is_completed);
    }

    #[test]
    fn module_second_unlocked_when_first_completed() {
        let modules = vec![
            ModuleInfo { module_id: uuid(1), order_index: 1 },
            ModuleInfo { module_id: uuid(2), order_index: 2 },
        ];
        let mut completed = HashSet::new();
        completed.insert(uuid(1));

        let gates = compute_module_gates(&modules, &completed);

        assert!(gates[0].is_unlocked);
        assert!(gates[0].is_completed);
        assert!(gates[1].is_unlocked);
        assert!(!gates[1].is_completed);
    }

    #[test]
    fn module_sequential_chain_three_modules() {
        let modules = vec![
            ModuleInfo { module_id: uuid(1), order_index: 1 },
            ModuleInfo { module_id: uuid(2), order_index: 2 },
            ModuleInfo { module_id: uuid(3), order_index: 3 },
        ];
        // Only module 1 completed, module 2 not completed
        let mut completed = HashSet::new();
        completed.insert(uuid(1));

        let gates = compute_module_gates(&modules, &completed);

        assert!(gates[0].is_unlocked);
        assert!(gates[0].is_completed);
        assert!(gates[1].is_unlocked);
        assert!(!gates[1].is_completed);
        assert!(!gates[2].is_unlocked); // module 3 locked because module 2 not completed
        assert!(!gates[2].is_completed);
    }

    // ── compute_submaterial_gates ─────────────────────────────────────────

    #[test]
    fn sub_all_locked_when_module_locked() {
        let subs = vec![
            SubmaterialInfo { submaterial_id: uuid(10), order_index: 1 },
            SubmaterialInfo { submaterial_id: uuid(11), order_index: 2 },
        ];
        let completed = HashSet::new();
        let gates = compute_submaterial_gates(false, &subs, &completed);

        assert!(!gates[0].is_unlocked);
        assert!(!gates[1].is_unlocked);
    }

    #[test]
    fn sub_first_unlocked_when_module_unlocked() {
        let subs = vec![
            SubmaterialInfo { submaterial_id: uuid(10), order_index: 1 },
            SubmaterialInfo { submaterial_id: uuid(11), order_index: 2 },
        ];
        let completed = HashSet::new();
        let gates = compute_submaterial_gates(true, &subs, &completed);

        assert!(gates[0].is_unlocked);
        assert!(!gates[0].is_completed);
        assert!(!gates[1].is_unlocked);
        assert!(!gates[1].is_completed);
    }

    #[test]
    fn sub_second_unlocked_when_first_completed() {
        let subs = vec![
            SubmaterialInfo { submaterial_id: uuid(10), order_index: 1 },
            SubmaterialInfo { submaterial_id: uuid(11), order_index: 2 },
            SubmaterialInfo { submaterial_id: uuid(12), order_index: 3 },
        ];
        let mut completed = HashSet::new();
        completed.insert(uuid(10));

        let gates = compute_submaterial_gates(true, &subs, &completed);

        assert!(gates[0].is_unlocked);
        assert!(gates[0].is_completed);
        assert!(gates[1].is_unlocked);
        assert!(!gates[1].is_completed);
        assert!(!gates[2].is_unlocked);
        assert!(!gates[2].is_completed);
    }

    // ── Progress calculation tests ────────────────────────────────────────

    #[test]
    fn progress_zero_percent() {
        let total = 6.0_f64;
        let done = 0.0_f64;
        let pct = (done / total * 100.0).round();
        assert_eq!(pct, 0.0);
    }

    #[test]
    fn progress_fifty_percent() {
        let total = 6.0_f64;
        let done = 3.0_f64;
        let pct = (done / total * 100.0).round();
        assert_eq!(pct, 50.0);
    }

    #[test]
    fn progress_hundred_percent() {
        let total = 6.0_f64;
        let done = 6.0_f64;
        let pct = (done / total * 100.0).round();
        assert_eq!(pct, 100.0);
    }

    #[test]
    fn progress_rounding() {
        // 1/3 = 33.333... should round to 33.0
        let total = 3.0_f64;
        let done = 1.0_f64;
        let pct = (done / total * 100.0).round();
        assert_eq!(pct, 33.0);

        // 2/3 = 66.666... should round to 67.0
        let done2 = 2.0_f64;
        let pct2 = (done2 / total * 100.0).round();
        assert_eq!(pct2, 67.0);
    }
}
