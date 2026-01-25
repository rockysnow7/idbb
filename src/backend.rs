use crate::baseball::{BaseballGame, BatterDecision, EventsSummary, GameOutcome, GameStateSummary, Player, PlayerMetrics, StrikeZoneLocation, Team};
use crate::text::{Granularity, TextEngine};
use prompted::input;
use rand::prelude::*;
use std::collections::HashMap;

const FIRST_NAMES: &str = include_str!("data/names/first_names.txt");
const LAST_NAMES: &str = include_str!("data/names/last_names.txt");

fn get_first_names() -> Vec<&'static str> {
    FIRST_NAMES.lines().collect()
}

fn get_last_names() -> Vec<&'static str> {
    LAST_NAMES.lines().collect()
}

fn generate_name(
    first_names: &[&'static str],
    last_names: &[&'static str],
    rng: &mut ThreadRng,
) -> String {
    let first_name = first_names.choose(rng).unwrap();
    let last_name = last_names.choose(rng).unwrap();
    let mut name = format!("{} {}", first_name, last_name);

    if rng.random_bool(0.05) {
        name = format!("{name} Jr.");
    } else if rng.random_bool(0.03) {
        name = format!("{name} III");
    } else if rng.random_bool(0.02) {
        name = format!("{name} Sr.");
    }

    name
}

fn generate_players(num_players: usize, rng: &mut ThreadRng) -> HashMap<String, Player> {
    let first_names = get_first_names();
    let last_names = get_last_names();

    let mut all_players = HashMap::new();
    for _ in 0..num_players {
        let name = generate_name(&first_names, &last_names, rng);
        let player = Player {
            name: name.clone(),
            metrics: PlayerMetrics::random(rng),
        };
        all_players.insert(name, player);
    }

    all_players
}

/// Returns a `Team` with the given name and players taken from the given list of names, as well as a vector of the names consumed from the list.
fn generate_team(team_name: &str, all_names: &mut Vec<String>) -> (Team, Vec<String>) {
    let batting_order = all_names.drain(..9).collect::<Vec<_>>();
    let all_pitchers = all_names.drain(..5).collect::<Vec<_>>();
    let current_pitcher = all_pitchers[0].clone();
    let fielders = all_names.drain(..8).collect::<Vec<_>>();
    let bullpen = all_names.clone();

    let team = Team {
        name: team_name.to_string(),
        batting_order: batting_order.clone().try_into().unwrap(),
        all_pitchers: all_pitchers.clone(),
        current_pitcher: current_pitcher,
        fielders: fielders.clone().try_into().unwrap(),
        bullpen: bullpen.clone(),
    };

    let mut consumed_names = Vec::new();
    consumed_names.extend(batting_order);
    consumed_names.extend(all_pitchers);
    consumed_names.extend(fielders);
    consumed_names.extend(bullpen);

    (team, consumed_names)
}

#[derive(Debug)]
pub enum UserInput {
    // for now
    StartNewGame,

    // pitch-level inputs
    PitchAim(StrikeZoneLocation),
    BatterDecision(BatterDecision),

    // inning-level inputs
    PlayAggressive,
    PlayWithheld,
}

#[derive(Debug)]
enum GamePhase {
    PreGame,
    InGame {
        baseball_game: BaseballGame,
        text_engine: TextEngine,
    },
    BetweenGames,
}

#[derive(Debug)]
pub enum GameOutput {
    PitchOutput {
        events_summary: EventsSummary,
        new_game_state_summary: GameStateSummary,
        description: String,
    },
    HalfInningOutput {
        events_summaries: Vec<EventsSummary>,
        game_state_summaries: Vec<GameStateSummary>,
        description: String,
    },
    StartNewGame,
}

#[derive(Debug)]
pub enum GameError {
    InvalidUserInput,
}

#[derive(Debug)]
pub struct Game {
    all_players: HashMap<String, Player>,
    own_team: Option<Team>, // will be None until the user sets it up, then it will be Some for the rest of the game
    other_teams: HashMap<String, Team>,
    rng: ThreadRng,
    phase: GamePhase,
}

impl Game {
    pub fn new() -> Self {
        let mut rng = rand::rng();

        let all_possible_players = generate_players(1000, &mut rng);
        let mut all_possible_names = all_possible_players.keys().cloned().collect::<Vec<_>>();

        let mut all_players = HashMap::new();
        let (own_team, consumed_names) = generate_team("Montreal Expos", &mut all_possible_names);
        let players: HashMap<String, Player> = consumed_names.iter()
            .map(|name| (name.clone(), all_possible_players.get(name).unwrap().clone()))
            .collect();
        all_players.extend(players);

        let mut other_teams = HashMap::new();
        let other_team_names = [
            "New York Yankees",
            "Chicago Cubs",
            "Los Angeles Dodgers",
        ];
        for team_name in other_team_names {
            let (team, consumed_names) = generate_team(team_name, &mut all_possible_names);
            other_teams.insert(team_name.to_string(), team);

            let players: HashMap<String, Player> = consumed_names.iter()
                .map(|name| (name.clone(), all_possible_players.get(name).unwrap().clone()))
                .collect();
            all_players.extend(players);
        }

        Self {
            all_players,
            own_team: Some(own_team),
            other_teams,
            rng,
            phase: GamePhase::PreGame,
        }
    }

    pub fn start_new_game(&mut self) {
        let visiting_team = self.other_teams.values().choose(&mut self.rng).unwrap().clone();
        let baseball_game = BaseballGame::new(
            self.all_players.clone(),
            self.own_team.as_ref().unwrap().clone(),
            visiting_team.clone(),
        );
        let text_engine = TextEngine::new(
            self.all_players.clone(),
            self.own_team.as_ref().unwrap().name.clone(),
            visiting_team.name,
        );
        self.phase = GamePhase::InGame { baseball_game, text_engine };
    }

    pub fn valid_user_inputs(&self) -> Vec<UserInput> {
        match &self.phase {
            GamePhase::PreGame => vec![UserInput::StartNewGame],
            GamePhase::InGame { baseball_game: current_game, .. } => {
                let game_state_summary = current_game.state_summary();
                let granularity = Granularity::from_state_summary(&game_state_summary);

                match granularity {
                    Granularity::Pitch => {
                        let home_team_is_at_bat = current_game.home_team_is_at_bat();
                        if home_team_is_at_bat {
                            vec![UserInput::BatterDecision(BatterDecision::Swing), UserInput::BatterDecision(BatterDecision::Take)]
                        } else {
                            vec![UserInput::PitchAim(StrikeZoneLocation::In), UserInput::PitchAim(StrikeZoneLocation::Out)]
                        }
                    }
                    Granularity::HalfInning => vec![UserInput::PlayAggressive, UserInput::PlayWithheld],
                }
            },
            GamePhase::BetweenGames => todo!(),
        }
    }

    pub fn process_user_input(&mut self, user_input: &UserInput) -> Result<GameOutput, GameError> {
        match &mut self.phase {
            GamePhase::PreGame => {
                let UserInput::StartNewGame = user_input else {
                    return Err(GameError::InvalidUserInput);
                };

                self.start_new_game();

                Ok(GameOutput::StartNewGame)
            },
            GamePhase::InGame { baseball_game: current_game, text_engine } => {
                let game_state_summary = current_game.state_summary();
                let granularity = Granularity::from_state_summary(&game_state_summary);
                let home_team_is_at_bat = current_game.home_team_is_at_bat();

                let game_output = match (granularity, user_input, home_team_is_at_bat) {
                    (Granularity::Pitch, UserInput::BatterDecision(decision), true) => {
                        let events_summary = current_game.simulate_pitch(None, Some(*decision));
                        let new_game_state_summary = current_game.state_summary();
                        let description = text_engine.describe_pitch_level_summaries(
                            &game_state_summary,
                            &events_summary,
                            &new_game_state_summary,
                        );

                        GameOutput::PitchOutput {
                            events_summary,
                            new_game_state_summary,
                            description,
                        }
                    },
                    (Granularity::Pitch, UserInput::PitchAim(location), false) => {
                        let events_summary = current_game.simulate_pitch(Some(*location), None);
                        let new_game_state_summary = current_game.state_summary();
                        let description = text_engine.describe_pitch_level_summaries(
                            &game_state_summary,
                            &events_summary,
                            &new_game_state_summary,
                        );

                        GameOutput::PitchOutput {
                            events_summary,
                            new_game_state_summary,
                            description,
                        }
                    },
                    (Granularity::HalfInning, UserInput::PlayAggressive, _) => {
                        let current_half = game_state_summary.half_inning.top;
                        let mut events_summaries = Vec::new();
                        let mut game_state_summaries = Vec::new();
                        loop {
                            let (pitch_location, batter_decision) = if current_game.home_team_is_at_bat() {
                                (None, Some(BatterDecision::Swing))
                            } else {
                                (Some(StrikeZoneLocation::In), None)
                            };
                            let events_summary = current_game.simulate_pitch(pitch_location, batter_decision);
                            events_summaries.push(events_summary);
                            let game_state_summary = current_game.state_summary();
                            game_state_summaries.push(game_state_summary);
                            if game_state_summaries.last().unwrap().half_inning.top != current_half {
                                break;
                            }
                        }
                        let description = text_engine.describe_half_inning_summaries(
                            &game_state_summary,
                            &events_summaries,
                            &game_state_summaries,
                        );

                        GameOutput::HalfInningOutput {
                            events_summaries,
                            game_state_summaries,
                            description,
                        }
                    },
                    (Granularity::HalfInning, UserInput::PlayWithheld, _) => todo!(),
                    _ => return Err(GameError::InvalidUserInput),
                };

                Ok(game_output)
            },
            GamePhase::BetweenGames => todo!(),
        }
    }
}
