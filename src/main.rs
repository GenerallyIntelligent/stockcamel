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

// fn time_comparison(n: u32) {
//     let board = create_board();

//     let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
//     for _ in 1..n {
//         let _ = probabilities::solve_game_from(board, 5, 8);
//     }
//     let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
//     println!(
//         "Test 1: {:.4} per test",
//         (end.as_millis() - start.as_millis()) / n as u128
//     );

//     let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
//     for _ in 1..n {
//         let _ = probabilities::solve_round_from(board, 8);
//     }
//     let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
//     println!(
//         "Test 2: {:.4} per test",
//         (end.as_millis() - start.as_millis()) / n as u128
//     );

//     println!("Done");
// }

// fn run_game(n: u32) {
//     let board = create_board();

//     for _ in 1..n {
//         let _ = probabilities::solve_game_from(board, 5, 8);
//     }

//     println!("Done")
// }

// fn run_round(n: u32) {
//     let board = create_board();

//     for _ in 1..n {
//         let _ = probabilities::solve_round_from(board, 8);
//     }

//     println!("Done")
// }

fn main() {
    let board = create_board();
    let (game_position_probabilies, round_position_probabilities, tile_probabilities) =
        probabilities::calculate::solve_probabilities(board, 5);
    println!(
        "{}\n{}\n{}\n",
        game_position_probabilies, round_position_probabilities, tile_probabilities
    );
}
