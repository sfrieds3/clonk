use colony_runtime_spike::{Runtime, Snapshot};
fn main() {
    let mut runtime = Runtime::new();
    let snapshot = Snapshot {
        supply_available: true,
        ..Snapshot::default()
    };
    let source = r#"
        fn supply_workshop(amount) { this.keep_stock("foundry:ore", amount); }
        fn production(amount) { this.supply_workshop(amount); }
        colony.production(20);
    "#;
    runtime.activate(1, source, &snapshot).unwrap();
    runtime.advance();
    runtime.evaluate(&snapshot).unwrap();
    println!(
        "revision {:?}, goals {:?}, jobs {:?}",
        runtime.active_revision(),
        runtime.goals,
        runtime.jobs
    );
}
