pub const WIDTH: i32 = 12;
pub const HEIGHT: i32 = 9;
pub const START: (i32, i32) = (2, 4);
pub const ORE: (i32, i32) = (7, 4);
pub const STARTING_ORE: u32 = 12;
pub const BATTERY: u32 = 40;
pub const LOW_BATTERY: u32 = 10;
pub const CHARGER_COST: u32 = 1;
pub const ROBOT_COST: u32 = 2;
pub const MAX_ROBOTS: usize = 4;
/// Control-flow steps a robot may take in one tick before it gives up.
const THINK_LIMIT: usize = 1000;
const PROGRAM_LIMIT: usize = 10000;
pub const DEMO: &str = "# Every robot runs this same program.
# Out along row 4, back along row 5,
# and into home from the west.
while(not ore)
  move()
end
collect()
turn(right)
while(robot)
  wait()
end
move()
turn(right)
while(not wall)
  move()
end
turn(right)
while(robot)
  wait()
end
move()
turn(right)
while(not home)
  move()
end
deposit()
turn(left)
if(not charger)
  build(charger)
end
charge()
turn(right)
if(id(1))
  build(robot)
end
repeat()";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    East,
    South,
    West,
    North,
}

impl Direction {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::East => (1, 0),
            Direction::South => (0, 1),
            Direction::West => (-1, 0),
            Direction::North => (0, -1),
        }
    }

    pub fn right(self) -> Self {
        match self {
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
            Direction::North => Direction::East,
        }
    }

    pub fn left(self) -> Self {
        self.right().right().right()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Building {
    Charger,
    Robot,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Sensor {
    Wall,
    Robot,
    Charger,
    Blocked,
    Ore,
    Home,
    Carrying,
    Low,
    Id(u32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Condition {
    pub sensor: Sensor,
    pub negated: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    Move,
    Turn,
    TurnLeft,
    Collect,
    Deposit,
    Charge,
    Wait,
    Build(Building),
    /// Continue at the given instruction index.
    Jump(usize),
    /// Continue at the given index unless the condition holds.
    JumpUnless(Condition, usize),
}

impl Op {
    fn is_control(self) -> bool {
        matches!(self, Op::Jump(_) | Op::JumpUnless(..))
    }
}

/// One executable step, tagged with the zero-based source line it came from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Instruction {
    pub op: Op,
    pub line: usize,
}

enum Block {
    If { branch: usize, exit: Option<usize> },
    While { start: usize, branch: usize },
}

fn set_target(op: &mut Op, target: usize) {
    if let Op::Jump(t) | Op::JumpUnless(_, t) = op {
        *t = target;
    }
}

fn emit(program: &mut Vec<Instruction>, line: usize, op: Op, count: usize) {
    program.extend(std::iter::repeat_n(Instruction { op, line }, count));
}

fn parse_condition(text: &str) -> Option<Condition> {
    let (negated, text) = match text.strip_prefix("not ") {
        Some(rest) => (true, rest.trim()),
        None => (false, text),
    };
    let sensor = match text {
        "wall" => Sensor::Wall,
        "robot" => Sensor::Robot,
        "charger" => Sensor::Charger,
        "blocked" => Sensor::Blocked,
        "ore" => Sensor::Ore,
        "home" => Sensor::Home,
        "carrying" => Sensor::Carrying,
        "low" => Sensor::Low,
        _ => {
            let id = text.strip_prefix("id(")?.strip_suffix(')')?;
            Sensor::Id(id.trim().parse().ok()?)
        }
    };
    Some(Condition { sensor, negated })
}

pub fn parse(source: &str) -> Result<Vec<Instruction>, String> {
    let mut program: Vec<Instruction> = Vec::new();
    let mut blocks: Vec<(usize, Block)> = Vec::new();
    for (line, raw) in source.lines().enumerate() {
        let raw = raw.split('#').next().unwrap_or("").trim();
        let text = raw.to_lowercase();
        if text.is_empty() {
            continue;
        }
        let error = || format!("Line {}: invalid instruction: {}", line + 1, raw);
        let count_error = || format!("Line {}: counts must be 1–100: {}", line + 1, raw);
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
                    return Err(count_error());
                }
                emit(&mut program, line, Op::Move, count);
            }
            "turn" => {
                let args = args.filter(|a| !a.is_empty()).unwrap_or("right");
                let parts: Vec<_> = args.split(',').map(str::trim).collect();
                // `around` is two right turns, so its count is in half turns.
                let (op, turns_per_count) = match parts[0] {
                    "right" => (Op::Turn, 1),
                    "left" => (Op::TurnLeft, 1),
                    "around" => (Op::Turn, 2),
                    _ => return Err(error()),
                };
                let count = match parts.as_slice() {
                    [_] => 1,
                    [_, count] => count.parse::<usize>().map_err(|_| error())?,
                    _ => return Err(error()),
                };
                if !(1..=100).contains(&count) {
                    return Err(count_error());
                }
                emit(&mut program, line, op, count * turns_per_count);
            }
            "collect" if empty() => emit(&mut program, line, Op::Collect, 1),
            "deposit" if empty() => emit(&mut program, line, Op::Deposit, 1),
            "charge" if empty() => emit(&mut program, line, Op::Charge, 1),
            "wait" if empty() => emit(&mut program, line, Op::Wait, 1),
            "repeat" if empty() => emit(&mut program, line, Op::Jump(0), 1),
            "build" if args == Some("charger") => {
                emit(&mut program, line, Op::Build(Building::Charger), 1)
            }
            "build" if args == Some("robot") => {
                emit(&mut program, line, Op::Build(Building::Robot), 1)
            }
            "if" | "while" => {
                let condition = args.and_then(parse_condition).ok_or_else(error)?;
                let branch = program.len();
                emit(&mut program, line, Op::JumpUnless(condition, 0), 1);
                let block = if name == "if" {
                    Block::If { branch, exit: None }
                } else {
                    Block::While {
                        start: branch,
                        branch,
                    }
                };
                blocks.push((line, block));
            }
            "else" if empty() => match blocks.last_mut() {
                Some((_, Block::If { branch, exit })) if exit.is_none() => {
                    let jump = program.len();
                    emit(&mut program, line, Op::Jump(0), 1);
                    set_target(&mut program[*branch].op, jump + 1);
                    *exit = Some(jump);
                }
                _ => return Err(format!("Line {}: else without a matching if", line + 1)),
            },
            "end" if empty() => match blocks.pop() {
                Some((_, Block::If { branch, exit })) => {
                    let end = program.len();
                    set_target(&mut program[exit.unwrap_or(branch)].op, end);
                }
                Some((_, Block::While { start, branch })) => {
                    emit(&mut program, line, Op::Jump(start), 1);
                    let end = program.len();
                    set_target(&mut program[branch].op, end);
                }
                None => return Err(format!("Line {}: end without an if or while", line + 1)),
            },
            _ => return Err(error()),
        }
        if program.len() > PROGRAM_LIMIT {
            return Err(format!("Line {}: program is too large", line + 1));
        }
    }
    if let Some((line, _)) = blocks.last() {
        return Err(format!("Line {}: this block needs an end", line + 1));
    }
    if program.iter().all(|i| i.op.is_control()) {
        return Err("Give your little robot an instruction.".into());
    }
    Ok(program)
}

