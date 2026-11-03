//! Presentation only: the simulation's integer grid lies on the X/Z plane.
use crate::sim::*;
use macroquad::prelude::*;

pub const BODY_COLORS: [u32; MAX_ROBOTS] = [0x9ae2d2, 0xf2b3c9, 0xc8bbf2, 0xf5cf96];
const GOLD: Color = Color::new(1.0, 0.75, 0.28, 1.0);
const VIEW_SIZE: Vec2 = vec2(720., 500.);
const VIEW_ORIGIN: Vec2 = vec2(24., 130.);

pub struct WorldView {
    target: RenderTarget,
    positions: Vec<Vec2>,
}

impl WorldView {
    pub fn new() -> Self {
        let target = render_target_ex(
            VIEW_SIZE.x as u32,
            VIEW_SIZE.y as u32,
            RenderTargetParams {
                depth: true,
                ..Default::default()
            },
        );
        target.texture.set_filter(FilterMode::Linear);
        Self {
            target,
            positions: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.positions.clear();
    }

    pub fn draw(&mut self, world: &World, dt: f32) {
        self.positions.truncate(world.robots.len());
        for (i, robot) in world.robots.iter().enumerate() {
            let destination = vec2(robot.position.0 as f32, robot.position.1 as f32);
            match self.positions.get_mut(i) {
                Some(p) => *p = p.lerp(destination, (dt * 12.).clamp(0., 1.)),
                None => self.positions.push(destination),
            }
        }
        let mut camera = board_camera();
        camera.render_target = Some(self.target.clone());
        set_camera(&camera);
        clear_background(Color::from_hex(0x192b30));
        draw_board(world);
        for (robot, position) in world.robots.iter().zip(&self.positions) {
            draw_robot(robot, grid_position(*position));
        }
        // Restore screen-space rendering before drawing the editor and HUD.
        set_default_camera();
        draw_texture_ex(
            &self.target.texture,
            VIEW_ORIGIN.x,
            VIEW_ORIGIN.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(VIEW_SIZE),
                flip_y: true,
                ..Default::default()
            },
        );
        draw_rectangle_lines(
            VIEW_ORIGIN.x,
            VIEW_ORIGIN.y,
            VIEW_SIZE.x,
            VIEW_SIZE.y,
            1.,
            Color::from_hex(0x355059),
        );
        draw_text("TABLETOP / 3D", 40., 153., 16., GRAY);
        draw_text(
            "CYAN: HOME   GOLD: ORE   TEAL: CHARGER   PLUM: REFINERY",
            40.,
            615.,
            15.,
            LIGHTGRAY,
        );
    }
}

fn grid_position(position: Vec2) -> Vec3 {
    vec3(position.x, 0., position.y)
}

fn board_camera() -> Camera3D {
    let center = vec3((WIDTH - 1) as f32 / 2., 0., (HEIGHT - 1) as f32 / 2.);
    Camera3D {
        position: center + vec3(10., 17., 15.),
        target: center,
        up: Vec3::Y,
        projection: Projection::Orthographics,
        fovy: 13.,
        aspect: Some(VIEW_SIZE.x / VIEW_SIZE.y),
        z_near: 0.1,
        z_far: 100.,
        ..Default::default()
    }
}

/// Flat face shading gives procedural models depth without shaders or assets.
fn block(center: Vec3, size: Vec3, color: Color) {
    let base = center - size / 2.;
    let x = Vec3::X * size.x;
    let y = Vec3::Y * size.y;
    let z = Vec3::Z * size.z;
    for (offset, a, b, light) in [
        (base, x, z, 0.55),
        (base + y, x, z, 1.0),
        (base, x, y, 0.78),
        (base + z, x, y, 0.78),
        (base, y, z, 0.62),
        (base + x, y, z, 0.62),
    ] {
        let shaded = Color::new(color.r * light, color.g * light, color.b * light, color.a);
        draw_affine_parallelogram(offset, a, b, None, shaded);
    }
}

