use idbb::{Game, UserInput, GameOutput, baseball::StrikeZoneLocation};
use prompted::input;

fn print_options(options: &Vec<UserInput>) {
    println!("OPTIONS:");
    for (i, option) in options.iter().enumerate() {
        println!("\t{i}: {option:?}");
    }
}

fn main() {
    let mut game = Game::new();
    let _ = game.process_user_input(&UserInput::StartNewGame);

    loop {
        let valid_user_inputs = game.valid_user_inputs();
        print_options(&valid_user_inputs);
        let user_input = input!("Enter your choice: ").parse::<usize>().unwrap();
        let choice = &valid_user_inputs[user_input];

        let Ok(GameOutput::PitchOutput { description, .. }) = game.process_user_input(choice) else { unreachable!() };
        // println!("Game output: {game_output:#?}");
        println!("\n{description}\n");
    }
}
