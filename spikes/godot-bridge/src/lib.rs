//! Trusted, disposable bridge. This is not a security sandbox.
use colony_runtime_spike::{Runtime, Snapshot};
use godot::prelude::*;
use std::cell::Cell;
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};
use std::thread::{self, JoinHandle};

struct Extension;
#[gdextension]
unsafe impl ExtensionLibrary for Extension {}

enum Request {
    Activate(u64, String),
    Tick,
}

struct Reply {
    revision: Option<u64>,
    request: &'static str,
    requested_revision: i64,
    status: String,
    tick: u64,
    stock: u32,
    target: u32,
    worker_id: u64,
    stockpile_id: u64,
    workshop_id: u64,
    job_id: u64,
    job_key: String,
    blocked: bool,
    progress: u32,
    job_count: usize,
}

fn worker(requests: Receiver<Request>, replies: SyncSender<Reply>) {
    let mut runtime = Runtime::new();
    let mut snapshot = Snapshot {
        supply_available: true,
        ..Snapshot::default()
    };
    let mut tick = 0_u64;
    while let Ok(request) = requests.recv() {
        let (kind, requested_revision) = match &request {
            Request::Activate(revision, _) => ("activate", *revision as i64),
            Request::Tick => ("tick", -1),
        };
        let result = match request {
            Request::Activate(revision, source) => runtime.activate(revision, &source, &snapshot),
            Request::Tick => {
                let result = runtime.evaluate(&snapshot);
                if result.is_ok() {
                    runtime.advance();
                    tick += 1;
                    // A deliberately synthetic single-stock fixture, not physical hauling.
                    if runtime
                        .jobs
                        .values()
                        .next()
                        .is_some_and(|j| !j.blocked && j.progress % 5 == 0)
                    {
                        snapshot.stock = snapshot.stock.saturating_add(1);
                    }
                    runtime.reconcile(&snapshot);
                }
                result
            }
        };
        let status = match result {
            Ok(()) => "OK".to_string(),
            Err(d) => format!("{} at {:?}: {}", d.stage, d.line, d.message),
        };
        let job = runtime.jobs.values().next();
        let goal = job
            .and_then(|j| runtime.goals.get(&j.key))
            .or_else(|| runtime.goals.values().next());
        let reply = Reply {
            revision: runtime.active_revision(),
            request: kind,
            requested_revision,
            status,
            tick,
            stock: snapshot.stock,
            target: goal.map_or(0, |g| g.minimum),
            worker_id: snapshot.worker_id,
            stockpile_id: snapshot.stockpile_id,
            workshop_id: snapshot.workshop_id,
            job_id: job.map_or(0, |j| j.id),
            job_key: job.map_or_else(String::new, |j| j.key.clone()),
            blocked: job.is_some_and(|j| j.blocked),
            progress: runtime.jobs.values().next().map_or(0, |job| job.progress),
            job_count: runtime.jobs.len(),
        };
        if replies.send(reply).is_err() {
            break;
        }
    }
}

// Owns no Godot objects; Runtime is created inside the worker, not moved across threads.
struct WorkerClient {
    requests: Option<SyncSender<Request>>,
    replies: Option<Receiver<Reply>>,
    thread: Option<JoinHandle<()>>,
    disconnected: Cell<bool>,
}

impl WorkerClient {
    fn spawn() -> Self {
        Self::spawn_with(worker)
    }

    fn spawn_with(run: impl FnOnce(Receiver<Request>, SyncSender<Reply>) + Send + 'static) -> Self {
        let (request_tx, request_rx) = mpsc::sync_channel(2);
        let (reply_tx, reply_rx) = mpsc::sync_channel(2);
        let thread = thread::spawn(move || run(request_rx, reply_tx));
        Self {
            requests: Some(request_tx),
            replies: Some(reply_rx),
            thread: Some(thread),
            disconnected: Cell::new(false),
        }
    }

    fn enqueue(&self, request: Request) -> bool {
        match self
            .requests
            .as_ref()
            .expect("client is live")
            .try_send(request)
        {
            Ok(()) => true,
            Err(TrySendError::Full(_)) => false,
            Err(TrySendError::Disconnected(_)) => {
                self.disconnected.set(true);
                false
            }
        }
    }

    fn poll(&self) -> Result<Reply, TryRecvError> {
        let result = self.replies.as_ref().expect("client is live").try_recv();
        if matches!(result, Err(TryRecvError::Disconnected)) {
            self.disconnected.set(true);
        }
        result
    }
}

