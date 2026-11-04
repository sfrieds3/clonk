extends Node3D

const Settlement = preload("res://settlement.gd")
const SAMPLE = 'fn supply(amount) { this.keep_stock("ore", amount); }\ncolony.supply(20);'
var world: Node3D
var editor: CodeEdit
var status: Label
var diagnostic: Label
var inspect: Label
var bridge
var revision := 0
var active_source := ""
var submitted_source := ""
var pending := ""
var snapshot := {}
var timer := 0.0
var selected := ""
var last_diagnostic := "No policy applied yet."
var worker_unavailable := false

func _ready() -> void:
	world = Settlement.new()
	add_child(world)
	build_ui()
	if ClassDB.class_exists("ColonyBridge"):
		bridge = ClassDB.instantiate("ColonyBridge")
	else:
		worker_unavailable = true
		last_diagnostic = "Bridge unavailable: build the extension first."
	refresh_ui()

func ui_box(color: Color, padding: float) -> StyleBoxFlat:
	var box := StyleBoxFlat.new()
	box.bg_color = color
	box.set_corner_radius_all(5)
	box.set_content_margin_all(padding)
	return box

func build_ui() -> void:
	var canvas := CanvasLayer.new()
	add_child(canvas)
	var theme := Theme.new()
	theme.default_font = load("res://fonts/Lato-Regular.ttf")
	theme.default_font_size = 15
	var panel := PanelContainer.new()
	panel.theme = theme
	panel.position = Vector2(20, 20)
	panel.size = Vector2(460, 0)
	panel.add_theme_stylebox_override("panel", ui_box(Color(0.10, 0.14, 0.13, 0.92), 14))
	canvas.add_child(panel)
	var layout := VBoxContainer.new()
	layout.add_theme_constant_override("separation", 8)
	panel.add_child(layout)
	var title := Label.new()
	title.text = "C L O N K   /   COLONY POLICY"
	title.add_theme_font_size_override("font_size", 18)
	layout.add_child(title)
	status = Label.new()
	status.add_theme_color_override("font_color", Color("c5cfc1"))
	layout.add_child(status)
	editor = CodeEdit.new()
	editor.custom_minimum_size = Vector2(432, 170)
	editor.set_line_wrapping_mode(TextEdit.LINE_WRAPPING_BOUNDARY)
	editor.add_theme_font_override("font", load("res://fonts/SourceCodePro-Regular.ttf"))
	editor.add_theme_font_size_override("font_size", 14)
	editor.add_theme_stylebox_override("normal", ui_box(Color("202b29"), 10))
	editor.add_theme_stylebox_override("focus", ui_box(Color("263633"), 10))
	editor.text = SAMPLE
	editor.text_changed.connect(refresh_ui)
	layout.add_child(editor)
	var apply := Button.new()
	apply.text = "Apply policy   Ctrl + Enter"
	apply.focus_mode = Control.FOCUS_NONE
	apply.pressed.connect(apply_policy)
	layout.add_child(apply)
	diagnostic = Label.new()
	diagnostic.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	layout.add_child(diagnostic)
	var inspect_panel := PanelContainer.new()
	inspect_panel.theme = theme
	inspect_panel.position = Vector2(20, 385)
	inspect_panel.add_theme_stylebox_override("panel", ui_box(Color(0.10, 0.14, 0.13, 0.92), 12))
	canvas.add_child(inspect_panel)
	inspect = Label.new()
	inspect_panel.add_child(inspect)
	var help_panel := PanelContainer.new()
	help_panel.theme = theme
	help_panel.set_anchors_preset(Control.PRESET_BOTTOM_LEFT)
	help_panel.position = Vector2(20, -51)
	help_panel.add_theme_stylebox_override("panel", ui_box(Color(0.10, 0.14, 0.13, 0.92), 10))
	canvas.add_child(help_panel)
	var help := Label.new()
	help.text = "WASD move   •   E inspect nearby   •   F2 editor   •   Esc leave editor"
	help_panel.add_child(help)

