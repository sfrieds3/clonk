mod sim;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};
use sim::*;

const BG: Color = Color::new(0.10, 0.15, 0.17, 1.0);
const GOLD: Color = Color::new(1.0, 0.75, 0.28, 1.0);
/// Body colors, one per robot, so you can tell who is clonking whom.
const BODIES: [u32; MAX_ROBOTS] = [0x9ae2d2, 0xf2b3c9, 0xc8bbf2, 0xf5cf96];
const SPEEDS: [(&str, f32); 3] = [("normal", 0.45), ("fast", 0.12), ("frantic", 0.03)];

fn config() -> Conf {
    Conf {
        window_title: "CLONK — Probably productive.".into(),
        window_width: 1120,
        window_height: 850,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    let mut world = World::default();
    let mut source = DEMO.to_string();
    let mut program = parse(&source).unwrap();
    let mut applied = source.clone();
    let mut editor_error: Option<String> = None;
    let mut running = false;
    let mut elapsed = 0.0;
    let mut speed = 0;
    let mut robot_pos: Vec<Vec2> = Vec::new();
    loop {
        clear_background(BG);
        draw_text("CLONK", 32., 59., 48., GOLD);
        draw_text(
            "They're doing exactly what you told them.",
            34.,
            86.,
            20.,
            GRAY,
        );
        let previous = source.clone();
        let mut step = false;
        let mut reset = false;
        let mut compile = false;
        widgets::Window::new(hash!(), vec2(770., 112.), vec2(325., 700.))
            .label("COLONY / INSTRUCTIONS")
            .movable(false)
            .ui(&mut root_ui(), |ui| {
                ui.label(
                    None,
                    if source == applied {
                        "Applied program"
                    } else {
                        "Unapplied edits: Run uses applied code"
                    },
                );
                widgets::Editbox::new(hash!(), vec2(295., 330.)).ui(ui, &mut source);
                compile = ui.button(None, "Apply & reset");
                if ui.button(None, if running { "Pause" } else { "Run" }) {
                    running = !running;
                    elapsed = 0.;
                }
                ui.same_line(0.);
                step = ui.button(None, "Step");
                ui.same_line(0.);
                reset = ui.button(None, "Reset");
                if ui.button(None, format!("Speed: {}", SPEEDS[speed].0)) {
                    speed = (speed + 1) % SPEEDS.len();
                }
                ui.label(None, "move(5) turn(left) turn(around) wait()");
                ui.label(None, "collect() deposit() charge()");
                ui.label(None, "build(charger) build(robot) repeat()");
                ui.label(None, "if(...) else end / while(...) end");
                ui.label(None, "wall robot charger blocked ore home");
                ui.label(None, "carrying low id(2) / not <sensor>");
                if let Some(error) = &editor_error {
                    ui.label(None, "Apply failed:");
                    for line in diagnostic_lines(error, 38).into_iter().take(5) {
                        ui.label(None, &line);
                    }
                }
            });
        if source != previous {
            running = false;
        }
        if compile {
            running = false;
            match parse(&source) {
                Ok(next) => {
                    program = next;
                    applied = source.clone();
                    world = World::default();
                    robot_pos.clear();
                    editor_error = None;
                    elapsed = 0.;
                }
                Err(error) => editor_error = Some(error),
            }
        }
        if reset {
            world = World::default();
            robot_pos.clear();
            running = false;
            elapsed = 0.;
        }
        if step {
            running = false;
            world.step(&program);
        }
        if running {
            elapsed += get_frame_time();
            if elapsed >= SPEEDS[speed].1 {
                elapsed = 0.;
                running = world.step(&program);
            }
        }
        robot_pos.truncate(world.robots.len());
        for (i, robot) in world.robots.iter().enumerate() {
            let target = vec2(robot.position.0 as f32, robot.position.1 as f32);
            match robot_pos.get_mut(i) {
                Some(pos) => *pos = pos.lerp(target, (get_frame_time() * 12.).min(1.)),
                None => robot_pos.push(target),
            }
        }
        draw_text(
            if world.first_shift_complete() {
                "FIRST SHIFT COMPLETE. Keep building; this is only the beginning."
            } else {
                "FIRST SHIFT: build a charger, recharge, then deliver again."
            },
            34.,
            110.,
            18.,
            SKYBLUE,
        );
        let cell = 57.;
        let origin = vec2(34., 125.);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let p = origin + vec2(x as f32 * cell, y as f32 * cell);
                let color = if wall(x, y) {
                    Color::from_hex(0x586d68)
                } else if (x + y) % 2 == 0 {
                    Color::from_hex(0xabbca2)
                } else {
                    Color::from_hex(0xa4b59c)
                };
                draw_rectangle(p.x, p.y, cell - 2., cell - 2., color);
                if wall(x, y) {
                    draw_rectangle(
                        p.x + 5.,
                        p.y + 5.,
                        cell - 12.,
                        cell - 18.,
                        Color::from_hex(0x71847b),
                    );
                    draw_line(
                        p.x + 7.,
                        p.y + 37.,
                        p.x + 46.,
                        p.y + 37.,
                        2.,
                        Color::from_hex(0x465c58),
                    );
                }
                if (x, y) == START {
                    draw_rectangle_lines(p.x + 4., p.y + 4., cell - 11., cell - 11., 2., SKYBLUE);
                    draw_text("HOME", p.x + 6., p.y + 47., 15., SKYBLUE);
                }
                if world.chargers.contains(&(x, y)) {
                    draw_rectangle(p.x + 9., p.y + 10., 38., 39., Color::from_hex(0x486f76));
                    draw_rectangle(p.x + 13., p.y + 7., 30., 28., Color::from_hex(0x99ded1));
                    draw_text("+", p.x + 20., p.y + 30., 23., Color::from_hex(0x355a61));
                    draw_circle(p.x + 39., p.y + 43., 3., GOLD);
                }
                if (x, y) == ORE {
                    draw_poly(
                        p.x + 27.,
                        p.y + 25.,
                        6,
                        15.,
                        30.,
                        if world.ore > 0 { GOLD } else { DARKGRAY },
                    );
                    draw_text(world.ore.to_string(), p.x + 23., p.y + 49., 17., WHITE);
                }
            }
        }
        for (robot, pos) in world.robots.iter().zip(&robot_pos) {
            draw_robot(robot, origin + vec2(pos.x * cell + 27., pos.y * cell + 25.));
        }
        draw_text(
            format!(
                "Charger [{}]  Recharged [{}]  Delivery after recharge [{}]",
                if !world.chargers.is_empty() { "x" } else { " " },
                if world.robots.iter().any(|r| r.recharges > 0) {
                    "x"
                } else {
                    " "
                },
                if world.delivered_after_recharge {
                    "x"
                } else {
                    " "
                },
            ),
            34.,
            650.,
            17.,
            SKYBLUE,
        );
        let working = world.robots.iter().any(|r| r.state == State::Working);
        draw_text(
            format!(
                "DEPOT {:02}   MINE {:02}   ROBOTS {}/{}   TICK {}   {}",
                world.delivered,
                world.ore,
                world.robots.len(),
                MAX_ROBOTS,
                world.ticks,
                if running && working {
                    "WORKING, ALLEGEDLY"
                } else {
                    "PAUSED"
                }
            ),
            34.,
            672.,
            22.,
            GOLD,
        );
        draw_text(&world.message, 34., 700., 18., WHITE);
        for (i, robot) in world.robots.iter().enumerate() {
            let y = 732. + i as f32 * 26.;
            let next = match robot.state {
                State::Working => upcoming(&program, robot.pc)
                    .map(|i| {
                        let text = applied.lines().nth(i.line).unwrap_or("");
                        let text = text.split('#').next().unwrap_or("").trim();
                        format!("L{:02} {}", i.line + 1, truncate(text, 22))
                    })
                    .unwrap_or("FINISHING".into()),
                State::Finished => "FINISHED".into(),
                State::Lost => "LOST".into(),
                State::PoweredDown => "OFFLINE".into(),
            };
            let color = Color::from_hex(BODIES[i % BODIES.len()]);
            draw_text(robot.name(), 34., y, 18., color);
            let power = format!("PWR {:02}", robot.energy);
            let power_color = if robot.energy < LOW_BATTERY {
                RED
            } else {
                SKYBLUE
            };
            draw_text(power, 100., y, 18., power_color);
            draw_text(next, 180., y, 18., SKYBLUE);
            draw_text(truncate(&robot.message, 48), 420., y, 18., WHITE);
        }
        next_frame().await;
    }
}

