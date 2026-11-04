use colony_runtime_spike::{Runtime, Snapshot};
const PROGRAM: &str = r#"
fn supply(key, amount) { this.keep_stock(key, amount); }
fn outer(amount) { this.supply("ore", amount); }
colony.outer(10);
"#;
#[test]
fn nested_binding_and_progress() {
    let mut rt = Runtime::new();
    let snap = Snapshot {
        supply_available: true,
        ..Snapshot::default()
    };
    rt.activate(1, PROGRAM, &snap).unwrap();
    rt.advance();
    let job = rt.jobs["ore"].clone();
    rt.evaluate(&snap).unwrap();
    rt.activate(2, PROGRAM, &snap).unwrap();
    assert_eq!(rt.jobs.len(), 1);
    assert_eq!(rt.jobs["ore"], job);
    assert_eq!(rt.goals["ore"].revision, 2);
}
#[test]
fn errors_are_transactional() {
    let mut rt = Runtime::new();
    let snap = Snapshot::default();
    rt.activate(1, PROGRAM, &snap).unwrap();
    let goals = rt.goals.clone();
    let jobs = rt.jobs.clone();
    for bad in [
        "colony.keep_stock(\"new\", 5); throw \"oops\";",
        "colony.keep_stock(\"new\", -1);",
        "fn stock(c) {}",
        "fn dup(c) {} fn dup(c, x) {}",
        "colony.keep_stock(\"a\", 1); colony.keep_stock(\"a\", 2);",
        "colony.keep_stock(\"a\", true);",
        "fn bad( {",
        "while true {}",
        "fn recurse() { this.recurse(); } colony.recurse();",
        "import \"foo\";",
        "read_file(\"/etc/passwd\");",
        "colony.keep_stock(\"a\", 1); let a = [0; 10000];",
        "eval(\"1\");",
    ] {
        assert!(rt.activate(2, bad, &snap).is_err(), "accepted {bad}");
        assert_eq!(rt.active_revision(), Some(1));
        assert_eq!(rt.goals, goals);
        assert_eq!(rt.jobs, jobs);
    }
}
#[test]
fn snapshot_and_failed_later_evaluation() {
    for error in [
        "throw \"late\";",
        "colony.missing();",
        "let x = 1 / 0;",
        "colony.keep_stock(\"bad\", true);",
    ] {
        let mut rt = Runtime::new();
        let mut snap = Snapshot::default();
        let original = snap.clone();
        let source = format!("colony.keep_stock(\"ore\", 5); if colony.stock() > 0 {{ {error} }}");
        rt.activate(3, &source, &snap).unwrap();
        assert_eq!(snap, original);
        let goals = rt.goals.clone();
        let jobs = rt.jobs.clone();
        snap.stock = 1;
        assert!(rt.evaluate(&snap).is_err());
        assert_eq!(rt.active_revision(), Some(3));
        assert_eq!(rt.goals, goals);
        assert_eq!(rt.jobs, jobs);
        snap.stock = 0;
        rt.evaluate(&snap).unwrap();
        assert_eq!(rt.goals, goals);
        assert_eq!(rt.jobs, jobs);
    }
}
#[test]
fn source_and_goal_limits() {
    let mut rt = Runtime::new();
    let snap = Snapshot::default();
    assert!(rt.activate(1, &" ".repeat(20_000), &snap).is_err());
    assert!(rt
        .activate(
            1,
            "for x in 0..65 { colony.keep_stock(x.to_string(), 1); }",
            &snap
        )
        .is_err());
    assert!(rt
        .activate(
            1,
            "try { colony.keep_stock(\"bad\", -1); } catch(e) {}",
            &snap
        )
        .is_err());
}

#[test]
fn duplicate_signature_and_budget_cannot_be_caught() {
    let mut rt = Runtime::new();
    let snap = Snapshot::default();
    let duplicate = rt.activate(1, "fn same() {} fn same() {}", &snap);
    assert!(
        duplicate.is_err(),
        "duplicate same-signature definitions accepted"
    );
    let loop_source = "colony.keep_stock(\"first\", 1); try { while true {} } catch(e) {} colony.keep_stock(\"second\", 2);";
    assert!(rt.activate(2, loop_source, &snap).is_err());
    assert!(rt.goals.is_empty());
}

