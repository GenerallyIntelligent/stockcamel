use crate::board;
use crate::board::Board;
use crate::constants;
use crate::probabilities::accumulators::{AtomicPositionAccumulator, AtomicTileAccumulator};
use crate::probabilities::odds::{CamelOdds, TileOdds};
use parking_lot::Mutex;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::sync::{atomic, Arc};

pub fn solve_probabilities(board: board::Board, depth: u8) -> (CamelOdds, CamelOdds, TileOdds) {
    let round_positions_accumulator = Arc::new(Mutex::new(AtomicPositionAccumulator::new()));
    let game_positions_accumulator = Arc::new(Mutex::new(AtomicPositionAccumulator::new()));
    let tile_accumulator = Arc::new(Mutex::new(AtomicTileAccumulator::new()));

    solve_round_and_game_counts(
        board,
        depth,
        game_positions_accumulator.clone(),
        round_positions_accumulator.clone(),
        tile_accumulator.clone(),
    );

    {
        let round_positions_accumulator = round_positions_accumulator.lock().clone();
        let game_positions_accumulator = game_positions_accumulator.lock().clone();
        let tile_accumulator = tile_accumulator.lock().clone();

        let rount_terminal_states = round_positions_accumulator.count_terminal();
        let rount_position_odds =
            CamelOdds::new(&round_positions_accumulator.into(), &rount_terminal_states);
        let game_position_odds = game_positions_accumulator.into();
        let tile_odds = TileOdds::new(&tile_accumulator.into(), &rount_terminal_states);
        (game_position_odds, rount_position_odds, tile_odds)
    }
}

fn solve_round_and_game_counts(
    board: board::Board,
    depth: u8,
    game_positions_accumulator: Arc<Mutex<AtomicPositionAccumulator>>,
    round_positions_accumulator: Arc<Mutex<AtomicPositionAccumulator>>,
    tile_accumulator: Arc<Mutex<AtomicTileAccumulator>>,
) {
    if depth == 0 {
        let mut round_lock = round_positions_accumulator.lock();
        *round_lock += terminal_node_heuristic(board);
        let mut game_lock = game_positions_accumulator.lock();
        *game_lock += terminal_node_heuristic(board);
        return;
    } else if board.is_terminal() {
        let mut round_lock = round_positions_accumulator.lock();
        *round_lock += board.camel_order();
        let mut game_lock = game_positions_accumulator.lock();
        *game_lock += board.camel_order();
        return;
    }

    if board.all_rolled() {
        solve_game_counts(board, depth, game_positions_accumulator);
        return;
    }

    board.potential_moves().par_iter().for_each(|roll| {
        roll_then_solve_round_and_game_counts(
            roll,
            board,
            depth,
            game_positions_accumulator.clone(),
            round_positions_accumulator.clone(),
            tile_accumulator.clone(),
        )
    });
}

fn roll_then_solve_round_and_game_counts(
    roll: &board::Roll,
    board: board::Board,
    depth: u8,
    game_positions_accumulator: Arc<Mutex<AtomicPositionAccumulator>>,
    round_positions_accumulator: Arc<Mutex<AtomicPositionAccumulator>>,
    tile_accumulator: Arc<Mutex<AtomicTileAccumulator>>,
) {
    let (board, target) = board.update_with_target(roll);
    let n_states_from_current = terminal_round_states_from_board(board);
    {
        let tile_lock = tile_accumulator.lock();
        tile_lock[target].fetch_add(n_states_from_current, atomic::Ordering::Relaxed);
    }
    solve_round_and_game_counts(
        board,
        depth - 1,
        game_positions_accumulator,
        round_positions_accumulator,
        tile_accumulator,
    );
}

fn solve_game_counts(
    board: board::Board,
    depth: u8,
    position_accumulator: Arc<Mutex<AtomicPositionAccumulator>>,
) {
    if depth == 0 {
        let mut round_lock = position_accumulator.lock();
        *round_lock += terminal_node_heuristic(board);
        return;
    } else if board.is_terminal() {
        let mut round_lock = position_accumulator.lock();
        *round_lock += board.camel_order();
        return;
    }

    board.potential_moves().par_iter().for_each(|roll| {
        roll_then_solve_game_counts(roll, board, depth, position_accumulator.clone())
    });
}

fn roll_then_solve_game_counts(
    roll: &board::Roll,
    board: Board,
    depth: u8,
    position_accumulator: Arc<Mutex<AtomicPositionAccumulator>>,
) {
    let board = board.update(roll);
    solve_game_counts(board, depth - 1, position_accumulator);
}

fn terminal_node_heuristic(board: board::Board) -> board::CamelOrder {
    return board.camel_order();
}

fn terminal_round_states_from_board(board: board::Board) -> u32 {
    let num_unrolled = board.num_unrolled() as u32;
    return num_unrolled.pow(constants::MAX_ROLL as u32);
}
