mod backend;

use backend::Game;

fn main() {
    let mut game = Game::new();
    game.run();
}
