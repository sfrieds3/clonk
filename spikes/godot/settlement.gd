extends Node3D
## Procedural presentation only. The worker's pose follows fixture progress, not a route.
var worker: Node3D
var worker_legs: Array[Node3D] = []
var player_legs: Array[Node3D] = []
var stock_crates: Array[Node3D] = []
var player: CharacterBody3D
var camera: Camera3D
var phase := 0.0
var worker_position := Vector3(-3, 0, 0)

func _ready() -> void:
	var env := WorldEnvironment.new()
	var atmosphere := Environment.new()
	atmosphere.background_mode = Environment.BG_COLOR
	atmosphere.background_color = Color("a4b9b1")
	atmosphere.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	atmosphere.ambient_light_color = Color("c9d3c2")
	atmosphere.ambient_light_energy = 0.35
	env.environment = atmosphere
	add_child(env)
	var sun := DirectionalLight3D.new()
	sun.rotation_degrees = Vector3(-55, -35, 0)
	sun.light_color = Color("ffe3b7")
	sun.light_energy = 0.65
	sun.shadow_enabled = true
	add_child(sun)
	block(self, Vector3(0, -0.4, 0), Vector3(30, 0.8, 30), "6d805c", true)
	# Workshop, roof, porch and window: original procedural geometry.
	block(self, Vector3(4, 1.15, -2), Vector3(4, 2.3, 3), "b79b78", true)
	block(self, Vector3(4, 2.5, -2), Vector3(4.5, 0.5, 3.5), "715444", false)
	block(self, Vector3(4, 0.65, -0.45), Vector3(1.0, 1.3, 0.12), "516b6a", false)
	block(self, Vector3(5.1, 1.45, -0.43), Vector3(0.8, 0.65, 0.12), "9ecbc5", false)
	block(self, Vector3(-3, 0.18, 0), Vector3(2.8, 0.36, 2.3), "9e8060", true)
	for i in range(3):
		var crate := Node3D.new()
		add_child(crate)
		crate.position = Vector3(-3.8 + i * 0.8, 0, 0)
		block(crate, Vector3(0, 0.5, 0), Vector3(0.65, 0.6, 0.65), "d5aa70", false)
		crate.visible = false
		stock_crates.append(crate)
	block(self, Vector3(-5, 0.55, -4), Vector3(2, 1.1, 1.7), "809da0", true)
	for x in range(-5, 6):
		block(self, Vector3(x * 0.9, 0.025, 2.5), Vector3(0.82, 0.05, 1.0), "a99c7d", false)
	for pos in [Vector3(-7, 0, -3), Vector3(8, 0, 3), Vector3(-7, 0, 6), Vector3(8, 0, -7)]:
		tree(pos)
	player = CharacterBody3D.new()
	add_child(player)
	player.position = Vector3(0, 1, 6)
	var shape := CollisionShape3D.new()
	var capsule := CapsuleShape3D.new()
	capsule.height = 1.7
	capsule.radius = 0.35
	shape.shape = capsule
	player.add_child(shape)
	var avatar := Node3D.new()
	avatar.position.y = -0.85
	player.add_child(avatar)
	humanoid(avatar, "527f85", player_legs)
	worker = Node3D.new()
	add_child(worker)
	worker.position = worker_position
	humanoid(worker, "ce8562", worker_legs)
	camera = Camera3D.new()
	add_child(camera)
	camera.current = true

func block(parent: Node3D, pos: Vector3, size: Vector3, color: String, solid: bool) -> void:
	var body := StaticBody3D.new() if solid else Node3D.new()
	body.position = pos
	parent.add_child(body)
	var mesh := MeshInstance3D.new()
	var box := BoxMesh.new()
	box.size = size
	mesh.mesh = box
	var material := StandardMaterial3D.new()
	material.albedo_color = Color(color)
	material.roughness = 0.8
	mesh.material_override = material
	body.add_child(mesh)
	if solid:
		var collision := CollisionShape3D.new()
		var shape := BoxShape3D.new()
		shape.size = size
		collision.shape = shape
		body.add_child(collision)

func humanoid(parent: Node3D, coat: String, legs: Array[Node3D]) -> void:
	block(parent, Vector3(0, 1.05, 0), Vector3(0.55, 0.7, 0.32), coat, false)
	block(parent, Vector3(0, 1.58, 0), Vector3(0.38, 0.38, 0.38), "e8be93", false)
	block(parent, Vector3(0, 1.79, 0), Vector3(0.43, 0.09, 0.43), "625b50", false)
	for side in [-1, 1]:
		block(parent, Vector3(side * 0.36, 1.05, 0), Vector3(0.16, 0.6, 0.2), coat, false)
		var leg := Node3D.new()
		leg.position = Vector3(side * 0.15, 0.7, 0)
		parent.add_child(leg)
		block(leg, Vector3(0, -0.32, 0), Vector3(0.2, 0.64, 0.24), "485c5b", false)
		legs.append(leg)

func tree(pos: Vector3) -> void:
	block(self, pos + Vector3(0, 0.7, 0), Vector3(0.32, 1.4, 0.32), "84664d", true)
	var foliage := MeshInstance3D.new()
	var sphere := SphereMesh.new()
	sphere.radius = 1.1
	sphere.height = 2.0
	foliage.mesh = sphere
	foliage.position = pos + Vector3(0, 2.1, 0)
	var mat := ShaderMaterial.new()
	mat.shader = load("res://foliage.gdshader")
	foliage.material_override = mat
	add_child(foliage)

func show_snapshot(data: Dictionary) -> void:
	# Position is an illustration of five authoritative progress steps; no path is simulated.
	var stock: int = data.get("stock", 0)
	for i in range(stock_crates.size()):
		stock_crates[i].visible = stock > i
	var progress: int = data.get("progress", 0)
	var working: bool = data.get("job_id", 0) != 0 and not data.get("blocked", false)
	var amount := float(progress % 5) / 4.0 if working else 0.0
	worker_position = Vector3(-3.0 + amount * 3.8, 0, 1.5)

func update_motion(delta: float, direction: Vector3) -> void:
	# Safety net for leaving the finite floor; the simulation snapshot is untouched.
	if player.position.y < -5.0 or absf(player.position.x) > 16.0 or absf(player.position.z) > 16.0:
		player.position = Vector3(0, 1, 6)
		player.velocity = Vector3.ZERO
	player.velocity = Vector3(direction.x * 4.0, player.velocity.y - 12.0 * delta, direction.z * 4.0)
	player.move_and_slide()
	if direction.length() > 0.01:
		phase += delta * 9.0
		player.get_child(1).rotation.y = atan2(direction.x, direction.z)
	for i in range(player_legs.size()):
		player_legs[i].rotation.x = sin(phase + i * PI) * 0.5 if direction.length() > 0.01 else 0.0
	worker.position = worker.position.lerp(worker_position, minf(1.0, delta * 4.0))
	for i in range(worker_legs.size()):
		worker_legs[i].rotation.x = sin(Time.get_ticks_msec() * 0.009 + i * PI) * 0.4 if worker.position.distance_to(worker_position) > 0.05 else 0.0
	camera.position = player.position + Vector3(0, 7, 10)
	camera.look_at(player.position, Vector3.UP)
