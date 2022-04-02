mod board;
mod camel;
mod constants;
mod probabilities;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::camel::create_camel;

fn create_board() -> board::Board {
    let camels = [0, 2, 4, 6, 8];
    let oasis = [false; 16];
    let desert = [false; 16];
    let board = board::Board::new(camels, oasis, desert);
    return board;
}

fn time_probabilities(n: u32) {
    let board = create_board();

    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    for _ in 1..n {
        let _ = probabilities::solve_probabilities(board, 5, 124);
    }
    let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    println!(
        "{:.4} per solve",
        (end.as_micros() - start.as_micros()) / n as u128
    );
}

fn main() {
    coz::thread_init();
    let camels = [33, 31, 40, 50, 71];
    let oasis = [false; 16];
    let desert = [false; 16];
    let board = board::Board::new(camels, oasis, desert);
    let (game_position_probabilies, round_position_probabilities, tile_probabilities) =
        probabilities::solve_probabilities(board, 6, 14);
    println!(
        "{}\n{}\n{}\n",
        game_position_probabilies, round_position_probabilities, tile_probabilities
    );

    time_probabilities(1000);
}