/// The instruction a robot at `pc` will look at next, skipping unconditional jumps.
pub fn upcoming(program: &[Instruction], mut pc: usize) -> Option<&Instruction> {
    for _ in 0..THINK_LIMIT {
        match program.get(pc)? {
            Instruction {
                op: Op::Jump(target),
                ..
            } => pc = *target,
            instruction => return Some(instruction),
        }
    }
    None
}

pub fn wall(x: i32, y: i32) -> bool {
    x <= 0
        || y <= 0
        || x >= WIDTH - 1
        || y >= HEIGHT - 1
        || (x == 5 && (2..=3).contains(&y))
        || (x == 8 && y == 6)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Working,
    Finished,
    /// Spent a whole tick jumping around without reaching an action.
    Lost,
    PoweredDown,
}

#[derive(Clone, Debug)]
pub struct Robot {
    pub id: u32,
    pub position: (i32, i32),
    pub direction: Direction,
    pub carrying: bool,
    pub energy: u32,
    pub pc: usize,
    pub state: State,
    pub message: String,
}

impl Robot {
    pub fn new(id: u32, position: (i32, i32), direction: Direction) -> Self {
        Self {
            id,
            position,
            direction,
            carrying: false,
            energy: BATTERY,
            pc: 0,
            state: State::Working,
            message: "Ready. Enthusiasm: high. Judgment: yours.".into(),
        }
    }