#[test]
fn caught_errors_reject_entire_candidate() {
    let mut rt = Runtime::new();
    let snap = Snapshot::default();
    rt.activate(1, PROGRAM, &snap).unwrap();
    let goals = rt.goals.clone();
    let jobs = rt.jobs.clone();
    for error in [
        "colony.keep_stock(\"bad\", true);",
        "colony.missing();",
        "let x = 1 / 0;",
        "throw \"oops\";",
    ] {
        let source = format!("colony.keep_stock(\"first\", 1); try {{ {error} }} catch(e) {{}}");
        assert!(rt.activate(2, &source, &snap).is_err(), "accepted {source}");
        assert_eq!(rt.active_revision(), Some(1));
        assert_eq!(rt.goals, goals);
        assert_eq!(rt.jobs, jobs);
    }
}

#[test]
fn receiver_and_commands_cannot_be_tampered_with() {
    let snap = Snapshot::default();
    for source in [
        "colony.__host = (); colony.keep_stock(\"x\", 1);",
        "colony[\"__host\"] = ();",
        "colony.remove(\"__host\");",
        "fn helper() { this.keep_stock(\"x\", 1); } colony[\"helper\"] = Fn(\"stock\");",
        "fn __host() {} colony.keep_stock(\"x\", 1);",
        "fn helper() { this.keep_stock(\"x\", 1); } colony.helper = Fn(\"stock\"); colony.helper();",
        "fn helper() { this.keep_stock(\"x\", 1); } colony.remove(\"helper\"); colony.helper();",
        "fn helper() { this.helper = Fn(\"stock\"); } colony.helper();",
        "colony = #{}; colony.keep_stock(\"x\", 1);",
        "let copy = colony; copy.__host = ();",
        "fn helper() { this.keep_stock(\"x\", 1); } let copy = colony; copy.helper = Fn(\"stock\"); copy.helper();",
        "fn helper(colony) { colony.keep_stock(\"x\", 1); } colony.helper(#{});",
        "let colony = #{}; colony.keep_stock(\"x\", 1);",
    ] {
        let mut rt = Runtime::new();
        assert!(rt.activate(1, source, &snap).is_err(), "accepted {source}");
    }
}

#[test]
fn satisfied_goal_drops_work_and_new_deficit_gets_new_job() {
    let mut rt = Runtime::new();
    let mut snap = Snapshot {
        supply_available: true,
        ..Snapshot::default()
    };
    rt.activate(1, "colony.keep_stock(\"ore\", 5);", &snap)
        .unwrap();
    let id = rt.jobs["ore"].id;
    snap.stock = 5;
    rt.evaluate(&snap).unwrap();
    assert!(rt.jobs.is_empty());
    rt.advance();
    snap.stock = 0;
    rt.evaluate(&snap).unwrap();
    assert_ne!(rt.jobs["ore"].id, id);
    assert_eq!(rt.jobs["ore"].progress, 0);
}

#[test]
fn later_tampering_failure_preserves_active_state() {
    let mut rt = Runtime::new();
    let mut snap = Snapshot::default();
    rt.activate(
        4,
        r#"colony.keep_stock("ore", 5); if colony.stock() > 0 { colony.__host = (); }"#,
        &snap,
    )
    .unwrap();
    let goals = rt.goals.clone();
    let jobs = rt.jobs.clone();
    snap.stock = 1;
    assert!(rt.evaluate(&snap).is_err());
    assert_eq!(rt.active_revision(), Some(4));
    assert_eq!(rt.goals, goals);
    assert_eq!(rt.jobs, jobs);
    snap.stock = 0;
    rt.evaluate(&snap).unwrap();
    assert_eq!(rt.goals, goals);
    assert_eq!(rt.jobs, jobs);
}
