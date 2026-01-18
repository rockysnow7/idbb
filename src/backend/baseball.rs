use rand::prelude::*;
use std::collections::HashMap;
use strum::IntoEnumIterator;

#[derive(Copy, Clone)]
pub enum SkillLevel {
    VeryHigh,
    High,
    Medium,
    Low,
    VeryLow,
}

impl Into<f64> for SkillLevel {
    fn into(self) -> f64 {
        match self {
            SkillLevel::VeryHigh => 1.0,
            SkillLevel::High => 0.75,
            SkillLevel::Medium => 0.5,
            SkillLevel::Low => 0.25,
            SkillLevel::VeryLow => 0.0,
        }
    }
}

pub struct PlayerMetrics {
    hitting: SkillLevel,
    running: SkillLevel,
    fielding: SkillLevel,
    pitching: SkillLevel,
}

pub struct Player {
    name: String,
    metrics: PlayerMetrics,
}

pub struct Team {
    name: String,
    batting_order: Vec<String>,
    starting_pitcher: String,
    fielders: Vec<String>,
}

struct HalfInning {
    number: u8,
    top: bool,
}

impl HalfInning {
    pub fn next(&self) -> Self {
        match self.top {
            true => Self { number: self.number, top: false },
            false => Self { number: self.number + 1, top: true },
        }
    }
}

struct Bases {
    first: Option<String>,
    second: Option<String>,
    third: Option<String>,
}

impl Bases {
    pub fn empty() -> Self {
        Self { first: None, second: None, third: None }
    }
}

struct Count {
    balls: u8,
    strikes: u8,
}

impl Count {
    pub fn empty() -> Self {
        Self { balls: 0, strikes: 0 }
    }
}

struct GameState {
    home_team_runs: u8,
    visiting_team_runs: u8,
    half_inning: HalfInning,
    bases: Bases,
    outs: u8,
    count: Count,
    game_outcome: GameOutcome,
}

impl GameState {
    pub fn start_of_game() -> Self {
        Self {
            home_team_runs: 0,
            visiting_team_runs: 0,
            half_inning: HalfInning { number: 1, top: true },
            bases: Bases::empty(),
            outs: 0,
            count: Count::empty(),
            game_outcome: GameOutcome::Ongoing,
        }
    }
}

#[derive(strum::EnumIter, Copy, Clone)]
pub enum StrikeZoneLocation {
    In,
    Out,
}

#[derive(strum::EnumIter, Copy, Clone)]
pub enum BatterDecision {
    Swing,
    Take,
}

#[derive(strum::EnumIter, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FieldLocation {
    Close,
    Infield,
    Outfield,
    OutOfPark,
}

impl FieldLocation {
    fn next(&self) -> Self {
        match self {
            Self::Close => Self::Infield,
            Self::Infield => Self::Outfield,
            Self::Outfield => Self::OutOfPark,
            Self::OutOfPark => unreachable!(),
        }
    }

