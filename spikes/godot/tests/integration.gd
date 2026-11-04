extends SceneTree

var bridge
var failures: Array[String] = []
const VALID = 'fn supply(n) { this.keep_stock("ore", n); }\ncolony.supply(%d);'

class DisconnectedBridge:
	extends RefCounted
	var polls := 0
	func poll() -> Dictionary:
		polls += 1
		return {"disconnected": true}
	func is_disconnected() -> bool:
		return true
	func activate(_revision: int, _source: String) -> bool:
		return false
	func tick() -> bool:
		return false

func _initialize() -> void:
	call_deferred("run")

func check(condition: bool, message: String) -> void:
	if not condition:
		failures.append(message)
		push_error(message)

func receive() -> Dictionary:
	var deadline := Time.get_ticks_msec() + 5000
	while Time.get_ticks_msec() < deadline:
		var result: Dictionary = bridge.poll()
		if not result.is_empty():
			return result
		await create_timer(0.005).timeout
	return {}

func run() -> void:
	check(ClassDB.class_exists("ColonyBridge"), "extension unavailable")
	if not ClassDB.class_exists("ColonyBridge"):
		quit(1)
		return
	bridge = ClassDB.instantiate("ColonyBridge")
	check(bridge.activate(0, 'throw "bad initial policy";'), "initial failure enqueue")
	var initial_failure := await receive()
	check(initial_failure.get("status") != "OK" and initial_failure.get("revision") == -1 and initial_failure.get("worker_id") == 2 and initial_failure.get("stockpile_id") == 1 and initial_failure.get("workshop_id") == 3, "first failed activation supplies real snapshot")
	check(bridge.activate(1, VALID % 20), "activation enqueue")
	var initial := await receive()
	check(initial.get("status") == "OK" and initial.get("job_count") == 1, "initial helper goal")
	check(bridge.tick(), "tick enqueue")
	var step := await receive()
	check(step.get("progress") == 1 and step.get("job_id") == initial.get("job_id"), "stable first step")
	check(bridge.activate(2, VALID % 25), "update enqueue")
	var updated := await receive()
	check(updated.get("revision") == 2 and updated.get("job_count") == 1 and updated.get("progress") == 1 and updated.get("job_id") == step.get("job_id"), "compatible update preserves progress")
	check(bridge.activate(3, 'colony.keep_stock("new", 4); throw "late failure";'), "late failure enqueue")
	var rejected := await receive()
	check(rejected.get("status") != "OK" and rejected.get("revision") == 2 and rejected.get("job_id") == step.get("job_id") and rejected.get("job_count") == 1, "late failure atomic")
	check(bridge.activate(4, 'loop { }'), "runaway enqueue")
	var frames := 0
	var deadline := Time.get_ticks_msec() + 5000
	var runaway := {}
	while Time.get_ticks_msec() < deadline:
		runaway = bridge.poll()
		if not runaway.is_empty():
			break
		frames += 1
		await process_frame
	check(not runaway.is_empty() and runaway.get("status") != "OK" and runaway.get("revision") == 2, "runaway rejected")
	check(frames > 0, "engine frames ran while runaway request pending")
	check(bridge.tick(), "post-rejection tick enqueue")
	var resumed := await receive()
	check(resumed.get("progress") == 2 and resumed.get("revision") == 2, "accepted policy resumes")
	for i in range(3):
		check(bridge.tick(), "stock tick enqueue")
		resumed = await receive()
	check(resumed.get("stock") == 1 and resumed.get("target") == 25, "synthetic stock tracks fake executor")
	var scene = load("res://main.tscn").instantiate()
	root.add_child(scene)
	await process_frame
	scene.world.player.position = scene.world.worker.position + Vector3(0, 1, 0)
	scene.select_nearby()
	check(scene.selected == "worker" and "unavailable" in scene.inspect.text and "#0" not in scene.inspect.text, "inspection before snapshot is unavailable")
	scene.world.player.position = Vector3(0, 1, 6)
	scene.editor.grab_focus()
	await process_frame
	check(scene.editor.has_focus(), "editor acquires focus")
	var before: Vector3 = scene.world.player.position
	var key := InputEventKey.new()
	key.keycode = KEY_D
	key.pressed = true
	Input.parse_input_event(key)
	await physics_frame
	await physics_frame
	await physics_frame
	key = InputEventKey.new()
	key.keycode = KEY_D
	key.pressed = false
	Input.parse_input_event(key)
	check(absf(scene.world.player.position.x - before.x) < 0.001, "focused editor suppresses movement")
	scene.editor.release_focus()
	key = InputEventKey.new()
	key.keycode = KEY_D
	key.pressed = true
	Input.parse_input_event(key)
	await physics_frame
	await physics_frame
	await physics_frame
	key = InputEventKey.new()
	key.keycode = KEY_D
	key.pressed = false
	Input.parse_input_event(key)
	check(scene.world.player.position.x > before.x, "movement resumes after leaving editor")
	scene.editor.text = VALID % 25
	scene.apply_policy()
	deadline = Time.get_ticks_msec() + 5000
	while scene.pending != "" and Time.get_ticks_msec() < deadline:
		await physics_frame
	check(scene.snapshot.get("revision", -1) == 1, "scene policy accepted")
	scene.world.player.position = scene.world.worker.position + Vector3(0, 1, 0)
	scene.select_nearby()
	check(scene.selected == "worker", "nearby worker selectable")
	var selected_id: int = scene.snapshot.worker_id
	scene.world.worker.position += Vector3(1, 0, 0)
	check(scene.selected == "worker" and scene.snapshot.worker_id == selected_id, "selection retains stable identity")
	var saved_stock: int = scene.snapshot.stock
	scene.world.player.position = Vector3(20, -10, 0)
	scene.world.update_motion(0.016, Vector3.ZERO)
	check(scene.world.player.position.distance_to(Vector3(0, 1, 6)) < 0.1 and scene.snapshot.stock == saved_stock, "out of bounds recovery preserves simulation")
	# Submit on the scene's own bridge; interaction is possible before its reply is consumed.
	scene.editor.text = 'loop { }'
	scene.apply_policy()
	check(scene.pending == "activate", "scene runaway pending")
	var previous_revision: int = scene.snapshot.revision
	var previous_progress: int = scene.snapshot.progress
	var pending_position: Vector3 = scene.world.player.position
	scene.world.update_motion(0.1, Vector3.RIGHT)
	check(scene.world.player.position.x > pending_position.x, "movement while scene request pending")
	scene.world.player.position = scene.world.worker.position + Vector3(0, 1, 0)
	scene.select_nearby()
	check(scene.selected == "worker" and scene.snapshot.revision == previous_revision and scene.snapshot.progress == previous_progress, "selection during scene pending preserves snapshot")
	deadline = Time.get_ticks_msec() + 5000
	while scene.pending != "" and Time.get_ticks_msec() < deadline:
		await physics_frame
	check(scene.pending == "" and scene.snapshot.revision == previous_revision and scene.snapshot.progress == previous_progress, "scene runaway rejected without changing snapshot")
	var dead := DisconnectedBridge.new()
	scene.bridge = dead
	scene.pending = "tick"
	scene._physics_process(0.016)
	check(scene.pending == "" and scene.worker_unavailable and "unavailable" in scene.status.text, "disconnect clears pending and reports unavailable")
	scene._physics_process(1.0)
	check(dead.polls == 1, "disconnected worker is not retried")
	scene.apply_policy()
	check("unavailable" in scene.diagnostic.text, "disconnected activation reports unavailable")
	scene.queue_free()
	bridge = null
	if failures.is_empty():
		print("PASS: real bridge, transactional updates, responsiveness, focus and selection")
	quit(0 if failures.is_empty() else 1)
