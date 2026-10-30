mod sim;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};
use sim::*;

const BG: Color = Color::new(0.10, 0.15, 0.17, 1.0);
const GOLD: Color = Color::new(1.0, 0.75, 0.28, 1.0);
fn config() -> Conf {
    Conf {
        window_title: "CLONK — Probably productive.".into(),
        window_width: 1120,
        window_height: 760,
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
    let mut running = false;
    let mut elapsed = 0.0;
    let mut delay = 0.45;
    let mut robot_pos = vec2(START.0 as f32, START.1 as f32);
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
        widgets::Window::new(hash!(), vec2(770., 112.), vec2(325., 610.))
            .label("CL-01 / INSTRUCTIONS")
            .movable(false)
            .ui(&mut root_ui(), |ui| {
                ui.label(None, "Edit, then Apply & reset.");
                widgets::Editbox::new(hash!(), vec2(295., 360.)).ui(ui, &mut source);
                compile = ui.button(None, "Apply & reset");
                if ui.button(None, if running { "Pause" } else { "Run" }) {
                    running = !running;
                    elapsed = 0.;
                }
                ui.same_line(0.);
                step = ui.button(None, "Step");
                ui.same_line(0.);
                reset = ui.button(None, "Reset");
                if ui.button(None, "Speed: normal / fast") {
                    delay = if delay > 0.2 { 0.12 } else { 0.45 };
                }
                ui.label(None, "move(5) / turn(left) / turn(around)");
                ui.label(None, "collect() / deposit() / build(charger)");
                ui.label(None, "Comments start with #");
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
                    elapsed = 0.;
                }
                Err(error) => world.message = error,
            }
        }
        if reset {
            world = World::default();
            running = false;
            elapsed = 0.;
        }
        if step {
            running = false;
            world.step(&program);
        }
        if running {
            elapsed += get_frame_time();
            if elapsed >= delay {
                elapsed = 0.;
                running = world.step(&program);
            }
        }
        robot_pos = robot_pos.lerp(
            vec2(world.position.0 as f32, world.position.1 as f32),
            (get_frame_time() * 12.).min(1.),
        );
        let cell = 57.;
        let origin = vec2(34., 125.);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let p = origin + vec2(x as f32 * cell, y as f32 * cell);
                let color = if wall(x, y) {
                    Color::from_hex(0x586d68)
                } else {
                    if (x + y) % 2 == 0 {
                        Color::from_hex(0xabbca2)
                    } else {
                        Color::from_hex(0xa4b59c)
                    }
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
                if world.charger == Some((x, y)) {
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
        let p = origin + vec2(robot_pos.x * cell + 27., robot_pos.y * cell + 25.);
        draw_ellipse(
            p.x + 3.,
            p.y + 17.,
            21.,
            7.,
            0.,
            Color::new(0.15, 0.2, 0.18, 0.3),
        );
        draw_circle(p.x, p.y, 20., Color::from_hex(0x386c70));
        draw_circle(p.x, p.y - 3., 18., Color::from_hex(0x9ae2d2));
        draw_circle(p.x - 6., p.y - 8., 5., Color::from_hex(0xc6f4e4));
        let d = [vec2(1., 0.), vec2(0., 1.), vec2(-1., 0.), vec2(0., -1.)][world.direction];
        let side = vec2(-d.y, d.x);
        for offset in [-5., 5.] {
            draw_circle(
                p.x + d.x * 10. + side.x * offset,
                p.y + d.y * 10. + side.y * offset - 2.,
                2.5,
                BG,
            );
        }
        if world.carrying {
            draw_circle(p.x - 14., p.y - 15., 7., GOLD);
        }
        draw_text(
            format!(
                "DEPOT ORE  {:02}     TICK  {}     {}",
                world.delivered,
                world.ticks,
                if running {
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
        draw_text(&world.message, 34., 705., 18., WHITE);
        let next = program
            .get(world.pc)
            .map(|i| {
                let text = applied.lines().nth(i.line).unwrap_or("");
                let text = text.split('#').next().unwrap_or("").trim();
                format!("LINE {:02}: {}", i.line + 1, text)
            })
            .unwrap_or("FINISHED".into());
        draw_text(format!("NEXT  {next}"), 34., 736., 18., SKYBLUE);
        next_frame().await;
    }
}