    pub fn random_from_skill(&self, rng: &mut ThreadRng, skill: f64) -> Self {
        if let Self::OutOfPark = self {
            *self
        } else if rng.random_bool(skill) {
            self.next().random_from_skill(rng, skill)
        } else {
            *self
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Base {
    Batting,
    First,
    Second,
    Third,
    Home,
}

impl Base {
    fn next(&self) -> Option<Self> {
        match self {
            Self::Batting => Some(Self::First),
            Self::First => Some(Self::Second),
            Self::Second => Some(Self::Third),
            Self::Third => Some(Self::Home),
            Self::Home => None,
        }
    }

    fn prev(&self) -> Option<Self> {
        match self {
            Self::Batting => None,
            Self::First => Some(Self::Batting),
            Self::Second => Some(Self::First),
            Self::Third => Some(Self::Second),
            Self::Home => Some(Self::Third),
        }
    }

    fn plus(&self, bases: u8) -> Option<Self> {
        if bases == 0 {
            Some(*self)
        } else {
            self.next().and_then(|base| base.plus(bases - 1))
        }
    }

    fn minus(&self, bases: u8) -> Option<Self> {
        if bases == 0 {
            Some(*self)
        } else {
            self.prev().and_then(|base| base.minus(bases - 1))
        }
    }
}

pub struct RunnerAdvancement {
    name: String,
    from_base: Base,
    to_base: Option<Base>, // if None, the runner is out
}

pub enum BattingOutcome {
    Strike,
    Ball,
    Hit {
        to_field_location: FieldLocation,
    },
}

pub enum AtBatOutcome {
    Strikeout,
    Walk,
    Single,
    Double,
    Triple,
    HomeRun,
    Out,
}

#[derive(Copy, Clone)]
pub enum GameOutcome {
    HomeTeamWins,
    VisitingTeamWins,
    Ongoing,
}

pub struct EventsSummary {
    pitch_location: StrikeZoneLocation,
    batter_decision: BatterDecision,
    batting_outcome: BattingOutcome,
    at_bat_outcome: Option<AtBatOutcome>,
    runner_advancements: Vec<RunnerAdvancement>,
    game_outcome: GameOutcome,
}

pub struct BaseballGame {
    rng: ThreadRng,
    state: GameState,
    all_players: HashMap<String, Player>,
    home_team: Team,
    visiting_team: Team,
}

impl BaseballGame {
    pub fn new(all_players: HashMap<String, Player>, home_team: Team, visiting_team: Team) -> Self {
        Self {
            rng: rand::rng(),
            state: GameState::start_of_game(),
            all_players,
            home_team,
            visiting_team,
        }
    }

    fn walk_advancements(&self, batter_name: &String) -> Vec<RunnerAdvancement> {
        let mut runner_advancements = Vec::new();
        runner_advancements.push(RunnerAdvancement {
            name: batter_name.clone(),
            from_base: Base::Batting,
            to_base: Some(Base::First),
        });
        if let Some(runner) = self.state.bases.first.clone() {
            runner_advancements.push(RunnerAdvancement {
                name: runner,
                from_base: Base::First,
                to_base: Some(Base::Second),
            });
            if let Some(runner) = self.state.bases.second.clone() {
                runner_advancements.push(RunnerAdvancement {
                    name: runner,
                    from_base: Base::Second,
                    to_base: Some(Base::Third),
                });
                if let Some(runner) = self.state.bases.third.clone() {
                    runner_advancements.push(RunnerAdvancement {
                        name: runner,
                        from_base: Base::Third,
                        to_base: Some(Base::Home),
                    });
                }
            }
        }

        runner_advancements
    }

    fn apply_runner_advancements(&mut self, runner_advancements: &Vec<RunnerAdvancement>) {
        let mut runner_advancements_sorted = Vec::new();
        while runner_advancements_sorted.len() < runner_advancements.len() {
            let advancement = runner_advancements.iter().max_by_key(|advancement| advancement.from_base);
            runner_advancements_sorted.push(advancement.unwrap());
        }

        for advancement in runner_advancements_sorted {
            match advancement.to_base {
                Some(Base::Home) => {
                    if self.state.half_inning.top {
                        self.state.visiting_team_runs += 1;
                    } else {
                        self.state.home_team_runs += 1;
                    }
                },
                Some(Base::Third) => self.state.bases.third = Some(advancement.name.clone()),
                Some(Base::Second) => self.state.bases.second = Some(advancement.name.clone()),
                Some(Base::First) => self.state.bases.first = Some(advancement.name.clone()),
                Some(Base::Batting) => unreachable!(),
                None => self.state.outs += 1,
            }

            match advancement.from_base {
                Base::Batting => (),
                Base::First => self.state.bases.first = None,
                Base::Second => self.state.bases.second = None,
                Base::Third => self.state.bases.third = None,
                Base::Home => unreachable!(),
            }
        }
    }

    fn game_outcome(&self) -> GameOutcome {
        if self.state.half_inning.number <= 9 {
            GameOutcome::Ongoing
        } else if self.state.home_team_runs > self.state.visiting_team_runs {
            GameOutcome::HomeTeamWins
        } else if self.state.home_team_runs < self.state.visiting_team_runs {
            GameOutcome::VisitingTeamWins
        } else {
            GameOutcome::Ongoing
        }
    }

    fn simulate_fielding_and_running(&mut self, batter_name: &String, field_location: FieldLocation) -> (AtBatOutcome, Vec<RunnerAdvancement>) {
        // 1. determine how far the batter gets, if at all
        // 2. determine how far the runners get, if at all
        // 3. apply the transitions to the game state
        // 4. return the at-bat outcome and runner advancements

        // handle home run
        if let FieldLocation::OutOfPark = field_location {
            let mut runner_advancements = Vec::new();
            runner_advancements.push(RunnerAdvancement {
                name: batter_name.clone(),
                from_base: Base::Batting,
                to_base: Some(Base::Home),
            });
            if let Some(runner) = self.state.bases.third.clone() {
                runner_advancements.push(RunnerAdvancement {
                    name: runner,
                    from_base: Base::Third,
                    to_base: Some(Base::Home),
                });
            }
            if let Some(runner) = self.state.bases.second.clone() {
                runner_advancements.push(RunnerAdvancement {
                    name: runner,
                    from_base: Base::Second,
                    to_base: Some(Base::Home),
                });
            }
            if let Some(runner) = self.state.bases.first.clone() {
                runner_advancements.push(RunnerAdvancement {
                    name: runner,
                    from_base: Base::First,
                    to_base: Some(Base::Home),
                });
            }

            self.apply_runner_advancements(&runner_advancements);

            return (AtBatOutcome::HomeRun, runner_advancements);
        }

        // handle runners advancing
        let mut runner_advancements = Vec::new();

        let mean_fielder_skill = self.visiting_team.fielders.iter().map(|fielder| {
            let fielder_skill: f64 = self.all_players.get(fielder).unwrap().metrics.fielding.into();
            fielder_skill
        }).sum::<f64>() / self.visiting_team.fielders.len() as f64;

        if let Some(runner) = self.state.bases.third.clone() {
            let runner_skill: f64 = self.all_players.get(&runner).unwrap().metrics.running.into();
            let success_prob = runner_skill / (runner_skill + mean_fielder_skill);

            let mut bases = 0;
            for _ in 0..=1 {
                if self.rng.random_bool(success_prob) {
                    bases += 1;
                } else {
                    break;
                }
            }

            let to_base = if bases > 0 {
                Some(Base::Third.plus(bases).unwrap())
            } else {
                None
            };

            runner_advancements.push(RunnerAdvancement {
                name: runner,
                from_base: Base::Third,
                to_base,
            });
        }
        if let Some(runner) = self.state.bases.second.clone() {
            let runner_skill: f64 = self.all_players.get(&runner).unwrap().metrics.running.into();
            let success_prob = runner_skill / (runner_skill + mean_fielder_skill);

            let upper_bound = {
                let min_advancement = runner_advancements.iter().min_by_key(|advancement| advancement.to_base);
                match min_advancement {
                    None => 2,
                    Some(advancement) => match advancement.to_base {
                        Some(Base::Home) => 2,
                        _ => 0,
                    },
                }
            };
            let mut bases = 0;
            for _ in 0..=upper_bound {
                if self.rng.random_bool(success_prob) {
                    bases += 1;
                } else {
                    break;
                }
            }
            let to_base = if bases > 0 {
                Some(Base::Second.plus(bases).unwrap())
            } else {
                None
            };

            runner_advancements.push(RunnerAdvancement {
                name: runner,
                from_base: Base::Second,
                to_base,
            });
        }
        if let Some(runner) = self.state.bases.first.clone() {
            let runner_skill: f64 = self.all_players.get(&runner).unwrap().metrics.running.into();
            let success_prob = runner_skill / (runner_skill + mean_fielder_skill);

            let upper_bound = {
                let min_advancement = runner_advancements.iter().min_by_key(|advancement| advancement.to_base);
                match min_advancement {
                    None => 3,
                    Some(advancement) => match advancement.to_base {
                        Some(Base::Home) => 3,
                        Some(Base::Third) => 1,
                        _ => 0,
                    },
                }
            };
            let mut bases = 0;
            for _ in 0..=upper_bound {
                if self.rng.random_bool(success_prob) {
                    bases += 1;
                } else {
                    break;
                }
            }
            let to_base = if bases > 0 {
                Some(Base::First.plus(bases).unwrap())
            } else {
                None
            };

            runner_advancements.push(RunnerAdvancement {
                name: runner,
                from_base: Base::First,
                to_base,
            });
        }

        // handle batter running
        let batter_running_skill: f64 = self.all_players.get(batter_name).unwrap().metrics.running.into();
        let batter_advance_prob = batter_running_skill / (batter_running_skill + mean_fielder_skill);

        let upper_bound = {
            let min_advancement = runner_advancements.iter().min_by_key(|advancement| advancement.to_base);
            match min_advancement {
                None => 4,
                Some(advancement) => match advancement.to_base {
                    Some(Base::Home) => 4,
                    Some(Base::Third) => 2,
                    Some(Base::Second) => 1,
                    _ => 0,
                },
            }
        };
        let mut batter_bases = 0;
        for _ in 0..=upper_bound {
            if self.rng.random_bool(batter_advance_prob) {
                batter_bases += 1;
            } else {
                break;
            }
        }
        let to_base = if batter_bases > 0 {
            Some(Base::Batting.plus(batter_bases).unwrap())
        } else {
            None
        };
        runner_advancements.push(RunnerAdvancement {
            name: batter_name.clone(),
            from_base: Base::Batting,
            to_base,
        });
        let at_bat_outcome = match batter_bases {
            0 => AtBatOutcome::Out,
            1 => AtBatOutcome::Single,
            2 => AtBatOutcome::Double,
            3 => AtBatOutcome::Triple,
            4 => AtBatOutcome::HomeRun,
            _ => unreachable!(),
        };

        self.apply_runner_advancements(&runner_advancements);

        (at_bat_outcome, runner_advancements)
    }

    pub fn simulate_pitch(
        &mut self,
        pitcher_name: &String,
        batter_name: &String,
        pitch_aim_location: Option<StrikeZoneLocation>, // if Some, the pitcher will aim for the given location; if None, the pitcher will throw a random pitch
        batter_decision: Option<BatterDecision>, // if Some, the batter will follow the given swing decision; if None, the batter will decide to swing/take/bunt randomly
    ) -> EventsSummary {
        // 1. decide if the pitch is in or out of the strike zone
        // 2. decide if the batter swings/takes
        // 3. determine if the pitch is a strike/ball/hit
        // 4. if the pitch is a hit, determine the fielding and running outcomes
        // 5. apply the outcomes to the game state
        // 6. return a summary of what has happened

        let pitch_aim_location = pitch_aim_location.unwrap_or(StrikeZoneLocation::iter().choose(&mut self.rng).unwrap());
        let pitcher_skill: f64 = self.all_players.get(pitcher_name).unwrap().metrics.pitching.into();
        let pitcher_succeeds = self.rng.random_bool(pitcher_skill);
        let pitch_location = match (pitch_aim_location, pitcher_succeeds) {
            (StrikeZoneLocation::In, true) => StrikeZoneLocation::In,
            (StrikeZoneLocation::In, false) => StrikeZoneLocation::Out,
            (StrikeZoneLocation::Out, true) => StrikeZoneLocation::Out,
            (StrikeZoneLocation::Out, false) => StrikeZoneLocation::In,
        };

        let batter_decision = batter_decision.unwrap_or(BatterDecision::iter().choose(&mut self.rng).unwrap());
        let batter_skill: f64 = self.all_players.get(batter_name).unwrap().metrics.hitting.into();
        let mut events_summary = match (pitch_location, batter_decision) {
            (_, BatterDecision::Swing) => {
                let contact = self.rng.random_bool(batter_skill);
                if contact { // swing and contact
                    let field_location = FieldLocation::Close.random_from_skill(&mut self.rng, batter_skill);
                    let (at_bat_outcome, runner_advancements) = self.simulate_fielding_and_running(batter_name, field_location);

                    EventsSummary {
                        pitch_location,
                        batter_decision,
                        batting_outcome: BattingOutcome::Hit { to_field_location: field_location },
                        at_bat_outcome: Some(at_bat_outcome),
                        runner_advancements,
                        game_outcome: GameOutcome::Ongoing,
                    }
                } else { // swing and miss
                    self.state.count.strikes += 1;

                    EventsSummary {
                        pitch_location,
                        batter_decision,
                        batting_outcome: BattingOutcome::Strike,
                        at_bat_outcome: None,
                        runner_advancements: Vec::new(),
                        game_outcome: GameOutcome::Ongoing,
                    }
                }
            },
            (StrikeZoneLocation::In, BatterDecision::Take) => { // strike looking
                self.state.count.strikes += 1;

                EventsSummary {
                    pitch_location,
                    batter_decision,
                    batting_outcome: BattingOutcome::Strike,
                    at_bat_outcome: None,
                    runner_advancements: Vec::new(),
                    game_outcome: GameOutcome::Ongoing,
                }
            },
            (StrikeZoneLocation::Out, BatterDecision::Take) => {
                self.state.count.balls += 1;

                EventsSummary {
                    pitch_location,
                    batter_decision,
                    batting_outcome: BattingOutcome::Ball,
                    at_bat_outcome: None,
                    runner_advancements: Vec::new(),
                    game_outcome: GameOutcome::Ongoing,
                }
            },
        };

        if self.state.count.strikes == 3 {
            self.state.outs += 1;
            events_summary.at_bat_outcome = Some(AtBatOutcome::Strikeout);
        } else if self.state.count.balls == 4 {
            events_summary.at_bat_outcome = Some(AtBatOutcome::Walk);
            let walk_advancements = self.walk_advancements(batter_name);
            self.apply_runner_advancements(&walk_advancements);
            events_summary.runner_advancements = walk_advancements;
        }

        if self.state.outs == 3 {
            self.state.half_inning = self.state.half_inning.next();
        }

        self.state.game_outcome = self.game_outcome();
        events_summary.game_outcome = self.state.game_outcome.clone();

        events_summary
    }
}
