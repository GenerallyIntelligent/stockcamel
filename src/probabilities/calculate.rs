use crate::board;
use crate::board::Board;
use crate::constants;
use crate::probabilities::accumulators::{
    AtomicPositionAccumulator, AtomicTileAccumulator, PositionAccumulator, TileAccumulator,
};
use crate::probabilities::odds::{CamelOdds, TileOdds};
use crate::probabilities::transposition::{
    GameTranspositionTable, ProbabilitiesTranspositionTable, RoundTranspositionTable,
};
use crossbeam::queue::ArrayQueue;
use parking_lot::RwLock;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::sync::{Arc, Mutex};
use std::{panic, thread};

pub fn solve_probabilities(
    board: board::Board,
    depth: u8,
    num_workers: usize,
) -> (CamelOdds, CamelOdds, TileOdds) {
    coz::scope!("Solve Probabilities");
    let round_positions_accumulator = AtomicPositionAccumulator::new();
    let game_positions_accumulator = AtomicPositionAccumulator::new();
    let tile_accumulator = AtomicTileAccumulator::new();

    let round_transposition_table = RoundTranspositionTable::new(1e5 as u32);
    let game_transposition_table = GameTranspositionTable::new(1e5 as u32);

    let stack = ArrayQueue::new(num_workers * 2);
    let _ = stack.push((board, depth));
    seed_stack(&stack, num_workers);

    let transition_depth = depth - board.num_unrolled();

    (0..num_workers).into_par_iter().for_each(|_| {
        coz::thread_init();
        start_worker(
            &stack,
            &round_transposition_table,
            &game_transposition_table,
            transition_depth,
            &game_positions_accumulator,
            &round_positions_accumulator,
            &tile_accumulator,
        );
    });

    let round_positions_accumulator: PositionAccumulator = round_positions_accumulator.into();
    let game_positions_accumulator: PositionAccumulator = game_positions_accumulator.into();
    let tile_accumulator: TileAccumulator = tile_accumulator.into();

    let round_terminal_states = round_positions_accumulator.count_terminal();
    let round_position_odds = CamelOdds::new(&round_positions_accumulator, &round_terminal_states);
    let game_position_odds = game_positions_accumulator.into();
    let tile_odds = TileOdds::new(&tile_accumulator, &round_terminal_states);
    (game_position_odds, round_position_odds, tile_odds)
}

fn start_worker(
    stack: &ArrayQueue<(Board, u8)>,
    round_transposition_table: &RoundTranspositionTable,
    game_transposition_table: &GameTranspositionTable,
    transition_depth: u8,
    game_positions_accumulator: &AtomicPositionAccumulator,
    round_positions_accumulator: &AtomicPositionAccumulator,
    tile_accumulator: &AtomicTileAccumulator,
) {
    let mut private_game_positions = PositionAccumulator::new();
    let mut private_round_positions = PositionAccumulator::new();
    let mut private_tile_positions = TileAccumulator::new();
    loop {
        let (board, depth) = match stack.pop() {
            Some((board, depth)) => (board, depth),
            None => break,
        };
        if depth > transition_depth {
            let (game_accumulations, round_accumulations, tile_accumulations) =
                calculate_round_and_game_terminal_states(
                    &board,
                    &depth,
                    &transition_depth,
                    round_transposition_table,
                    game_transposition_table,
                );
            private_game_positions += game_accumulations;
            private_round_positions += round_accumulations;
            private_tile_positions += tile_accumulations;
        } else {
            let game_accumulations =
                calculate_game_terminal_states(&board, &depth, game_transposition_table);
            private_game_positions += game_accumulations;
        }
    }
    game_positions_accumulator.add(private_game_positions);
    round_positions_accumulator.add(private_round_positions);
    tile_accumulator.add(private_tile_positions);
}

fn calculate_round_and_game_terminal_states(
    board: &Board,
    depth: &u8,
    transition_depth: &u8,
    round_transposition_table: &RoundTranspositionTable,
    game_transposition_table: &GameTranspositionTable,
) -> (PositionAccumulator, PositionAccumulator, TileAccumulator) {
    match round_transposition_table.check(board) {
        Some(accumulators) => return accumulators,
        None => {}
    }

    if depth == &0 {
        let accum = terminal_node_heuristic(board).into();
        return (accum, accum, TileAccumulator::new());
    } else if board.is_terminal() {
        let accum = board.camel_order().into();
        return (accum, accum, TileAccumulator::new());
    }

    let mut game_positions_accumulator = PositionAccumulator::new();
    let mut round_positions_accumulator = PositionAccumulator::new();
    let mut tile_accumulator = TileAccumulator::new();

    if depth <= transition_depth {
        round_positions_accumulator += board.camel_order().into();
        let game_positions = calculate_game_terminal_states(board, depth, game_transposition_table);
        game_positions_accumulator += game_positions;
        return (
            game_positions_accumulator,
            round_positions_accumulator,
            tile_accumulator,
        );
    }

    for roll in board.potential_moves() {
        let (next_board, target) = board.update_with_target(&roll);
        tile_accumulator[target] += terminal_round_states_from_board(&next_board);
        let (game_positions, round_positions, tiles) = calculate_round_and_game_terminal_states(
            &next_board,
            &(depth - 1),
            transition_depth,
            round_transposition_table.clone(),
            game_transposition_table.clone(),
        );
        game_positions_accumulator += game_positions;
        round_positions_accumulator += round_positions;
        tile_accumulator += tiles;
    }

    round_transposition_table.update(
        board,
        *depth,
        (
            game_positions_accumulator,
            round_positions_accumulator,
            tile_accumulator,
        ),
    );

    return (
        game_positions_accumulator,
        round_positions_accumulator,
        tile_accumulator,
    );
}

fn calculate_game_terminal_states(
    board: &Board,
    depth: &u8,
    transposition_table: &GameTranspositionTable,
) -> PositionAccumulator {
    match transposition_table.check(board) {
        Some(game_positions) => return game_positions,
        None => {}
    }

    if depth == &0 {
        return terminal_node_heuristic(board).into();
    } else if board.is_terminal() {
        return board.camel_order().into();
    }

    let mut positions_accumulator = PositionAccumulator::new();

    for roll in board.potential_moves() {
        let next_board = board.update(&roll);
        let positions =
            calculate_game_terminal_states(&next_board, &(depth - 1), transposition_table.clone());
        positions_accumulator += positions;
    }

    transposition_table.update(board, *depth, positions_accumulator);

    return positions_accumulator;
}

fn seed_stack(stack: &ArrayQueue<(Board, u8)>, num_to_seed: usize) {
    let mut num_seeded = stack.len();
    while num_seeded < num_to_seed {
        let (board, depth) = match stack.pop() {
            Some((board, depth)) => (board, depth),
            None => panic!(
                "Failed to seed the stack with at least {} board states!",
                num_seeded
            ),
        };
        num_seeded -= 1;
        for roll in board.potential_moves() {
            let next_board = board.update(&roll);
            match stack.push((next_board, depth - 1)) {
                Ok(_) => {}
                Err(_) => panic!("Exceeded probability stack!"),
            };
            num_seeded += 1;
        }
    }
}

fn terminal_node_heuristic(board: &board::Board) -> board::CamelOrder {
    return board.camel_order();
}

// Calculating directly is expensive, especially if the values are known for each possible
fn terminal_round_states_from_board(board: &board::Board) -> u32 {
    match board.num_unrolled() {
        0 => 1,
        1 => 3,
        2 => 18,
        3 => 162,
        4 => 1944,
        5 => 1,
        _ => {
            panic!("Invalid number of unrolled camels!")
        }
    }
}