    pub fn name(&self) -> String {
        format!("CL-{:02}", self.id)
    }

    fn ahead(&self) -> (i32, i32) {
        let (dx, dy) = self.direction.delta();
        (self.position.0 + dx, self.position.1 + dy)
    }
}

pub struct World {
    pub robots: Vec<Robot>,
    pub ore: u32,
    pub delivered: u32,
    pub chargers: Vec<(i32, i32)>,
    pub ticks: u64,
    pub message: String,
}

impl Default for World {
    fn default() -> Self {
        Self {
            robots: vec![Robot::new(1, START, Direction::East)],
            ore: STARTING_ORE,
            delivered: 0,
            chargers: Vec::new(),
            ticks: 0,
            message: "Meet CL-01. Enthusiasm: high. Judgment: yours.".into(),
        }
    }
}

impl World {
    /// Advances every working robot by one action. Returns false once nobody is working.
    pub fn step(&mut self, program: &[Instruction]) -> bool {
        if !self.robots.iter().any(|r| r.state == State::Working) {
            self.message = "Nobody is working. A remarkably quiet workplace.".into();
            return false;
        }
        self.ticks += 1;
        // Robots built during this tick start working on the next one.
        for i in 0..self.robots.len() {
            if self.robots[i].state != State::Working {
                continue;
            }
            let mut robot = self.robots[i].clone();
            self.run(&mut robot, program);
            self.robots[i] = robot;
        }
        true
    }

    fn robot_at(&self, position: (i32, i32), except: u32) -> Option<&Robot> {
        self.robots
            .iter()
            .find(|r| r.position == position && r.id != except)
    }

    fn run(&mut self, robot: &mut Robot, program: &[Instruction]) {
        for _ in 0..THINK_LIMIT {
            let Some(instruction) = program.get(robot.pc) else {
                robot.state = State::Finished;
                robot.message = "Program finished. Awaiting further genius.".into();
                return;
            };
            robot.pc += 1;
            match instruction.op {
                Op::Jump(target) => robot.pc = target,
                Op::JumpUnless(condition, target) => {
                    if !self.check(robot, condition) {
                        robot.pc = target;
                    }
                }
                op => {
                    robot.message = self.act(robot, op);
                    return;
                }
            }
        }
        robot.state = State::Lost;
        robot.message = "Lost in thought. So many decisions, zero actions.".into();
    }

    fn check(&self, robot: &Robot, condition: Condition) -> bool {
        let ahead = robot.ahead();
        let is_wall = wall(ahead.0, ahead.1);
        let is_charger = self.chargers.contains(&ahead);
        let is_robot = self.robot_at(ahead, robot.id).is_some();
        let value = match condition.sensor {
            Sensor::Wall => is_wall,
            Sensor::Robot => is_robot,
            Sensor::Charger => is_charger,
            Sensor::Blocked => is_wall || is_charger || is_robot,
            Sensor::Ore => robot.position == ORE,
            Sensor::Home => robot.position == START,
            Sensor::Carrying => robot.carrying,
            Sensor::Low => robot.energy < LOW_BATTERY,
            Sensor::Id(id) => robot.id == id,
        };
        value != condition.negated
    }