fn draw_tile(x: i32, z: i32) {
    let p = grid_position(vec2(x as f32, z as f32));
    let color = if (x + z) % 2 == 0 { 0xabbca2 } else { 0x96aa90 };
    block(
        p - Vec3::Y * 0.08,
        vec3(0.96, 0.16, 0.96),
        Color::from_hex(color),
    );
    if wall(x, z) {
        // Low perimeter walls preserve sightlines into the colony.
        let perimeter = x == 0 || z == 0 || x == WIDTH - 1 || z == HEIGHT - 1;
        let height = if perimeter { 0.28 } else { 0.8 };
        block(
            p + Vec3::Y * height / 2.,
            vec3(0.94, height, 0.94),
            Color::from_hex(0x71847b),
        );
    }
}

fn draw_board(world: &World) {
    let center = vec3((WIDTH - 1) as f32 / 2., -0.32, (HEIGHT - 1) as f32 / 2.);
    block(
        center,
        vec3(WIDTH as f32 + 0.2, 0.4, HEIGHT as f32 + 0.2),
        Color::from_hex(0x355059),
    );
    for z in 0..HEIGHT {
        for x in 0..WIDTH {
            draw_tile(x, z);
        }
    }
    let home = grid_position(vec2(START.0 as f32, START.1 as f32));
    block(home + Vec3::Y * 0.025, vec3(0.9, 0.05, 0.9), SKYBLUE);
    block(
        home + Vec3::Y * 0.055,
        vec3(0.68, 0.02, 0.68),
        Color::from_hex(0x355059),
    );
    let ore = grid_position(vec2(ORE.0 as f32, ORE.1 as f32));
    block(
        ore + Vec3::Y * 0.02,
        vec3(0.88, 0.04, 0.88),
        Color::from_hex(0x6e624c),
    );
    if world.ore > 0 {
        for (offset, size) in [
            (vec3(-0.22, 0.17, 0.12), 0.3),
            (vec3(0.14, 0.23, 0.03), 0.42),
            (vec3(-0.05, 0.13, -0.24), 0.22),
        ] {
            block(ore + offset, Vec3::splat(size), GOLD);
        }
    }
    for &(x, z) in &world.chargers {
        draw_charger(grid_position(vec2(x as f32, z as f32)));
    }
    for &(x, z) in &world.refineries {
        draw_refinery(grid_position(vec2(x as f32, z as f32)));
    }
}

fn draw_charger(p: Vec3) {
    block(
        p + vec3(0., 0.12, 0.),
        vec3(0.8, 0.24, 0.8),
        Color::from_hex(0x486f76),
    );
    block(
        p + vec3(0., 0.61, 0.),
        vec3(0.52, 0.78, 0.48),
        Color::from_hex(0x99ded1),
    );
    block(p + vec3(0., 1.04, 0.), vec3(0.32, 0.12, 0.3), GOLD);
    block(p + vec3(0., 0.65, 0.25), vec3(0.3, 0.08, 0.03), WHITE);
    block(p + vec3(0., 0.65, 0.25), vec3(0.08, 0.3, 0.03), WHITE);
}

fn draw_refinery(p: Vec3) {
    block(
        p + vec3(0., 0.36, 0.),
        vec3(0.82, 0.72, 0.76),
        Color::from_hex(0xa47898),
    );
    block(
        p + vec3(0.23, 0.99, -0.16),
        vec3(0.22, 0.54, 0.25),
        Color::from_hex(0x655164),
    );
    block(
        p + vec3(0., 0.35, 0.39),
        vec3(0.48, 0.32, 0.04),
        Color::from_hex(0x382d3b),
    );
    block(p + vec3(0., 0.34, 0.42), vec3(0.32, 0.14, 0.04), GOLD);
    block(
        p + vec3(-0.23, 0.79, 0.),
        vec3(0.28, 0.14, 0.4),
        Color::from_hex(0xd8b4c9),
    );
}

