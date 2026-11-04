# Godot bridge spike

The bridge owns a worker thread and bounded request and reply queues. Dropping the bridge disconnects the reply receiver first, then the request sender, and joins the worker. This releases a worker blocked on a full reply queue or an idle request receive; queued work may be abandoned during teardown. No Godot handles cross threads, and the runtime is constructed on the worker thread.

The nonblocking `activate`, `tick`, and `poll` API reports request kind, requested and active revision, diagnostic, fixture tick, stock and target, stable fixture IDs, first job identity/key/progress, and job count. The bridge owns the copied stock snapshot; each fifth unblocked progress step of the first fixture job increases its synthetic stock by one. This makes the display observable but does not model resource conservation, physical workers, routing, or multiple-goal allocation. The scene cannot change stock or complete jobs.

Joining waits for any in-progress runtime call to return. Rhai's operation limit bounds the runaway script exercised by the tests, but does not guarantee that arbitrary native callbacks terminate; a blocked callback can therefore block bridge destruction indefinitely. This is a trusted spike, not a sandbox or a cancellation mechanism.

A separate fresh-import crash occurs in the unpatched official Godot 4.7.2 editor, even for a minimal registered class. A local source build with upstream deferred-documentation-callback fix has passed controlled fresh-import tests; see the external engine spike evidence. The worker lifecycle is not the engine fix and cached imports must not be treated as one.