impl Drop for WorkerClient {
    fn drop(&mut self) {
        // A worker may be blocked sending to a full reply queue or receiving a request.
        // Disconnect both ends before joining, in that order, so either wait can finish.
        drop(self.replies.take());
        drop(self.requests.take());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[derive(GodotClass)]
#[class(base=RefCounted)]
struct ColonyBridge {
    client: WorkerClient,
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for ColonyBridge {
    fn init(base: Base<RefCounted>) -> Self {
        Self {
            client: WorkerClient::spawn(),
            base,
        }
    }
}

#[godot_api]
impl ColonyBridge {
    #[func]
    fn activate(&self, revision: i64, source: GString) -> bool {
        if revision < 0 {
            return false;
        }
        self.client
            .enqueue(Request::Activate(revision as u64, source.to_string()))
    }

    #[func]
    fn tick(&self) -> bool {
        self.client.enqueue(Request::Tick)
    }

    #[func]
    fn is_disconnected(&self) -> bool {
        self.client.disconnected.get()
    }

    #[func]
    fn poll(&self) -> VarDictionary {
        let mut out = VarDictionary::new();
        let result = self.client.poll();
        if let Ok(reply) = result {
            out.set("revision", reply.revision.map_or(-1, |v| v as i64));
            out.set("request", reply.request);
            out.set("requested_revision", reply.requested_revision);
            out.set("status", reply.status);
            out.set("tick", reply.tick as i64);
            out.set("stock", i64::from(reply.stock));
            out.set("target", i64::from(reply.target));
            out.set("worker_id", reply.worker_id as i64);
            out.set("stockpile_id", reply.stockpile_id as i64);
            out.set("workshop_id", reply.workshop_id as i64);
            out.set("job_id", reply.job_id as i64);
            out.set("job_key", reply.job_key);
            out.set("blocked", reply.blocked);
            out.set("progress", i64::from(reply.progress));
            out.set("job_count", reply.job_count as i64);
        } else if matches!(result, Err(TryRecvError::Disconnected)) {
            out.set("disconnected", true);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const DEADLINE: Duration = Duration::from_secs(10);

    fn recv<T>(rx: &Receiver<T>) -> T {
        rx.recv_timeout(DEADLINE).expect("worker test timed out")
    }

    fn drop_with_deadline(client: WorkerClient) {
        let (done_tx, done_rx) = mpsc::channel();
        thread::spawn(move || {
            drop(client);
            done_tx.send(()).unwrap();
        });
        recv(&done_rx);
    }

    fn reply() -> Reply {
        Reply {
            revision: None,
            request: "tick",
            requested_revision: -1,
            status: String::new(),
            tick: 0,
            stock: 0,
            target: 0,
            worker_id: 0,
            stockpile_id: 0,
            workshop_id: 0,
            job_id: 0,
            job_key: String::new(),
            blocked: false,
            progress: 0,
            job_count: 0,
        }
    }

    fn request(client: &WorkerClient, request: Request) -> Reply {
        assert!(client.enqueue(request));
        recv(client.replies.as_ref().unwrap())
    }

    #[test]
    fn tick_reconciles_completion_and_idle_does_not_create_stock() {
        let client = WorkerClient::spawn();
        let initial = request(
            &client,
            Request::Activate(7, "colony.keep_stock(\"ore\", 1);".into()),
        );
        assert_eq!(
            (
                initial.request,
                initial.requested_revision,
                initial.revision
            ),
            ("activate", 7, Some(7))
        );
        let id = initial.job_id;
        for step in 1..=5 {
            let reply = request(&client, Request::Tick);
            assert_eq!(
                (
                    reply.request,
                    reply.requested_revision,
                    reply.revision,
                    reply.tick
                ),
                ("tick", -1, Some(7), step)
            );
            if step < 5 {
                assert_eq!(
                    (reply.stock, reply.job_id, reply.progress),
                    (0, id, step as u32)
                );
            } else {
                assert_eq!(
                    (reply.stock, reply.target, reply.job_count, reply.job_id),
                    (1, 1, 0, 0)
                );
            }
        }
        let idle = request(&client, Request::Tick);
        assert_eq!((idle.stock, idle.job_count, idle.tick), (1, 0, 6));
    }

    #[test]
    fn goal_replacement_and_failed_tick_are_transactional() {
        let client = WorkerClient::spawn();
        let policy =
            "if colony.stock() > 0 { throw \"stock changed\"; } colony.keep_stock(\"ore\", 2);";
        let first = request(&client, Request::Activate(10, policy.into()));
        let stepped = request(&client, Request::Tick);
        assert_eq!((stepped.job_id, stepped.progress), (first.job_id, 1));
        let replaced = request(
            &client,
            Request::Activate(11, "colony.keep_stock(\"wood\", 2);".into()),
        );
        assert_ne!(replaced.job_id, first.job_id);
        assert_eq!((replaced.job_key.as_str(), replaced.progress), ("wood", 0));
        let withdrawn = request(&client, Request::Activate(12, "".into()));
        assert_eq!(withdrawn.job_count, 0);
        let accepted = request(&client, Request::Activate(13, policy.into()));
        for _ in 0..5 {
            request(&client, Request::Tick);
        }
        let failed = request(&client, Request::Tick);
        assert_ne!(failed.status, "OK");
        assert_eq!(
            (failed.request, failed.requested_revision, failed.revision),
            ("tick", -1, Some(13))
        );
        assert_eq!(
            (failed.tick, failed.stock, failed.job_id, failed.progress),
            (6, 1, accepted.job_id, 5)
        );
    }

    #[test]
    fn exited_worker_reports_disconnection_for_poll_and_enqueue() {
        let (done_tx, done_rx) = mpsc::channel();
        let mut client = WorkerClient::spawn_with(move |_, _| {
            done_tx.send(()).unwrap();
        });
        recv(&done_rx);
        client.thread.take().unwrap().join().unwrap();
        assert!(!client.enqueue(Request::Tick));
        assert!(client.disconnected.get());
        client.disconnected.set(false);
        assert!(matches!(client.poll(), Err(TryRecvError::Disconnected)));
        assert!(client.disconnected.get());
    }

    #[test]
    fn idle_drop_waits_for_worker_exit() {
        let (disconnected_tx, disconnected_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let client = WorkerClient::spawn_with(move |requests, _| {
            assert!(requests.recv().is_err());
            disconnected_tx.send(()).unwrap();
            recv(&release_rx);
        });
        thread::spawn(move || {
            drop(client);
            done_tx.send(()).unwrap();
        });
        recv(&disconnected_rx);
        assert!(matches!(
            done_rx.recv_timeout(Duration::from_millis(100)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        release_tx.send(()).unwrap();
        recv(&done_rx);
    }

    #[test]
    fn queued_requests_disconnect_before_join() {
        let (ready_tx, ready_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let client = WorkerClient::spawn_with(move |requests, replies| {
            ready_tx.send(()).unwrap();
            recv(&release_rx);
            worker(requests, replies);
        });
        recv(&ready_rx);
        assert!(client.enqueue(Request::Tick));
        assert!(client.enqueue(Request::Tick));
        assert!(!client.enqueue(Request::Tick));
        release_tx.send(()).unwrap();
        drop_with_deadline(client);
    }

    #[test]
    fn full_reply_queue_unblocks_on_drop() {
        let (full_tx, full_rx) = mpsc::channel();
        let client = WorkerClient::spawn_with(move |_, replies| {
            replies.send(reply()).unwrap();
            replies.send(reply()).unwrap();
            full_tx.send(()).unwrap();
            assert!(replies.send(reply()).is_err());
        });
        recv(&full_rx);
        drop_with_deadline(client);
    }

    #[test]
    fn disconnected_reply_receiver_stops_worker() {
        let (done_tx, done_rx) = mpsc::channel();
        let mut client = WorkerClient::spawn_with(move |requests, replies| {
            worker(requests, replies);
            done_tx.send(()).unwrap();
        });
        drop(client.replies.take());
        assert!(client.enqueue(Request::Tick));
        recv(&done_rx);
        assert!(!client.enqueue(Request::Tick));
        drop_with_deadline(client);
    }

    #[test]
    fn bounded_runaway_evaluation_can_finish_during_teardown() {
        let (started_tx, started_rx) = mpsc::channel();
        let client = WorkerClient::spawn_with(move |requests, replies| {
            let request = requests.recv().unwrap();
            started_tx.send(()).unwrap();
            let mut runtime = Runtime::new();
            let Request::Activate(revision, source) = request else {
                panic!("expected activation")
            };
            let result = runtime.activate(revision, &source, &Snapshot::default());
            assert!(result.is_err(), "runaway must hit the operation limit");
            assert!(requests.recv().is_err());
            assert!(replies.send(reply()).is_err());
        });
        assert!(client.enqueue(Request::Activate(1, "loop { }".into())));
        recv(&started_rx);
        drop_with_deadline(client);
    }

    #[test]
    fn repeated_real_worker_creation_and_drop() {
        for _ in 0..50 {
            drop_with_deadline(WorkerClient::spawn());
        }
    }
}
