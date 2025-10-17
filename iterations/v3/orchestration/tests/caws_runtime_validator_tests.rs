use orchestration::caws_runtime::*;

#[tokio::test]
async fn default_validator_flags_violations() {
    let validator = DefaultValidator;
    let spec = WorkingSpec { risk_tier: 2, scope_in: vec!["src/".into()], change_budget_max_files: 2, change_budget_max_loc: 10 };
    let desc = TaskDescriptor { task_id: "T-X".into(), scope_in: vec!["src/".into()], risk_tier: 2 };
    let diff = DiffStats { files_changed: 3, lines_changed: 20, touched_paths: vec!["src/app.rs".into(), "other/outside.rs".into()] };
    let res = validator.validate(&spec, &desc, &diff, false, false, vec![]).await.unwrap();
    assert!(!res.snapshot.within_budget);
    assert!(!res.snapshot.within_scope);
    assert_eq!(res.violations.len(), 4);
}

