mod sim;
mod view;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};
use sim::*;
use view::{BODY_COLORS, WorldView};

const BG: Color = Color::new(0.10, 0.15, 0.17, 1.0);
const GOLD: Color = Color::new(1.0, 0.75, 0.28, 1.0);
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
    let mut world_view = WorldView::new();
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
                if ui.button(None, "Colony example") {
                    source = DEMO.into();
                }
                ui.same_line(0.);
                if ui.button(None, "Refinery example") {
                    source = REFINERY_DEMO.into();
                }
                widgets::Editbox::new(hash!(), vec2(295., 300.)).ui(ui, &mut source);
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
                ui.label(None, "collect() deposit() charge() refine()");
                ui.label(None, "build(charger/robot/refinery) repeat()");
                ui.label(None, "if(...) else end / while(...) end");
                ui.label(None, "wall robot charger refinery blocked");
                ui.label(None, "ore home / refinery: 2 depot ore");
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
                    world_view.reset();
                    editor_error = None;
                    elapsed = 0.;
                }
                Err(error) => editor_error = Some(error),
            }
        }
        if reset {
            world = World::default();
            world_view.reset();
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
        world_view.draw(&world, get_frame_time());
        draw_text(
            if world.first_shift_complete() {
                "FIRST SHIFT COMPLETE. Next: build a refinery and refine ore."
            } else {
                "FIRST SHIFT: build a charger, recharge, then deliver again."
            },
            34.,
            110.,
            18.,
            SKYBLUE,
        );
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
            674.,
            17.,
            SKYBLUE,
        );
        draw_text(
            if world.production_complete() {
                "FIRST PART COMPLETE. Production line operational."
            } else {
                "NEXT: build(refinery), face it with ore, then refine()."
            },
            34.,
            654.,
            17.,
            GOLD,
        );
        let working = world.robots.iter().any(|r| r.state == State::Working);
        draw_text(
            format!(
                "ORE {:02}  PARTS {:02}  MINE {:02}  BOTS {}/{}  TICK {}  {}",
                world.delivered,
                world.parts,
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
            698.,
            18.,
            GOLD,
        );
        draw_text(&world.message, 34., 724., 18., WHITE);
        for (i, robot) in world.robots.iter().enumerate() {
            let y = 750. + i as f32 * 26.;
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
            let color = Color::from_hex(BODY_COLORS[i % BODY_COLORS.len()]);
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
