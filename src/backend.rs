mod baseball;

use baseball::{BaseballGame, Player, PlayerMetrics, Team};
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

fn generate_all_players(rng: &mut ThreadRng) -> HashMap<String, Player> {
    let first_names = get_first_names();
    let last_names = get_last_names();

    let mut all_players = HashMap::new();
    for _ in 0..100 {
        let name = generate_name(&first_names, &last_names, rng);
        let player = Player {
            name: name.clone(),
            metrics: PlayerMetrics::random(rng),
        };
        all_players.insert(name, player);
    }

    all_players
}

pub struct Game {
    current_game: Option<BaseballGame>,
    rng: ThreadRng,
}

impl Game {
    pub fn new() -> Self {
        let mut rng = rand::rng();

        let all_players = generate_all_players(&mut rng);
        println!("{:#?}", all_players);

        // let home_team = Team {
        //     name: "Montreal Expos".to_string(),
        //     batting_order: 
        // };

        let current_game = None;

        Self {
            current_game,
            rng,
        }
    }

    pub fn next(&mut self) {
        todo!()
    }
}