fn diagnostic_lines(message: &str, width: usize) -> Vec<String> {
    message
        .chars()
        .collect::<Vec<_>>()
        .chunks(width)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.into()
    } else {
        text.chars().take(max - 1).chain(['…']).collect()
    }
}

fn draw_robot(robot: &Robot, p: Vec2) {
    let offline = robot.state == State::PoweredDown;
    let body = if offline {
        Color::from_hex(0x8a9590)
    } else {
        Color::from_hex(BODIES[(robot.id as usize - 1) % BODIES.len()])
    };
    draw_ellipse(
        p.x + 3.,
        p.y + 17.,
        21.,
        7.,
        0.,
        Color::new(0.15, 0.2, 0.18, 0.3),
    );
    draw_circle(p.x, p.y, 20., Color::from_hex(0x386c70));
    draw_circle(p.x, p.y - 3., 18., body);
    draw_circle(p.x - 6., p.y - 8., 5., Color::new(1., 1., 1., 0.35));
    let (dx, dy) = robot.direction.delta();
    let d = vec2(dx as f32, dy as f32);
    let side = vec2(-d.y, d.x);
    for offset in [-5., 5.] {
        let eye = vec2(
            p.x + d.x * 10. + side.x * offset,
            p.y + d.y * 10. + side.y * offset - 2.,
        );
        if offline {
            draw_line(eye.x - 2., eye.y, eye.x + 2., eye.y, 1.5, BG);
        } else {
            draw_circle(eye.x, eye.y, 2.5, BG);
        }
    }
    if robot.carrying {
        draw_circle(p.x - 14., p.y - 15., 7., GOLD);
    }
    if robot.state == State::Lost {
        draw_text("?", p.x + 10., p.y - 14., 24., WHITE);
    }
    draw_text(robot.id.to_string(), p.x - 4., p.y + 4., 16., BG);
    let charge = robot.energy as f32 / BATTERY as f32;
    let bar = if robot.energy < LOW_BATTERY {
        RED
    } else {
        GREEN
    };
    draw_rectangle(p.x - 18., p.y + 21., 36., 4., Color::new(0., 0., 0., 0.35));
    draw_rectangle(p.x - 18., p.y + 21., 36. * charge, 4., bar);
}
