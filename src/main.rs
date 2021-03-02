mod game;
mod mine_core;
use std::env;
fn main() {
    println!("Hello, minesweeper!");
    game::game_app(game::GameConfig { width: 30, height: 16, mine_count: 99 });
}
