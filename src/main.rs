mod constants;
mod concurrent;
mod board;
mod results;

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
    let (pos, tiles) = board.solve_round(8);
    println!("{}", pos);
    println!("{}", tiles);
    // println!("{}", board);
    let pos = board.solve_game(4, 8);
    println!("{}", pos);
}