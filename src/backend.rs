mod baseball;

use baseball::{BaseballGame, GameOutcome, Player, PlayerMetrics, Team};
use prompted::input;
use rand::prelude::*;
use std::collections::HashMap;

const FIRST_NAMES: &str = include_str!("data/first_names.txt");
const LAST_NAMES: &str = include_str!("data/last_names.txt");

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

#[derive(Debug)]
pub struct Game {
    current_game: Option<BaseballGame>,
    rng: ThreadRng,
}

impl Game {
    pub fn new() -> Self {
        let mut rng = rand::rng();

        let all_players = generate_players(100, &mut rng);
        let mut all_names = all_players.keys().cloned().collect::<Vec<_>>();

        let home_batting_order = all_names.drain(..9).collect::<Vec<_>>();
        let home_starting_pitcher = all_names.drain(..1).collect::<Vec<_>>()[0].clone();
        let home_fielders = all_names.drain(..8).collect::<Vec<_>>();
        let home_bullpen = all_names.drain(..9).collect::<Vec<_>>();

        let home_team = Team {
            name: "Montreal Expos".to_string(),
            batting_order: home_batting_order.try_into().unwrap(),
            current_pitcher: home_starting_pitcher,
            fielders: home_fielders.try_into().unwrap(),
            bullpen: home_bullpen,
        };

        let visiting_batting_order = all_names.drain(..9).collect::<Vec<_>>();
        let visiting_starting_pitcher = all_names.drain(..1).collect::<Vec<_>>()[0].clone();
        let visiting_fielders = all_names.drain(..8).collect::<Vec<_>>();
        let visiting_bullpen = all_names.drain(..9).collect::<Vec<_>>();

        let visiting_team = Team {
            name: "New York Yankees".to_string(),
            batting_order: visiting_batting_order.try_into().unwrap(),
            current_pitcher: visiting_starting_pitcher,
            fielders: visiting_fielders.try_into().unwrap(),
            bullpen: visiting_bullpen,
        };

        let current_game = Some(BaseballGame::new(all_players, home_team, visiting_team));

        Self { current_game, rng }
    }

    pub fn next(&mut self) {
        let current_game = self.current_game.as_mut().unwrap();

        let events_summary = current_game.simulate_pitch(None, None);
        println!("{:#?}", events_summary);

        let state_summary = current_game.state_summary();
        println!("{:#?}\n", state_summary);
    }

    pub fn run(&mut self) {
        while self.current_game.as_ref().unwrap().state.game_outcome == GameOutcome::Ongoing {
            self.next();
            // input!("Press Enter to continue...");
        }
    }
}
