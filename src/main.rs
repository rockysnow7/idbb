mod backend;

use backend::Game;

fn main() {
    let mut game = Game::new();

    for _ in 0..4 {
        game.next();
    }
}