func apply_policy() -> void:
	if bridge == null or worker_unavailable:
		last_diagnostic = "Bridge unavailable; draft was not submitted."
		refresh_ui()
		return
	if pending != "":
		last_diagnostic = "Bridge busy; draft was not submitted."
		refresh_ui()
		return
	var next := revision + 1
	if bridge.activate(next, editor.text):
		revision = next
		submitted_source = editor.text
		pending = "activate"
		last_diagnostic = "Validating revision %d…" % revision
	else:
		if bridge.is_disconnected():
			mark_worker_unavailable()
		else:
			last_diagnostic = "Request queue busy; draft was not submitted."
	refresh_ui()

func refresh_ui() -> void:
	if status == null:
		return
	var active: int = snapshot.get("revision", -1)
	var draft := "ACTIVE" if active >= 0 and editor.text == active_source else "UNAPPLIED DRAFT"
	status.text = "%s   |   active %s   |   %s" % [draft, active if active >= 0 else "none", "unavailable" if worker_unavailable else (pending if pending != "" else "ready")]
	diagnostic.text = last_diagnostic
	refresh_inspection()

func refresh_inspection() -> void:
	if inspect == null:
		return
	if selected == "":
		inspect.text = "APPROACH + E TO INSPECT\nFixture worker activity is an illustration, not physical routing."
		return
	if snapshot.is_empty():
		inspect.text = "%s  •  snapshot unavailable\nInventory, route, provenance: unavailable" % selected.to_upper()
		return
	var id: int = snapshot.get(selected + "_id", 0)
	var activity := "idle" if snapshot.get("job_id", 0) == 0 else ("blocked" if snapshot.get("blocked", false) else "working (synthetic)")
	inspect.text = "%s  #%d   •   tick %s\nStock %s / target %s   |   %s\nJob #%s  %s  •  progress %s\nInventory, route, provenance: unavailable" % [selected.to_upper(), id, snapshot.get("tick", 0), snapshot.get("stock", 0), snapshot.get("target", 0), activity, snapshot.get("job_id", 0), snapshot.get("job_key", ""), snapshot.get("progress", 0)]

func _input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_F2:
			editor.grab_focus()
			get_viewport().set_input_as_handled()
		elif event.keycode == KEY_ESCAPE and editor.has_focus():
			editor.release_focus()
			get_viewport().set_input_as_handled()
		elif event.keycode == KEY_ENTER and event.ctrl_pressed and editor.has_focus():
			apply_policy()
			get_viewport().set_input_as_handled()
		elif event.keycode == KEY_E and not editor.has_focus() and not get_viewport().gui_get_focus_owner() is Button:
			select_nearby()
			get_viewport().set_input_as_handled()

func select_nearby() -> void:
	var choices := {"worker": world.worker.position, "stockpile": Vector3(-3, 0, 0), "workshop": Vector3(4, 0, -2)}
	var nearest := 3.5
	selected = ""
	for key in choices:
		var distance: float = world.player.position.distance_to(choices[key])
		if distance < nearest:
			nearest = distance
			selected = key
	refresh_ui()

func mark_worker_unavailable() -> void:
	worker_unavailable = true
	pending = ""
	last_diagnostic = "Bridge worker disconnected; simulation unavailable."

func _physics_process(delta: float) -> void:
	var direction := Vector3.ZERO
	if get_viewport().gui_get_focus_owner() == null:
		direction = Vector3(float(Input.is_key_pressed(KEY_D)) - float(Input.is_key_pressed(KEY_A)), 0, float(Input.is_key_pressed(KEY_S)) - float(Input.is_key_pressed(KEY_W))).normalized()
	world.update_motion(delta, direction)
	if bridge == null or worker_unavailable:
		return
	var result: Dictionary = bridge.poll()
	if result.get("disconnected", false):
		mark_worker_unavailable()
		refresh_ui()
		return
	if not result.is_empty():
		pending = ""
		snapshot = result
		world.show_snapshot(result)
		if result.request == "activate":
			if result.status == "OK":
				active_source = submitted_source
				last_diagnostic = "Revision %s accepted; compatible fixture progress retained." % result.revision
			else:
				last_diagnostic = "Revision %s rejected: %s" % [result.requested_revision, result.status]
		elif result.status != "OK":
			last_diagnostic = "Active policy evaluation failed: " + result.status
		refresh_ui()
	timer += delta
	if timer >= 0.5 and pending == "" and snapshot.get("revision", -1) >= 0:
		if bridge.tick():
			pending = "tick"
			timer = 0.0
		elif bridge.is_disconnected():
			mark_worker_unavailable()
			refresh_ui()
