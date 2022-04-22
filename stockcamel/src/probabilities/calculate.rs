use crate::primitives::board;
use crate::primitives::board::Board;
use crate::probabilities::accumulators::{
    AtomicPositionAccumulator, AtomicTileAccumulator, PositionAccumulator, TileAccumulator,
};
use crate::probabilities::odds::{CamelOdds, TileOdds};
use crate::probabilities::sync::WorkerSync;
use crate::probabilities::transposition::{GameTranspositionTable, RoundTranspositionTable};
use crossbeam::queue::ArrayQueue;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::panic;
use std::sync::atomic;

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

    let stack = ArrayQueue::new((num_workers * 2).max(15 + 12 + 9 + 6 + 3 + 1));
    let _ = stack.push((board, depth));

    let transition_depth = depth - board.num_unrolled();

    let worker_sync = WorkerSync::new(num_workers);
    (0..num_workers).into_par_iter().for_each(|worker_id| {
        coz::thread_init();
        start_worker(
            worker_id,
            &worker_sync,
            &stack,
            1,
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
    worker_id: usize,
    worker_syncronization: &WorkerSync,
    stack: &ArrayQueue<(Board, u8)>,
    stack_minimum: usize,
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
            None => {
                worker_syncronization[worker_id].store(true, atomic::Ordering::Relaxed);
                let waiting_result = loop {
                    if let Some((board, depth)) = stack.pop() {
                        worker_syncronization[worker_id].store(false, atomic::Ordering::Relaxed);
                        break Some((board, depth));
                    }
                    if worker_syncronization.all() {
                        println!("Worker {} terminating!", worker_id);
                        break None;
                    }
                };
                match waiting_result {
                    Some((board, depth)) => (board, depth),
                    None => break,
                }
            }
        };
        let current_stack_size = stack.len();
        if current_stack_size < stack_minimum && depth > transition_depth {
            // Solve for round step and add to public stack
            let (game_accumulations, round_accumulations, tile_accumulations, next_boards) =
                calculate_round_step(&board, &depth);
            private_game_positions += game_accumulations;
            private_round_positions += round_accumulations;
            private_tile_positions += tile_accumulations;
            match next_boards {
                Some(next_boards) => {
                    for next_board in next_boards {
                        if let Err(_) = stack.push((next_board, depth - 1)) {
                            panic!("Solving probabilities overflowed shared queue!")
                        }
                    }
                }
                None => {}
            }
        } else if current_stack_size < stack_minimum && depth <= transition_depth {
            // Solve for game step and add to public stack
            let (game_accumulations, next_boards) = calculate_game_step(&board, &depth);
            private_game_positions += game_accumulations;
            private_round_positions += board.camel_order().into();
            match next_boards {
                Some(next_boards) => {
                    for next_board in next_boards {
                        if let Err(_) = stack.push((next_board, depth - 1)) {
                            panic!("Solving probabilities overflowed shared queue!")
                        }
                    }
                }
                None => {}
            }
        } else if current_stack_size >= stack_minimum && depth > transition_depth {
            // Solve for round and game recursively
            let (game_accumulations, round_accumulations, tile_accumulations) =
                calculate_round_and_game_recursive(
                    &board,
                    &depth,
                    &transition_depth,
                    round_transposition_table,
                    game_transposition_table,
                );
            private_game_positions += game_accumulations;
            private_round_positions += round_accumulations;
            private_tile_positions += tile_accumulations;
        } else if current_stack_size >= stack_minimum && depth <= transition_depth {
            // Solve for game recursively
            let game_accumulations =
                calculate_game_recursive(&board, &depth, game_transposition_table);
            private_game_positions += game_accumulations;
        }
    }
    game_positions_accumulator.add(private_game_positions);
    round_positions_accumulator.add(private_round_positions);
    tile_accumulator.add(private_tile_positions);
}

fn calculate_round_and_game_recursive(
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

    if depth <= transition_depth {
        let game_positions = calculate_game_recursive(board, depth, game_transposition_table);
        return (
            game_positions,
            board.camel_order().into(),
            TileAccumulator::new(),
        );
    }

    let mut game_positions_accumulator = PositionAccumulator::new();
    let mut round_positions_accumulator = PositionAccumulator::new();
    let mut tile_accumulator = TileAccumulator::new();

    let (game_positions, round_positions, tiles, boards) = calculate_round_step(board, depth);
    game_positions_accumulator += game_positions;
    round_positions_accumulator += round_positions;
    tile_accumulator += tiles;

    match boards {
        Some(boards) => {
            for next_board in boards {
                let (game_positions, round_positions, tiles) = calculate_round_and_game_recursive(
                    &next_board,
                    &(depth - 1),
                    transition_depth,
                    round_transposition_table,
                    game_transposition_table,
                );
                game_positions_accumulator += game_positions;
                round_positions_accumulator += round_positions;
                tile_accumulator += tiles;
            }
        }
        None => {}
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

fn calculate_round_step(
    board: &Board,
    depth: &u8,
) -> (
    PositionAccumulator,
    PositionAccumulator,
    TileAccumulator,
    Option<Vec<Board>>,
) {
    if depth == &0 {
        let accum = terminal_node_heuristic(board).into();
        return (accum, accum, TileAccumulator::new(), None);
    } else if board.is_terminal() {
        let accum = board.camel_order().into();
        return (accum, accum, TileAccumulator::new(), None);
    }

    let mut tile_accumulator = TileAccumulator::new();

    let mut boards = Vec::new();
    for roll in board.potential_moves() {
        let (next_board, target) = board.update_with_target(&roll);
        tile_accumulator[target] += terminal_round_states_from_board(&next_board);
        boards.push(next_board);
    }

    return (
        PositionAccumulator::new(),
        PositionAccumulator::new(),
        tile_accumulator,
        Some(boards),
    );
}

fn calculate_game_recursive(
    board: &Board,
    depth: &u8,
    transposition_table: &GameTranspositionTable,
) -> PositionAccumulator {
    match transposition_table.check(board) {
        Some(game_positions) => return game_positions,
        None => {}
    }

    let mut positions_accumulator = PositionAccumulator::new();
    let (positions, boards) = calculate_game_step(board, depth);
    positions_accumulator += positions;

    match boards {
        Some(boards) => {
            for board in boards {
                let positions = calculate_game_recursive(&board, &(depth - 1), transposition_table);
                positions_accumulator += positions;
            }
        }
        None => {}
    }

    transposition_table.update(board, *depth, positions_accumulator);
    return positions_accumulator;
}

fn calculate_game_step(board: &Board, depth: &u8) -> (PositionAccumulator, Option<Vec<Board>>) {
    if depth == &0 {
        return (terminal_node_heuristic(board).into(), None);
    } else if board.is_terminal() {
        return (board.camel_order().into(), None);
    }

    let mut boards = Vec::new();
    for roll in board.potential_moves() {
        let next_board = board.update(&roll);
        boards.push(next_board)
    }

    return (PositionAccumulator::new(), Some(boards));
}

fn terminal_node_heuristic(board: &board::Board) -> board::CamelOrder {
    return board.camel_order();
}

// Calculating directly is expensive, especially if the values are already known for each possible input
fn terminal_round_states_from_board(board: &board::Board) -> u32 {
    match board.num_unrolled() {
        5 => 1, // Equivalent to zero, since the camel rolls are flipped automatically at round end
        1 => 3,
        2 => 18,
        3 => 162,
        4 => 1944,
        _ => {
            panic!("Invalid number of unrolled camels!")
        }
    }
}
