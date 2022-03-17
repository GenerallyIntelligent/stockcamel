use crate::board;
use crate::board::Board;
use crate::constants;
use crate::probabilities::accumulators::{
    AtomicPositionAccumulator, AtomicTileAccumulator, PositionAccumulator, TileAccumulator,
};
use crate::probabilities::odds::{CamelOdds, TileOdds};
use crossbeam::queue::ArrayQueue;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::sync::{Arc, Mutex};
use std::{panic, thread};

pub fn solve_probabilities(
    board: board::Board,
    depth: u8,
    num_workers: u8,
) -> (CamelOdds, CamelOdds, TileOdds) {
    let round_positions_accumulator = Arc::new(AtomicPositionAccumulator::new());
    let game_positions_accumulator = Arc::new(AtomicPositionAccumulator::new());
    let tile_accumulator = Arc::new(AtomicTileAccumulator::new());

    //TODO: Figure out the smallest we can make these stacks and still not have problems.
    let round_stack = Arc::new(ArrayQueue::new(29160));
    let _ = round_stack.push((board, depth));
    let game_stack = Arc::new(ArrayQueue::new(10));

    seed_stack(round_stack.clone(), num_workers);

    let mut worker_handles = Vec::new();
    for _ in 0..num_workers {
        let round_stack = round_stack.clone();
        let game_stack = game_stack.clone();
        let game_positions_accumulator = game_positions_accumulator.clone();
        let round_positions_accumulator = round_positions_accumulator.clone();
        let tile_accumulator = tile_accumulator.clone();
        let handle = thread::spawn(move || {
            start_round_worker(
                round_stack,
                game_stack.clone(),
                round_positions_accumulator,
                tile_accumulator,
            );
            start_game_worker(game_stack, game_positions_accumulator);
        });
        worker_handles.push(handle);
    }

    for handle in worker_handles {
        handle.join().unwrap();
    }

    // (0..num_workers).into_par_iter().for_each(|worker_num| {
    //     println!("Created worker {}", worker_num);
    //     start_round_worker(
    //         round_stack.clone(),
    //         game_stack.clone(),
    //         game_positions_accumulator.clone(),
    //         round_positions_accumulator.clone(),
    //         tile_accumulator.clone(),
    //     );
    // });
    let round_positions_accumulator: PositionAccumulator = round_positions_accumulator.into();
    let game_positions_accumulator: PositionAccumulator = game_positions_accumulator.into();
    let tile_accumulator: TileAccumulator = tile_accumulator.into();

    let rount_terminal_states = round_positions_accumulator.count_terminal();
    let rount_position_odds = CamelOdds::new(&round_positions_accumulator, &rount_terminal_states);
    let game_position_odds = (game_positions_accumulator + round_positions_accumulator).into();
    let tile_odds = TileOdds::new(&tile_accumulator, &rount_terminal_states);
    (game_position_odds, rount_position_odds, tile_odds)
}

fn seed_stack(stack: Arc<ArrayQueue<(Board, u8)>>, num_to_seed: u8) {
    let mut num_seeded = 1;
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

fn start_round_worker(
    round_stack: Arc<ArrayQueue<(Board, u8)>>,
    game_stack: Arc<ArrayQueue<(Board, u8)>>,
    round_positions_accumulator: Arc<AtomicPositionAccumulator>,
    tile_accumulator: Arc<AtomicTileAccumulator>,
) {
    loop {
        let (next_board, depth) = match round_stack.pop() {
            Some((board, depth)) => (board, depth),
            None => return,
        };
        update_round_and_game_state(
            next_board,
            depth,
            round_stack.clone(),
            game_stack.clone(),
            round_positions_accumulator.clone(),
            tile_accumulator.clone(),
        );
    }
}

fn start_game_worker(
    game_stack: Arc<ArrayQueue<(Board, u8)>>,
    game_positions_accumulator: Arc<AtomicPositionAccumulator>,
) {
    loop {
        let (next_board, depth) = match game_stack.pop() {
            Some((board, depth)) => (board, depth),
            None => return,
        };
        update_game_state(
            next_board,
            depth,
            game_stack.clone(),
            game_positions_accumulator.clone(),
        );
    }
}

fn update_round_and_game_state(
    board: Board,
    depth: u8,
    round_stack: Arc<ArrayQueue<(Board, u8)>>,
    game_stack: Arc<ArrayQueue<(Board, u8)>>,
    round_positions_accumulator: Arc<AtomicPositionAccumulator>,
    tile_accumulator: Arc<AtomicTileAccumulator>,
) {
    if depth == 0 {
        round_positions_accumulator.update(terminal_node_heuristic(board));
        return;
    } else if board.is_terminal() {
        round_positions_accumulator.update(board.camel_order());
        return;
    }

    if board.all_rolled() {
        let _ = game_stack.push((board, depth));
        return;
    }

    for roll in board.potential_moves() {
        let next_board = board.update(&roll);
        match round_stack.push((next_board, depth - 1)) {
            Ok(_) => {}
            Err(_) => panic!("Exceeded probability stack!"),
        };
    }
}

fn update_game_state(
    board: Board,
    depth: u8,
    game_stack: Arc<ArrayQueue<(Board, u8)>>,
    game_positions_accumulator: Arc<AtomicPositionAccumulator>,
) {
    if depth == 0 {
        game_positions_accumulator.update(terminal_node_heuristic(board));
        return;
    } else if board.is_terminal() {
        game_positions_accumulator.update(board.camel_order());
        return;
    }

    for roll in board.potential_moves() {
        let next_board = board.update(&roll);
        match game_stack.push((next_board, depth - 1)) {
            Ok(_) => {}
            Err(_) => panic!("Exceeded probability stack!"),
        };
    }
}

fn terminal_node_heuristic(board: board::Board) -> board::CamelOrder {
    return board.camel_order();
}

fn terminal_round_states_from_board(board: board::Board) -> u32 {
    let num_unrolled = board.num_unrolled() as u32;
    return num_unrolled.pow(constants::MAX_ROLL as u32);
}
