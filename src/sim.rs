pub const WIDTH: i32 = 12;
pub const HEIGHT: i32 = 9;
pub const START: (i32, i32) = (2, 4);
pub const ORE: (i32, i32) = (7, 4);
pub const DEMO: &str = "# CL-01 builds its first charger!\nmove(5)\ncollect()\nturn(around)\nmove(5)\ndeposit()\nturn(right)\nbuild(charger)";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Instruction {
    Move,
    Turn,
    TurnLeft,
    Collect,
    Deposit,
    Repeat,
    BuildCharger,
}

pub fn parse(source: &str) -> Result<Vec<Instruction>, String> {
    let mut program = Vec::new();
    for (line, text) in source.lines().enumerate() {
        let text = text.split('#').next().unwrap_or("").trim().to_lowercase();
        if text.is_empty() {
            continue;
        }
        let error = || format!("Line {}: invalid instruction: {}", line + 1, text);
        let (name, args) = if let Some((name, tail)) = text.split_once('(') {
            let args = tail.strip_suffix(')').ok_or_else(error)?;
            (name.trim(), Some(args.trim()))
        } else {
            (text.as_str(), None)
        };
        let empty = || args.is_none_or(|a| a.is_empty());
        match name {
            "move" => {
                let count = match args {
                    None | Some("") => 1,
                    Some(a) => a.parse::<usize>().map_err(|_| error())?,
                };
                if !(1..=100).contains(&count) {
                    return Err(error());
                }
                program.extend(std::iter::repeat_n(Instruction::Move, count));
            }
            "turn" => {
                let parts: Vec<_> = args.unwrap_or("right").split(',').map(str::trim).collect();
                let (instruction, default_count) = match parts[0] {
                    "right" | "" => (Instruction::Turn, 1),
                    "left" => (Instruction::TurnLeft, 1),
                    "around" => (Instruction::Turn, 2),
                    _ => return Err(error()),
                };
                if parts.len() > 2 {
                    return Err(error());
                }
                let count = if parts.len() == 2 {
                    parts[1].parse::<usize>().map_err(|_| error())?
                } else {
                    default_count
                };
                if !(1..=100).contains(&count) {
                    return Err(error());
                }
                program.extend(std::iter::repeat_n(instruction, count));
            }
            "collect" if empty() => program.push(Instruction::Collect),
            "deposit" if empty() => program.push(Instruction::Deposit),
            "repeat" if empty() => program.push(Instruction::Repeat),
            "build" if args == Some("charger") => program.push(Instruction::BuildCharger),
            _ => return Err(error()),
        }
        if program.len() > 10000 {
            return Err(format!("Line {}: program is too large", line + 1));
        }
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
    pub charger: Option<(i32, i32)>,
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
            charger: None,
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
                if wall(next.0, next.1) || self.charger == Some(next) {
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
            Instruction::TurnLeft => {
                self.direction = (self.direction + 3) % 4;
                "Turning counterclockwise. Fancy."
            }
            Instruction::BuildCharger => {
                let delta = [(1, 0), (0, 1), (-1, 0), (0, -1)][self.direction];
                let target = (self.position.0 + delta.0, self.position.1 + delta.1);
                if self.position != START {
                    "Build at home, facing an empty square."
                } else if self.delivered < 1 {
                    "A charger needs one ore in the depot."
                } else if self.charger.is_some() {
                    "One charger is plenty for now."
                } else if wall(target.0, target.1) || target == ORE || target == START {
                    "Cannot build there. CLONK."
                } else {
                    self.delivered -= 1;
                    self.charger = Some(target);
                    "Charger built! Power functionality coming soon."
                }
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
    fn demo_builds_charger_and_conserves_ore() {
        let mut world = World::default();
        let program = parse(DEMO).unwrap();
        while world.step(&program) {}
        assert_eq!(world.charger, Some((2, 3)));
        assert_eq!(
            world.ore
                + world.delivered
                + u32::from(world.carrying)
                + u32::from(world.charger.is_some()),
            8
        );
    }
    #[test]
    fn invalid_build_does_not_spend_ore() {
        let mut world = World::default();
        world.step(&[Instruction::BuildCharger]);
        assert_eq!(world.charger, None);
        assert_eq!(world.delivered, 0);
    }
    #[test]
    fn arguments_expand_and_old_commands_still_work() {
        assert_eq!(
            parse("move(2)\nturn(left, 2)\nturn(around)\nCOLLECT").unwrap(),
            vec![
                Instruction::Move,
                Instruction::Move,
                Instruction::TurnLeft,
                Instruction::TurnLeft,
                Instruction::Turn,
                Instruction::Turn,
                Instruction::Collect
            ]
        );
        assert!(parse("move(0)").is_err());
        assert!(parse("turn(up)").is_err());
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