fn draw_robot(robot: &Robot, p: Vec3) {
    let offline = robot.state == State::PoweredDown;
    let body = if offline {
        GRAY
    } else {
        Color::from_hex(BODY_COLORS[(robot.id as usize - 1) % BODY_COLORS.len()])
    };
    let (dx, dz) = robot.direction.delta();
    let forward = vec3(dx as f32, 0., dz as f32);
    let side = vec3(-forward.z, 0., forward.x);
    let dark = Color::from_hex(0x304950);
    block(p + vec3(0., 0.17, 0.), vec3(0.64, 0.24, 0.64), dark);
    block(p + vec3(0., 0.49, 0.), vec3(0.57, 0.48, 0.57), body);
    block(p + vec3(0., 0.77, 0.), vec3(0.4, 0.12, 0.4), body);
    for offset in [-0.13, 0.13] {
        let eye = p + forward * 0.3 + side * offset + Vec3::Y * 0.58;
        block(
            eye,
            vec3(0.085, if offline { 0.025 } else { 0.095 }, 0.085),
            dark,
        );
    }
    if robot.carrying {
        block(p + forward * 0.44 + Vec3::Y * 0.32, Vec3::splat(0.25), GOLD);
    }
    // Eyes on the far side are occluded: a top arrow keeps all four headings readable.
    let marker = p + Vec3::Y * 0.835;
    let marker_color = if robot.state == State::Lost {
        GOLD
    } else {
        dark
    };
    draw_mesh(&Mesh {
        vertices: [
            marker + forward * 0.19,
            marker - forward * 0.04 + side * 0.12,
            marker - forward * 0.04 - side * 0.12,
        ]
        .map(|point| Vertex::new2(point, Vec2::ZERO, marker_color))
        .to_vec(),
        indices: vec![0, 1, 2],
        texture: None,
    });
    let charge = robot.energy as f32 / BATTERY as f32;
    let battery = p - forward * 0.23 + Vec3::Y * 0.85;
    let size = side.abs() * 0.42 + forward.abs() * 0.09;
    block(battery, size + Vec3::Y * 0.04, dark);
    if charge > 0. {
        let color = if robot.energy < LOW_BATTERY {
            RED
        } else {
            GREEN
        };
        block(
            battery + side * (-0.21 + 0.21 * charge) + Vec3::Y * 0.03,
            side.abs() * (0.42 * charge) + forward.abs() * 0.09 + Vec3::Y * 0.025,
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_maps_to_ground_plane_without_changing_axes() {
        assert_eq!(grid_position(vec2(2., 4.)), vec3(2., 0., 4.));
        for direction in [
            Direction::East,
            Direction::South,
            Direction::West,
            Direction::North,
        ] {
            let (x, z) = direction.delta();
            let delta = grid_position(vec2(x as f32, z as f32));
            assert_eq!(delta.length(), 1.);
            assert_eq!(delta.y, 0.);
        }
    }

    #[test]
    fn camera_fits_board_and_buildings_with_margin() {
        let camera = board_camera();
        // Camera::matrix queries the live graphics context, even with an explicit
        // aspect. Build its orthographic transform here to keep this test headless.
        let top = camera.fovy / 2.;
        let right = top * camera.aspect.unwrap();
        let matrix =
            Mat4::orthographic_rh_gl(-right, right, -top, top, camera.z_near, camera.z_far)
                * Mat4::look_at_rh(camera.position, camera.target, camera.up);
        for x in [-0.6, WIDTH as f32 - 0.4] {
            for z in [-0.6, HEIGHT as f32 - 0.4] {
                for y in [-0.52, 1.3] {
                    let projected = matrix.project_point3(vec3(x, y, z));
                    assert!(projected.x.abs() < 0.95, "{projected:?}");
                    assert!(projected.y.abs() < 0.95, "{projected:?}");
                    assert!(projected.z.abs() < 1.);
                }
            }
        }
    }
}
