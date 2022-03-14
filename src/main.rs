mod board;
mod constants;
mod probabilities;

use std::time::{SystemTime, UNIX_EPOCH};

fn create_board() -> board::Board {
    let mut positions = [[0; constants::NUM_CAMELS]; constants::BOARD_SIZE + 1];
    for index in 0..constants::NUM_CAMELS {
        positions[0][index] = index as u8 + 1;
    }
    let rolls = [false; 5];
    let oasis = [false; 16];
    let desert = [false; 16];
    let board = board::Board::new(positions, rolls, oasis, desert);
    return board;
}

fn time_probabilities(n: u32) {
    let board = create_board();

    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    for _ in 1..n {
        let _ = probabilities::solve_probabilities(board, 5, 24);
    }
    let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    println!(
        "{:.4} per solve",
        (end.as_millis() - start.as_millis()) / n as u128
    );
}

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(24)
        .build_global()
        .unwrap();

    let board = create_board();
    let (game_position_probabilies, round_position_probabilities, tile_probabilities) =
        probabilities::solve_probabilities(board, 8, 10);
    println!(
        "{}\n{}\n{}\n",
        game_position_probabilies, round_position_probabilities, tile_probabilities
    );

    time_probabilities(10);
}
