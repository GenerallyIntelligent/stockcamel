mod constants;
mod concurrent;
mod board;
mod probabilities;

fn main() {
    let mut positions = [[0; constants::NUM_CAMELS]; constants::BOARD_SIZE + 1];
    for index in 0..constants::NUM_CAMELS {
        positions[0][index] = index as u8 + 1;
    }
    let rolls = [false; 5];
    let oasis = [false; 16];
    let mut desert = [false; 16];
    desert[3] = true;
    let board = board::Board::new(positions, rolls, oasis, desert);
    let (pos, tiles) = probabilities::solve_round_from(board, 8);
    println!("{}", pos);
    let pos = probabilities::solve_game_from(board, 5, 8);
    println!("{}", pos);
}