mod baseball;

use baseball::{BaseballGame, StrikeZoneLocation};

/// The overall phase of the game.
enum SeasonPhase {
    RegularSeason(u8),
}

/// The phase of the current baseball game.
enum GamePhase {
    PreGame,
    InGame,
    PostGame,
}

pub enum Base {
    First,
    Second,
    Third,
    Home,
}

pub enum Ritual {
    Prayer,
    BloodSacrifice(String),
    ContinueToNextGame,
}

/// An action that the player can take.
pub enum Action {
    // pre-game actions
    SetBattingOrder(Vec<String>),
    SetStartingPitcher(String),
    StartGame,

    // in-game offensive actions
    ChooseSwing,
    ChooseBunt,
    ChooseTake,
    // AttemptSteal(Base), // should not be able to steal first base

    // in-game defensive actions
    ChoosePitchAimLocation(StrikeZoneLocation),

    // post-game actions
    PerformRitual(Ritual),
}

/// Data to be sent to the frontend.
pub struct GameView {
    display_text: String,
    available_actions: Vec<Action>,
}

pub enum GameError {}

pub struct Game {
    season_phase: SeasonPhase,
    game_phase: GamePhase,
    current_game: Option<BaseballGame>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            season_phase: SeasonPhase::RegularSeason(0),
            game_phase: GamePhase::PreGame,
            current_game: None,
        }
    }

    pub fn get_view(&self) -> GameView {
        GameView {
            display_text: self.get_display_text(),
            available_actions: self.get_available_actions(),
        }
    }

    fn get_display_text(&self) -> String {
        todo!()
    }

    fn get_available_actions(&self) -> Vec<Action> {
        todo!()
    }

    pub fn process_action(&mut self, action: Action) -> Result<GameView, GameError> {
        todo!()
    }
}
