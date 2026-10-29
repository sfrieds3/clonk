mod sim;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};
use sim::*;

const BG: Color = Color::new(0.055, 0.075, 0.10, 1.0);
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
    let mut running = false;
    let mut elapsed = 0.0;
    let mut delay = 0.3;
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
                    delay = if delay > 0.2 { 0.08 } else { 0.3 };
                }
                ui.label(None, "MOVE / TURN (clockwise)");
                ui.label(None, "COLLECT / DEPOSIT / REPEAT");
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
        let cell = 57.;
        let origin = vec2(34., 125.);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let p = origin + vec2(x as f32 * cell, y as f32 * cell);
                let color = if wall(x, y) {
                    Color::from_hex(0x303c48)
                } else {
                    Color::from_hex(0x15232c)
                };
                draw_rectangle(p.x, p.y, cell - 3., cell - 3., color);
                if (x, y) == START {
                    draw_rectangle_lines(p.x + 4., p.y + 4., cell - 11., cell - 11., 2., SKYBLUE);
                    draw_text("HOME", p.x + 6., p.y + 47., 15., SKYBLUE);
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
        let p = origin
            + vec2(
                world.position.0 as f32 * cell + 27.,
                world.position.1 as f32 * cell + 25.,
            );
        draw_rectangle(p.x - 16., p.y - 15., 32., 30., Color::from_hex(0x8cddd0));
        let d = [vec2(1., 0.), vec2(0., 1.), vec2(-1., 0.), vec2(0., -1.)][world.direction];
        draw_circle(p.x + d.x * 12., p.y + d.y * 12., 5., BG);
        if world.carrying {
            draw_circle(p.x - 14., p.y - 15., 7., GOLD);
        }
        draw_text(
            format!(
                "DELIVERED  {:02}     TICK  {}     {}",
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
        let instruction = program
            .get(world.pc)
            .map(|i| format!("{i:?}").to_uppercase())
            .unwrap_or("FINISHED".into());
        draw_text(
            format!("NEXT  {:02}: {}", world.pc + 1, instruction),
            34.,
            736.,
            18.,
            SKYBLUE,
        );
        next_frame().await;
    }
}
