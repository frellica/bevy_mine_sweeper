mod game;
mod mine_core;
use std::env;
fn main() {
    println!("Hello, minesweeper!");
    let args: Vec<String> = env::args().collect();
    let config_map: Vec<(usize, usize, usize)> = vec![(8, 8, 10), (16, 16, 40), (30, 16, 99)];
    let (width, height, mine_count) = match args.len() {
        1 => config_map[0],
        3 => config_map[args[2].parse().unwrap_or(0)],
        _ => {
            panic!("usage: ./minesweeper --level NUM")
        }
    };
    println!("{:?}-{:?}-{:?}", width, height, mine_count);
    game::game_app(game::GameConfig { width, height, mine_count });
}
