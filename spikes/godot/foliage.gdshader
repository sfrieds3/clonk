shader_type spatial;
render_mode diffuse_burley;
void vertex() {
	VERTEX.x += sin(TIME * 1.4 + VERTEX.y * 2.0 + MODEL_MATRIX[3].x) * 0.045 * max(VERTEX.y + 0.5, 0.0);
}
void fragment() {
	ALBEDO = vec3(0.23, 0.36, 0.24);
	ROUGHNESS = 0.9;
}
