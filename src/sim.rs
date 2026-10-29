pub const WIDTH: i32 = 12;
pub const HEIGHT: i32 = 9;
pub const START: (i32, i32) = (2, 4);
pub const ORE: (i32, i32) = (7, 4);
pub const DEMO: &str = "MOVE\nMOVE\nMOVE\nMOVE\nMOVE\nCOLLECT\nTURN\nTURN\nMOVE\nMOVE\nMOVE\nMOVE\nMOVE\nDEPOSIT\nTURN\nTURN\nREPEAT";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Instruction {
    Move,
    Turn,
    Collect,
    Deposit,
    Repeat,
}

pub fn parse(source: &str) -> Result<Vec<Instruction>, String> {
    let mut program = Vec::new();
    for (line, text) in source.lines().enumerate() {
        let text = text.split('#').next().unwrap_or("").trim().to_uppercase();
        if text.is_empty() {
            continue;
        }
        program.push(match text.as_str() {
            "MOVE" => Instruction::Move,
            "TURN" => Instruction::Turn,
            "COLLECT" => Instruction::Collect,
            "DEPOSIT" => Instruction::Deposit,
            "REPEAT" => Instruction::Repeat,
            _ => return Err(format!("Line {}: unknown instruction {text}", line + 1)),
        });
    }
    if program.is_empty() {
        return Err("Give your little robot an instruction.".into());
    }
    Ok(program)
}

pub fn wall(x: i32, y: i32) -> bool {
    x <= 0
        || y <= 0
        || x >= WIDTH - 1
        || y >= HEIGHT - 1
        || (x == 5 && (2..=3).contains(&y))
        || (x == 8 && y == 6)
}

pub struct World {
    pub position: (i32, i32),
    pub direction: usize,
    pub carrying: bool,
    pub ore: u32,
    pub delivered: u32,
    pub pc: usize,
    pub ticks: u64,
    pub message: String,
}

impl Default for World {
    fn default() -> Self {
        Self {
            position: START,
            direction: 0,
            carrying: false,
            ore: 8,
            delivered: 0,
            pc: 0,
            ticks: 0,
            message: "Meet CL-01. Enthusiasm: high. Judgment: yours.".into(),
        }
    }
}

impl World {
    pub fn step(&mut self, program: &[Instruction]) -> bool {
        let Some(instruction) = program.get(self.pc) else {
            self.message = "Program finished. A remarkably quiet workplace.".into();
            return false;
        };
        self.ticks += 1;
        self.pc += 1;
        self.message = match instruction {
            Instruction::Move => {
                let delta = [(1, 0), (0, 1), (-1, 0), (0, -1)][self.direction];
                let next = (self.position.0 + delta.0, self.position.1 + delta.1);
                if wall(next.0, next.1) {
                    "CLONK. That is a wall."
                } else {
                    self.position = next;
                    "Small steps. Questionable ambitions."
                }
            }
            Instruction::Turn => {
                self.direction = (self.direction + 1) % 4;
                "Turning clockwise. A fresh perspective."
            }
            Instruction::Collect => {
                if self.carrying {
                    "Pockets full. Pocket, singular."
                } else if self.position == ORE && self.ore > 0 {
                    self.ore -= 1;
                    self.carrying = true;
                    "Acquired: one extremely important rock."
                } else {
                    "No ore here. Confidence remains unaffected."
                }
            }
            Instruction::Deposit => {
                if self.position == START && self.carrying {
                    self.carrying = false;
                    self.delivered += 1;
                    "Rock delivered. Industry has begun."
                } else {
                    "Deposit requires a rock and the depot."
                }
            }
            Instruction::Repeat => {
                self.pc = 0;
                "Again! Management loves consistency."
            }
        }
        .into();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn demo_delivers_all_ore_without_creating_more() {
        let mut world = World::default();
        let program = parse(DEMO).unwrap();
        for _ in 0..(program.len() * 12) {
            world.step(&program);
        }
        assert_eq!(world.delivered, 8);
        assert_eq!(world.ore, 0);
        assert!(!world.carrying);
        assert_eq!(world.position, START);
    }
    #[test]
    fn wall_blocks_motion_and_program_terminates() {
        let mut world = World {
            position: (WIDTH - 2, 4),
            ..World::default()
        };
        assert!(world.step(&[Instruction::Move]));
        assert_eq!(world.position, (WIDTH - 2, 4));
        assert!(!world.step(&[Instruction::Move]));
    }
    #[test]
    fn parser_accepts_comments_and_reports_source_line() {
        assert_eq!(
            parse("# hello\nmove # go\nTURN").unwrap(),
            vec![Instruction::Move, Instruction::Turn]
        );
        assert!(parse("MOVE\n\nBOOP").unwrap_err().starts_with("Line 3:"));
        assert!(parse("# nothing").is_err());
    }
}