    fn act(&mut self, robot: &mut Robot, op: Op) -> String {
        let cost = match op {
            Op::Charge | Op::Wait => 0,
            _ => 1,
        };
        if robot.energy < cost {
            robot.state = State::PoweredDown;
            return format!("Battery empty. {} is now modern art.", robot.name());
        }
        robot.energy -= cost;
        let ahead = robot.ahead();
        match op {
            Op::Move => {
                if wall(ahead.0, ahead.1) {
                    "CLONK. That is a wall.".into()
                } else if self.chargers.contains(&ahead) {
                    "CLONK. That is a charger.".into()
                } else if let Some(other) = self.robot_at(ahead, robot.id) {
                    format!("CLONK. That is {}.", other.name())
                } else {
                    robot.position = ahead;
                    "Small steps. Questionable ambitions.".into()
                }
            }
            Op::Turn => {
                robot.direction = robot.direction.right();
                "Turning clockwise. A fresh perspective.".into()
            }
            Op::TurnLeft => {
                robot.direction = robot.direction.left();
                "Turning counterclockwise. Fancy.".into()
            }
            Op::Collect => {
                if robot.carrying {
                    "Pockets full. Pocket, singular.".into()
                } else if robot.position != ORE {
                    "No ore here. Confidence remains unaffected.".into()
                } else if self.ore == 0 {
                    "The mine is empty. Collecting air, thoroughly.".into()
                } else {
                    self.ore -= 1;
                    robot.carrying = true;
                    "Acquired: one extremely important rock.".into()
                }
            }
            Op::Deposit => {
                if robot.position == START && robot.carrying {
                    robot.carrying = false;
                    self.delivered += 1;
                    "Rock delivered. Industry has begun.".into()
                } else {
                    "Deposit requires a rock and the depot.".into()
                }
            }
            Op::Charge => {
                if self.chargers.contains(&ahead) {
                    robot.energy = BATTERY;
                    "Charged. Ready to make new mistakes.".into()
                } else {
                    "Charging requires facing a charger.".into()
                }
            }
            Op::Wait => "Waiting. Patiently. Mostly.".into(),
            Op::Build(building) => self.build(robot, building, ahead),
            Op::Jump(_) | Op::JumpUnless(..) => unreachable!("control flow is not an action"),
        }
    }

