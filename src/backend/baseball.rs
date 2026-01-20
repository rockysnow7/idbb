use rand::prelude::*;
use std::collections::HashMap;
use strum::IntoEnumIterator;

#[derive(Copy, Clone, Debug)]
pub enum Level {
    VeryHigh,
    High,
    Medium,
    Low,
    VeryLow,
}

impl Into<f64> for Level {
    fn into(self) -> f64 {
        match self {
            Level::VeryHigh => 1.0,
            Level::High => 0.75,
            Level::Medium => 0.5,
            Level::Low => 0.25,
            Level::VeryLow => 0.0,
        }
    }
}

#[derive(Debug)]
pub struct PlayerMetrics {
    hitting: Level,
    running: Level,
    fielding: Level,
    pitching: Level,
}

impl PlayerMetrics {
    pub fn random(rng: &mut ThreadRng) -> Self {
        let allowed_levels = [Level::High, Level::Medium, Level::Low];
        let hitting = *allowed_levels.choose(rng).unwrap();
        let running = *allowed_levels.choose(rng).unwrap();
        let fielding = *allowed_levels.choose(rng).unwrap();
        let pitching = *allowed_levels.choose(rng).unwrap();

        Self { hitting, running, fielding, pitching }
    }
}

#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub metrics: PlayerMetrics,
}

