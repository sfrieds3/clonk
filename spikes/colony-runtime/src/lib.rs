//! Disposable, engine-independent fixture. Not a security sandbox or production scheduler.
use rhai::{
    Dynamic, Engine, EvalAltResult, ImmutableString, NativeCallContext, Position, Scope, AST, INT,
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

const MAX_SOURCE: usize = 16_384;
const MAX_GOALS: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub stockpile_id: u64,
    pub worker_id: u64,
    pub workshop_id: u64,
    pub stock: u32,
    pub supply_available: bool,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            stockpile_id: 1,
            worker_id: 2,
            workshop_id: 3,
            stock: 0,
            supply_available: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Goal {
    pub key: String,
    pub minimum: u32,
    pub revision: u64,
    pub line: Option<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Job {
    pub id: u64,
    pub key: String,
    pub progress: u32,
    pub blocked: bool,
    pub revision: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub revision: u64,
    pub stage: &'static str,
    pub message: String,
    pub line: Option<usize>,
}

#[derive(Clone)]
struct Colony {
    snapshot: Snapshot,
    batch: Rc<RefCell<Batch>>,
    revision: u64,
}
#[derive(Default)]
struct Batch {
    goals: BTreeMap<String, Goal>,
    invalid: bool,
}
fn keep_stock(
    ctx: NativeCallContext,
    colony: &mut Colony,
    key: ImmutableString,
    minimum: INT,
) -> Result<(), Box<EvalAltResult>> {
    let mut batch = colony.batch.borrow_mut();
    let valid = !key.is_empty()
        && key.len() <= 64
        && (0..=10_000).contains(&minimum)
        && batch.goals.len() < MAX_GOALS
        && !batch.goals.contains_key(key.as_str());
    if !valid {
        batch.invalid = true;
        return Err(EvalAltResult::ErrorRuntime(
            "invalid or duplicate goal".into(),
            ctx.call_position(),
        )
        .into());
    }
    let goal = Goal {
        key: key.to_string(),
        minimum: minimum as u32,
        revision: colony.revision,
        line: ctx.call_position().line(),
    };
    batch.goals.insert(goal.key.clone(), goal);
    Ok(())
}

fn engine() -> Engine {
    let mut e = Engine::new();
    e.set_max_operations(20_000)
        .set_max_call_levels(32)
        .set_max_expr_depths(48, 48)
        .set_max_variables(128)
        .set_max_functions(64)
        .set_max_string_size(4096)
        .set_max_array_size(128)
        .set_max_map_size(128);
    e.disable_symbol("eval").disable_symbol("try");
    #[allow(deprecated)]
    e.on_def_var(|_, info, _| Ok(info.name() != "colony"));
    e.on_print(|_| {}).on_debug(|_, _, _| {});
    e.register_type_with_name::<Colony>("Colony");
    e.register_fn("keep_stock", keep_stock);
    e.register_fn("stock", |c: &mut Colony| -> INT { c.snapshot.stock.into() });
    e
}
fn diagnostic(revision: u64, stage: &'static str, message: String, pos: Position) -> Diagnostic {
    Diagnostic {
        revision,
        stage,
        message: message.chars().take(512).collect(),
        line: pos.line(),
    }
}

pub struct Runtime {
    engine: Engine,
    active: Option<(u64, AST)>,
    pub goals: BTreeMap<String, Goal>,
    pub jobs: BTreeMap<String, Job>,
    next_job: u64,
}
impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
impl Runtime {
    pub fn new() -> Self {
        Self {
            engine: engine(),
            active: None,
            goals: BTreeMap::new(),
            jobs: BTreeMap::new(),
            next_job: 1,
        }
    }
    pub fn active_revision(&self) -> Option<u64> {
        self.active.as_ref().map(|x| x.0)
    }
    /// Call at a host tick boundary. On failure neither revision nor goals/jobs change.
    pub fn activate(
        &mut self,
        revision: u64,
        source: &str,
        snapshot: &Snapshot,
    ) -> Result<(), Diagnostic> {
        if source.len() > MAX_SOURCE {
            return Err(diagnostic(
                revision,
                "source",
                "source too large".into(),
                Position::NONE,
            ));
        }
        let ast = self
            .engine
            .compile(source)
            .map_err(|e| diagnostic(revision, "compile", e.to_string(), e.position()))?;
        let mut names = BTreeSet::new();
        for f in ast.iter_functions() {
            if matches!(f.name, "keep_stock" | "stock" | "__host")
                || !names.insert(f.name.to_owned())
            {
                return Err(diagnostic(
                    revision,
                    "registration",
                    format!("reserved or duplicate command: {}", f.name),
                    Position::NONE,
                ));
            }
        }
        let batch = self.evaluate_ast(revision, &ast, snapshot)?;
        self.active = Some((revision, ast));
        self.commit(batch, snapshot);
        Ok(())
    }
    pub fn evaluate(&mut self, snapshot: &Snapshot) -> Result<(), Diagnostic> {
        let (revision, ast) = self.active.as_ref().ok_or_else(|| {
            diagnostic(0, "evaluation", "no active program".into(), Position::NONE)
        })?;
        let batch = self.evaluate_ast(*revision, ast, snapshot)?;
        self.commit(batch, snapshot);
        Ok(())
    }
    fn evaluate_ast(
        &self,
        revision: u64,
        ast: &AST,
        snapshot: &Snapshot,
    ) -> Result<BTreeMap<String, Goal>, Diagnostic> {
        let batch = Rc::new(RefCell::new(Batch::default()));
        let colony = Colony {
            snapshot: snapshot.clone(),
            batch: batch.clone(),
            revision,
        };
        let mut scope = Scope::new();
        scope.push_constant("colony", colony);
        let _ = self
            .engine
            .eval_ast_with_scope::<Dynamic>(&mut scope, ast)
            .map_err(|e| diagnostic(revision, "evaluation", e.to_string(), e.position()))?;
        let mut batch = batch.borrow_mut();
        if batch.invalid {
            return Err(diagnostic(
                revision,
                "evaluation",
                "invalid goal even if caught".into(),
                Position::NONE,
            ));
        }
        Ok(std::mem::take(&mut batch.goals))
    }
    fn commit(&mut self, batch: BTreeMap<String, Goal>, snapshot: &Snapshot) {
        self.jobs.retain(|key, _| {
            batch
                .get(key)
                .is_some_and(|goal| goal.minimum > snapshot.stock)
        });
        for goal in batch.values() {
            if goal.minimum > snapshot.stock {
                self.jobs
                    .entry(goal.key.clone())
                    .or_insert_with(|| {
                        let id = self.next_job;
                        self.next_job += 1;
                        Job {
                            id,
                            key: goal.key.clone(),
                            progress: 0,
                            blocked: !snapshot.supply_available,
                            revision: goal.revision,
                        }
                    })
                    .blocked = !snapshot.supply_available;
            }
        }
        self.goals = batch;
    }
    /// Reconcile the last accepted goals against a changed host snapshot without running policy code.
    pub fn reconcile(&mut self, snapshot: &Snapshot) {
        self.commit(self.goals.clone(), snapshot);
    }
    /// One fake physical step; it does not mutate the snapshot or model resource conservation.
    pub fn advance(&mut self) {
        for job in self.jobs.values_mut() {
            if !job.blocked {
                job.progress = job.progress.saturating_add(1);
            }
        }
    }
}
