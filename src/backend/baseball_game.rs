pub struct PlayerName(String);

pub struct Team {
    name: String,
    batting_order: Vec<PlayerName>,
    starting_pitcher: PlayerName,
}

struct HalfInning {
    number: u8,
    top: bool,
}

struct Bases {
    first: Option<PlayerName>,
    second: Option<PlayerName>,
    third: Option<PlayerName>,
}

struct GameState {
    home_team_runs: u8,
    visiting_team_runs: u8,
    half_inning: HalfInning,
    outs: u8,
    bases: Bases,
}

pub struct BaseballGame {
    home_team: Team,
    visiting_team: Team,
    state: GameState,
}

impl BaseballGame {
    pub fn new(player_team: Team, opposing_team: Team) -> Self {
        Self {
            home_team: player_team,
            visiting_team: opposing_team,
            state: GameState {
                home_team_runs: 0,
                visiting_team_runs: 0,
                half_inning: HalfInning { number: 1, top: true },
                outs: 0,
                bases: Bases { first: None, second: None, third: None },
            },
        }
    }
}