#[derive(Debug)]
pub struct Team {
    pub name: String,
    pub batting_order: [String; 9],
    pub current_pitcher: String,
    pub fielders: [String; 8], // does not include the pitcher
    pub bullpen: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HalfInning {
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

#[derive(Debug, Clone)]
pub struct Bases {
    first: Option<String>,
    second: Option<String>,
    third: Option<String>,
}

impl Bases {
    pub fn empty() -> Self {
        Self { first: None, second: None, third: None }
    }
}

#[derive(Debug, Clone)]
pub struct Count {
    balls: u8,
    strikes: u8,
}

impl Count {
    pub fn empty() -> Self {
        Self { balls: 0, strikes: 0 }
    }
}

#[derive(Debug)]
pub struct GameState {
    home_team_runs: u8,
    visiting_team_runs: u8,
    half_inning: HalfInning,
    last_inning_just_ended: bool,
    bases: Bases,
    outs: u8,
    count: Count,
    pub game_outcome: GameOutcome,
    home_team_batter_index: usize,
    visiting_team_batter_index: usize,
}

impl GameState {
    pub fn start_of_game() -> Self {
        Self {
            home_team_runs: 0,
            visiting_team_runs: 0,
            half_inning: HalfInning { number: 1, top: true },
            last_inning_just_ended: false,
            bases: Bases::empty(),
            outs: 0,
            count: Count::empty(),
            game_outcome: GameOutcome::Ongoing,
            home_team_batter_index: 0,
            visiting_team_batter_index: 0,
        }
    }
}

#[derive(strum::EnumIter, Copy, Clone, Debug)]
pub enum StrikeZoneLocation {
    In,
    Out,
}

#[derive(strum::EnumIter, Copy, Clone, Debug)]
pub enum BatterDecision {
    Swing,
    Take,
}

#[derive(strum::EnumIter, Copy, Clone, Hash, PartialEq, Eq, Debug)]
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
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

#[derive(PartialEq, Clone, Debug)]
pub struct RunnerAdvancement {
    name: String,
    from_base: Base,
    to_base: Option<Base>, // if None, the runner is out
}

#[derive(Debug)]
pub enum BattingOutcome {
    Strike,
    Ball,
    Hit {
        to_field_location: FieldLocation,
    },
}

#[derive(Debug)]
pub enum AtBatOutcome {
    Strikeout,
    Walk,
    Single,
    Double,
    Triple,
    HomeRun,
    Out,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum GameOutcome {
    HomeTeamWins,
    VisitingTeamWins,
    Ongoing,
}

#[derive(Debug)]
pub struct GameStateSummary {
    pub home_team_runs: u8,
    pub visiting_team_runs: u8,
    pub half_inning: HalfInning,
    pub bases: Bases,
    pub outs: u8,
    pub count: Count,
    pub batter: String,
    pub pitcher: String,
}

#[derive(Debug)]
pub struct EventsSummary {
    pitch_location: StrikeZoneLocation,
    batter_decision: BatterDecision,
    batting_outcome: BattingOutcome,
    at_bat_outcome: Option<AtBatOutcome>,
    runner_advancements: Vec<RunnerAdvancement>,
    game_outcome: GameOutcome,
}

fn random_advancement_between(from_base: Base, to_base: Base, success_prob: f64, rng: &mut ThreadRng) -> Option<Base> {
    let max_bases = match (from_base, to_base) {
        (Base::Batting, Base::First) => 1,
        (Base::Batting, Base::Second) => 2,
        (Base::Batting, Base::Third) => 3,
        (Base::Batting, Base::Home) => 4,
        (Base::First, Base::Second) => 1,
        (Base::First, Base::Third) => 2,
        (Base::First, Base::Home) => 3,
        (Base::Second, Base::Third) => 1,
        (Base::Second, Base::Home) => 2,
        (Base::Third, Base::Home) => 1,
        _ => unreachable!(),
    };

    let mut bases = 0;
    for _ in 0..=max_bases {
        if rng.random_bool(success_prob) {
            bases += 1;
        } else {
            break;
        }
    }

    if bases > 0 {
        Some(from_base.plus(bases).unwrap())
    } else {
        None
    }
}

#[derive(Debug)]
pub struct BaseballGame {
    rng: ThreadRng,
    pub state: GameState,
    all_players: HashMap<String, Player>,
    pub home_team: Team,
    pub visiting_team: Team,
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

    pub fn home_team_is_at_bat(&self) -> bool {
        !self.state.half_inning.top
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

    fn apply_runner_advancements(&mut self, runner_advancements: &mut Vec<RunnerAdvancement>) {
        if runner_advancements.is_empty() {
            return;
        }
        let mut runner_advancements_sorted = runner_advancements.clone();
        runner_advancements_sorted.sort_by_key(|advancement| advancement.from_base);

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
                Base::Batting => if self.state.half_inning.top {
                    self.state.visiting_team_batter_index = (self.state.visiting_team_batter_index + 1) % 9;
                } else {
                    self.state.home_team_batter_index = (self.state.home_team_batter_index + 1) % 9;
                },
                Base::First => self.state.bases.first = None,
                Base::Second => self.state.bases.second = None,
                Base::Third => self.state.bases.third = None,
                Base::Home => unreachable!(),
            }
        }
    }

    fn game_outcome(&self) -> GameOutcome {
        // first 8 innings
        if self.state.half_inning.number < 9 {
            return GameOutcome::Ongoing;
        }
        // walk-offs
        if self.state.half_inning.number == 9 && !self.state.half_inning.top {
            if self.state.home_team_runs > self.state.visiting_team_runs {
                return GameOutcome::HomeTeamWins;
            }
        }
        // extra innings
        if self.state.half_inning.number > 9 && self.state.half_inning.top && self.state.visiting_team_runs > self.state.home_team_runs && self.state.last_inning_just_ended {
            return GameOutcome::VisitingTeamWins;
        }
        GameOutcome::Ongoing
    }

    fn simulate_fielding_and_running(&mut self, batter_name: &String, field_location: FieldLocation) -> (AtBatOutcome, Vec<RunnerAdvancement>) {
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

            self.apply_runner_advancements(&mut runner_advancements);

            return (AtBatOutcome::HomeRun, runner_advancements);
        }

        let first_forced = self.state.bases.first.is_some();
        let second_forced = self.state.bases.first.is_some() && self.state.bases.second.is_some();
        let third_forced = self.state.bases.first.is_some() && self.state.bases.second.is_some() && self.state.bases.third.is_some();

        let distance_difficulty: f64 = match field_location {
            FieldLocation::Close => Level::High,
            FieldLocation::Infield => Level::Medium,
            FieldLocation::Outfield => Level::Low,
            FieldLocation::OutOfPark => unreachable!(),
        }.into();
        let mean_fielder_skill = self.visiting_team.fielders.iter().map(|fielder| {
            let fielder_skill: f64 = self.all_players.get(fielder).unwrap().metrics.fielding.into();
            fielder_skill
        }).sum::<f64>() / self.visiting_team.fielders.len() as f64;
        let overall_difficulty = distance_difficulty + mean_fielder_skill;

        let mut runner_advancements = Vec::new();
        let mut furthest_occupied = Base::Home;

        // advance runners
        if let Some(runner) = self.state.bases.third.clone() {
            let runner_skill: f64 = self.all_players.get(&runner).unwrap().metrics.running.into();
            let success_prob = runner_skill / (runner_skill + overall_difficulty);

            if third_forced { // runner is forced to advance
                if self.rng.random_bool(success_prob) {
                    runner_advancements.push(RunnerAdvancement {
                        name: runner,
                        from_base: Base::Third,
                        to_base: Some(Base::Home),
                    });
                } else { // force out
                    runner_advancements.push(RunnerAdvancement {
                        name: runner,
                        from_base: Base::Third,
                        to_base: None,
                    });
                    furthest_occupied = Base::Third;
                }
            } else if success_prob >= 0.5 { // runner may choose to advance if success probability is high enough
                if self.rng.random_bool(success_prob) {
                    runner_advancements.push(RunnerAdvancement {
                        name: runner,
                        from_base: Base::Third,
                        to_base: Some(Base::Home),
                    });
                } else {
                    runner_advancements.push(RunnerAdvancement {
                        name: runner,
                        from_base: Base::Third,
                        to_base: None,
                    });
                    furthest_occupied = Base::Third;
                }
            } else {
                furthest_occupied = Base::Third;
            }
        }

        if let Some(runner) = self.state.bases.second.clone() {
            let target_base = furthest_occupied.prev().unwrap();
            if target_base <= Base::Second { // no room to advance
                if second_forced { // force out
                    runner_advancements.push(RunnerAdvancement {
                        name: runner,
                        from_base: Base::Second,
                        to_base: None,
                    });
                }
                furthest_occupied = furthest_occupied.min(Base::Second);
            } else {
                let runner_skill: f64 = self.all_players.get(&runner).unwrap().metrics.running.into();
                let success_prob = runner_skill / (runner_skill + overall_difficulty);

                let to_base = random_advancement_between(Base::Second, target_base, success_prob, &mut self.rng);
                if second_forced {
                    runner_advancements.push(RunnerAdvancement {
                        name: runner,
                        from_base: Base::Second,
                        to_base,
                    });
                    furthest_occupied = to_base.unwrap_or(Base::Second);
                } else if success_prob >= 0.5 {
                    runner_advancements.push(RunnerAdvancement {
                        name: runner,
                        from_base: Base::Second,
                        to_base,
                    });
                    furthest_occupied = to_base.unwrap_or(Base::Second);
                } else {
                    furthest_occupied = Base::Second;
                }
            }
        }

        if let Some(runner) = self.state.bases.first.clone() {
            let target_base = furthest_occupied.prev().unwrap();
            if target_base <= Base::First { // no room to advance
                if first_forced { // force out
                    runner_advancements.push(RunnerAdvancement {
                        name: runner,
                        from_base: Base::First,
                        to_base: None,
                    });
                }
                furthest_occupied = furthest_occupied.min(Base::First);
            } else {
                let runner_skill: f64 = self.all_players.get(&runner).unwrap().metrics.running.into();
                let success_prob = runner_skill / (runner_skill + overall_difficulty);

                let to_base = random_advancement_between(Base::First, target_base, success_prob, &mut self.rng);
                if first_forced {
                    runner_advancements.push(RunnerAdvancement {
                        name: runner,
                        from_base: Base::First,
                        to_base,
                    });
                    furthest_occupied = to_base.unwrap_or(Base::First);
                } else if success_prob >= 0.5 {
                    runner_advancements.push(RunnerAdvancement {
                        name: runner,
                        from_base: Base::First,
                        to_base,
                    });
                    furthest_occupied = to_base.unwrap_or(Base::First);
                } else {
                    furthest_occupied = Base::First;
                }
            }
        }

        // advance batter
        let target_base = furthest_occupied.prev().unwrap();
        let at_bat_outcome = if target_base < Base::First { // no room to advance, automatically out
            runner_advancements.push(RunnerAdvancement {
                name: batter_name.clone(),
                from_base: Base::Batting,
                to_base: None,
            });

            AtBatOutcome::Out
        } else {
            let batter_skill: f64 = self.all_players.get(batter_name).unwrap().metrics.running.into();
            let success_prob = batter_skill / (batter_skill + overall_difficulty);

            let to_base = random_advancement_between(Base::Batting, target_base, success_prob, &mut self.rng);
            runner_advancements.push(RunnerAdvancement {
                name: batter_name.clone(),
                from_base: Base::Batting,
                to_base,
            });

            match to_base {
                None => AtBatOutcome::Out,
                Some(Base::First) => AtBatOutcome::Single,
                Some(Base::Second) => AtBatOutcome::Double,
                Some(Base::Third) => AtBatOutcome::Triple,
                Some(Base::Home) => AtBatOutcome::HomeRun,
                Some(Base::Batting) => unreachable!(),
            }
        };

        self.apply_runner_advancements(&mut runner_advancements);

        (at_bat_outcome, runner_advancements)
    }

    fn cycle_half_inning(&mut self) {
        self.state.half_inning = self.state.half_inning.next();
        self.state.bases = Bases::empty();
        self.state.outs = 0;
        self.state.count = Count::empty();
        self.state.last_inning_just_ended = true;
    }

    pub fn simulate_pitch(
        &mut self,
        pitch_aim_location: Option<StrikeZoneLocation>, // if Some, the pitcher will aim for the given location; if None, the pitcher will throw a random pitch
        batter_decision: Option<BatterDecision>, // if Some, the batter will follow the given swing decision; if None, the batter will decide to swing/take/bunt randomly
    ) -> EventsSummary {
        // 1. decide if the pitch is in or out of the strike zone
        // 2. decide if the batter swings/takes
        // 3. determine if the pitch is a strike/ball/hit
        // 4. if the pitch is a hit, determine the fielding and running outcomes
        // 5. apply the outcomes to the game state
        // 6. return a summary of what has happened

        self.state.last_inning_just_ended = false;

        let pitcher_name = if self.home_team_is_at_bat() {
            self.visiting_team.current_pitcher.clone()
        } else {
            self.home_team.current_pitcher.clone()
        };

        let pitch_aim_location = pitch_aim_location.unwrap_or(StrikeZoneLocation::iter().choose(&mut self.rng).unwrap());
        let pitcher_skill: f64 = self.all_players.get(&pitcher_name).unwrap().metrics.pitching.into();
        let pitcher_succeeds = self.rng.random_bool(pitcher_skill);
        let pitch_location = match (pitch_aim_location, pitcher_succeeds) {
            (StrikeZoneLocation::In, true) => StrikeZoneLocation::In,
            (StrikeZoneLocation::In, false) => StrikeZoneLocation::Out,
            (StrikeZoneLocation::Out, true) => StrikeZoneLocation::Out,
            (StrikeZoneLocation::Out, false) => StrikeZoneLocation::In,
        };

        let batter_name = if self.home_team_is_at_bat() {
            self.home_team.batting_order[self.state.home_team_batter_index].clone()
        } else {
            self.visiting_team.batting_order[self.state.visiting_team_batter_index].clone()
        };

        let batter_decision = batter_decision.unwrap_or(BatterDecision::iter().choose(&mut self.rng).unwrap());
        let batter_skill: f64 = self.all_players.get(&batter_name).unwrap().metrics.hitting.into();
        let mut events_summary = match (pitch_location, batter_decision) {
            (_, BatterDecision::Swing) => {
                let contact = self.rng.random_bool(batter_skill);
                if contact { // swing and contact
                    let field_location = FieldLocation::Close.random_from_skill(&mut self.rng, batter_skill);
                    let (at_bat_outcome, runner_advancements) = self.simulate_fielding_and_running(&batter_name, field_location);
                    self.state.count = Count::empty();

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
            self.state.count = Count::empty();

            if self.state.half_inning.top {
                self.state.visiting_team_batter_index = (self.state.visiting_team_batter_index + 1) % 9;
            } else {
                self.state.home_team_batter_index = (self.state.home_team_batter_index + 1) % 9;
            }
        } else if self.state.count.balls == 4 {
            events_summary.at_bat_outcome = Some(AtBatOutcome::Walk);
            self.state.count = Count::empty();
            let mut walk_advancements = self.walk_advancements(&batter_name);
            self.apply_runner_advancements(&mut walk_advancements); // this also cycles the batter
            events_summary.runner_advancements = walk_advancements;
        }

        // handle end of half-inning
        if self.state.outs >= 3 {
            self.cycle_half_inning();
        }

        self.state.game_outcome = self.game_outcome();
        events_summary.game_outcome = self.state.game_outcome.clone();

        events_summary
    }

    pub fn state_summary(&self) -> GameStateSummary {
        let batter_name = if self.state.half_inning.top {
            self.visiting_team.batting_order[self.state.visiting_team_batter_index].clone()
        } else {
            self.home_team.batting_order[self.state.home_team_batter_index].clone()
        };
        let pitcher_name = if self.state.half_inning.top {
            self.home_team.current_pitcher.clone()
        } else {
            self.visiting_team.current_pitcher.clone()
        };

        GameStateSummary {
            home_team_runs: self.state.home_team_runs,
            visiting_team_runs: self.state.visiting_team_runs,
            half_inning: self.state.half_inning.clone(),
            bases: self.state.bases.clone(),
            outs: self.state.outs,
            count: self.state.count.clone(),
            batter: batter_name,
            pitcher: pitcher_name,
        }
    }
}