    fn build(&mut self, robot: &Robot, building: Building, target: (i32, i32)) -> String {
        let (name, cost) = match building {
            Building::Charger => ("charger", CHARGER_COST),
            Building::Robot => ("robot", ROBOT_COST),
        };
        if robot.position != START {
            "Build at home, facing an empty square.".into()
        } else if self.delivered < cost {
            format!("A {name} needs {cost} ore in the depot.")
        } else if building == Building::Robot && self.robots.len() >= MAX_ROBOTS {
            "The colony is full. Four is already a crowd.".into()
        } else if wall(target.0, target.1)
            || target == ORE
            || target == START
            || self.chargers.contains(&target)
            || self.robot_at(target, robot.id).is_some()
        {
            "Cannot build there. CLONK.".into()
        } else {
            self.delivered -= cost;
            match building {
                Building::Charger => {
                    self.chargers.push(target);
                    self.message = format!("{} built a charger. Batteries rejoice.", robot.name());
                    self.message.clone()
                }
                Building::Robot => {
                    let id = self.robots.len() as u32 + 1;
                    let new = Robot::new(id, target, robot.direction);
                    self.message =
                        format!("{} assembled. It has your code. Good luck.", new.name());
                    self.robots.push(new);
                    self.message.clone()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ops(source: &str) -> Vec<Op> {
        parse(source).unwrap().iter().map(|i| i.op).collect()
    }
    fn program(ops: &[Op]) -> Vec<Instruction> {
        ops.iter().map(|&op| Instruction { op, line: 0 }).collect()
    }
    fn total_ore(world: &World) -> u32 {
        world.ore
            + world.delivered
            + world.robots.iter().filter(|r| r.carrying).count() as u32
            + world.chargers.len() as u32 * CHARGER_COST
            + (world.robots.len() as u32 - 1) * ROBOT_COST
    }
    #[test]
    fn demo_grows_a_colony_and_conserves_ore() {
        let mut world = World::default();
        let program = parse(DEMO).unwrap();
        for _ in 0..3000 {
            assert!(world.step(&program));
            assert_eq!(total_ore(&world), STARTING_ORE);
        }
        assert_eq!(world.chargers, vec![(2, 3)]);
        assert_eq!(world.robots.len(), MAX_ROBOTS);
        assert_eq!(world.ore, 0);
        assert!(world.robots.iter().all(|r| r.state == State::Working));
    }
    #[test]
    fn upcoming_skips_unconditional_jumps() {
        let program = parse("move()\nwhile(not wall)\nmove()\nend\nrepeat()").unwrap();
        assert_eq!(upcoming(&program, 3).unwrap().line, 1);
        assert_eq!(upcoming(&program, 4).unwrap().line, 0);
        assert!(upcoming(&parse("move()").unwrap(), 1).is_none());
    }
    #[test]
    fn invalid_build_does_not_spend_ore() {
        let mut world = World::default();
        world.step(&program(&[Op::Build(Building::Charger)]));
        world.step(&program(&[Op::Build(Building::Robot)]));
        assert!(world.chargers.is_empty());
        assert_eq!(world.robots.len(), 1);
        assert_eq!(world.delivered, 0);
    }
    #[test]
    fn arguments_expand_and_old_commands_still_work() {
        use Op::*;
        assert_eq!(
            ops("move(2)\nturn(left, 2)\nturn(around)\nCOLLECT"),
            vec![Move, Move, TurnLeft, TurnLeft, Turn, Turn, Collect]
        );
        assert!(parse("move(0)").is_err());
        assert!(parse("turn(up)").is_err());
        assert!(parse("turn(, 2)").is_err());
        assert_eq!(ops("turn()"), vec![Turn]);
    }
    #[test]
    fn turn_around_count_is_in_half_turns() {
        assert_eq!(ops("turn(around, 2)"), vec![Op::Turn; 4]);
        assert_eq!(ops("turn(around, 1)"), vec![Op::Turn; 2]);
    }
    #[test]
    fn instructions_remember_their_source_line() {
        let lines: Vec<_> = parse("# hi\nmove(2)\n\nturn(around)")
            .unwrap()
            .iter()
            .map(|i| i.line)
            .collect();
        assert_eq!(lines, vec![1, 1, 3, 3]);
    }
    #[test]
    fn errors_quote_the_original_text() {
        assert_eq!(
            parse("MOVE(Fast)").unwrap_err(),
            "Line 1: invalid instruction: MOVE(Fast)"
        );
        assert!(parse("move(101)").unwrap_err().contains("1–100"));
    }
    #[test]
    fn programs_without_actions_are_rejected() {
        assert!(parse("repeat()").is_err());
        assert!(parse("repeat()\n# still nothing\nrepeat()").is_err());
        assert!(parse("while(not ore)\nend").is_err());
    }
    #[test]
    fn control_flow_compiles_to_jumps() {
        let wall = Condition {
            sensor: Sensor::Wall,
            negated: true,
        };
        let home = Condition {
            sensor: Sensor::Home,
            negated: false,
        };
        assert_eq!(
            ops("while(not wall)\nmove()\nend\nif(home)\ndeposit()\nelse\nwait()\nend"),
            vec![
                Op::JumpUnless(wall, 3),
                Op::Move,
                Op::Jump(0),
                Op::JumpUnless(home, 6),
                Op::Deposit,
                Op::Jump(7),
                Op::Wait,
            ]
        );
    }
    #[test]
    fn unbalanced_blocks_are_reported() {
        assert!(
            parse("if(home)\nmove()")
                .unwrap_err()
                .starts_with("Line 1:")
        );
        assert!(parse("move()\nend").unwrap_err().starts_with("Line 2:"));
        assert!(parse("move()\nelse").is_err());
        assert!(parse("if(home)\nmove\nelse\nmove\nelse\nend").is_err());
        assert!(parse("if(sunny)\nmove\nend").is_err());
        assert!(parse("if(id(two))\nmove\nend").is_err());
    }
    #[test]
    fn while_loops_follow_sensors() {
        let mut world = World::default();
        let program = parse("while(not wall)\nmove()\nend").unwrap();
        while world.step(&program) {}
        assert_eq!(world.robots[0].position, (WIDTH - 2, 4));
        assert_eq!(world.robots[0].state, State::Finished);
    }
    #[test]
    fn a_robot_that_never_acts_gets_lost() {
        let mut world = World::default();
        let program = parse("while(home)\nend\nmove()").unwrap();
        assert!(world.step(&program));
        assert_eq!(world.robots[0].state, State::Lost);
        assert!(!world.step(&program));
    }
    #[test]
    fn batteries_drain_and_chargers_refill_them() {
        let mut world = World::default();
        world.robots[0].direction = Direction::West;
        let drain = parse("move()\nrepeat()").unwrap();
        for _ in 0..BATTERY {
            assert!(world.step(&drain));
        }
        assert_eq!(world.robots[0].energy, 0);
        assert!(world.step(&drain));
        assert_eq!(world.robots[0].state, State::PoweredDown);
        assert!(!world.step(&drain));

        let mut world = World {
            chargers: vec![(3, 4)],
            ..World::default()
        };
        world.robots[0].energy = 1;
        world.step(&program(&[Op::Charge]));
        assert_eq!(world.robots[0].energy, BATTERY);
    }
    #[test]
    fn robots_block_each_other_and_know_who_they_are() {
        let mut world = World {
            delivered: 2,
            ..World::default()
        };
        let program =
            parse("if(id(1))\nbuild(robot)\nwhile(robot)\nwait()\nend\nend\nmove()").unwrap();
        world.step(&program);
        assert_eq!(world.robots.len(), 2);
        assert_eq!(world.robots[1].position, (3, 4));
        assert_eq!(world.delivered, 0);
        // CL-01 waits while CL-02 skips the build and moves away.
        world.step(&program);
        assert_eq!(world.robots[0].position, START);
        assert_eq!(world.robots[1].position, (4, 4));
        world.step(&program);
        assert_eq!(world.robots[0].position, (3, 4));
    }
    #[test]
    fn moving_into_a_robot_is_a_clonk() {
        let mut world = World::default();
        world.robots.push(Robot::new(2, (3, 4), Direction::North));
        world.robots[1].state = State::PoweredDown;
        world.step(&program(&[Op::Move]));
        assert_eq!(world.robots[0].position, START);
        assert_eq!(world.robots[0].message, "CLONK. That is CL-02.");
    }
    #[test]
    fn collect_and_deposit_need_the_right_place() {
        let mut world = World::default();
        world.step(&program(&[Op::Collect]));
        assert!(!world.robots[0].carrying);
        world.robots[0].position = ORE;
        world.robots[0].pc = 0;
        world.step(&program(&[Op::Collect]));
        assert!(world.robots[0].carrying);
        world.robots[0].pc = 0;
        world.step(&program(&[Op::Deposit]));
        assert!(world.robots[0].carrying);
        assert_eq!(world.delivered, 0);
    }
    #[test]
    fn empty_mine_yields_nothing() {
        let mut world = World {
            ore: 0,
            ..World::default()
        };
        world.robots[0].position = ORE;
        world.step(&program(&[Op::Collect]));
        assert!(!world.robots[0].carrying);
    }
    #[test]
    fn wall_blocks_motion_and_program_terminates() {
        let mut world = World::default();
        world.robots[0].position = (WIDTH - 2, 4);
        let program = program(&[Op::Move]);
        assert!(world.step(&program));
        assert_eq!(world.robots[0].position, (WIDTH - 2, 4));
        assert!(world.step(&program));
        assert_eq!(world.robots[0].state, State::Finished);
        assert!(!world.step(&program));
    }
    #[test]
    fn parser_accepts_comments_and_reports_source_line() {
        assert_eq!(ops("# hello\nmove # go\nTURN"), vec![Op::Move, Op::Turn]);
        assert!(parse("MOVE\n\nBOOP").unwrap_err().starts_with("Line 3:"));
        assert!(parse("# nothing").is_err());
    }
}
