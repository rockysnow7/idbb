use idbb::{Game, UserInput, baseball::StrikeZoneLocation};

fn main() {
    let mut game = Game::new();
    let _ = game.process_user_input(&UserInput::StartNewGame);

    let valid_user_inputs = game.valid_user_inputs();
    println!("Valid user inputs: {valid_user_inputs:#?}");

    let game_output = game.process_user_input(&UserInput::PlayAggressive);
    println!("Game output: {game_output:#?}");

    let valid_user_inputs = game.valid_user_inputs();
    println!("Valid user inputs: {valid_user_inputs:#?}");
}
