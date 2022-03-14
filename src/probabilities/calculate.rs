use crate::board;
use crate::board::Board;
use crate::constants;
use crate::probabilities::accumulators::{
    AtomicPositionAccumulator, AtomicTileAccumulator, PositionAccumulator, TileAccumulator,
};
use crate::probabilities::odds::{CamelOdds, TileOdds};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::sync::{Arc, Mutex};
use std::{panic, thread};

pub fn solve_probabilities(
    board: board::Board,
    depth: u8,
    num_workers: u8,
) -> (CamelOdds, CamelOdds, TileOdds) {
    let round_positions_accumulator = Arc::new(Mutex::new(AtomicPositionAccumulator::new()));
    let game_positions_accumulator = Arc::new(Mutex::new(AtomicPositionAccumulator::new()));
    let tile_accumulator = Arc::new(Mutex::new(AtomicTileAccumulator::new()));

    let round_stack = Arc::new(Mutex::new(Vec::new()));
    {
        let mut round_stack = round_stack.lock().unwrap();
        round_stack.push((board, depth));
    }
    let game_stack = Arc::new(Mutex::new(Vec::new()));

    seed_stack(round_stack.clone(), num_workers);

    let mut worker_handles = Vec::new();
    for worker_num in 0..num_workers {
        let round_stack = round_stack.clone();
        let game_stack = game_stack.clone();
        let game_positions_accumulator = game_positions_accumulator.clone();
        let round_positions_accumulator = round_positions_accumulator.clone();
        let tile_accumulator = tile_accumulator.clone();
        let handle = thread::spawn(move || {
            start_round_worker(
                worker_num,
                round_stack,
                game_stack,
                game_positions_accumulator,
                round_positions_accumulator,
                tile_accumulator,
            );
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

    {
        let round_positions_accumulator: PositionAccumulator =
            round_positions_accumulator.lock().unwrap().clone().into();
        let game_positions_accumulator: PositionAccumulator =
            game_positions_accumulator.lock().unwrap().clone().into();
        let tile_accumulator: TileAccumulator = tile_accumulator.lock().unwrap().clone().into();

        let rount_terminal_states = round_positions_accumulator.count_terminal();
        let rount_position_odds =
            CamelOdds::new(&round_positions_accumulator, &rount_terminal_states);
        let game_position_odds = (game_positions_accumulator + round_positions_accumulator).into();
        let tile_odds = TileOdds::new(&tile_accumulator, &rount_terminal_states);
        (game_position_odds, rount_position_odds, tile_odds)
    }
}

fn seed_stack(stack: Arc<Mutex<Vec<(Board, u8)>>>, num_to_seed: u8) {
    let mut num_seeded = 1;
    while num_seeded < num_to_seed {
        let (board, depth) = match stack.clone().lock().unwrap().pop() {
            Some((board, depth)) => (board, depth),
            None => panic!(
                "Failed to seed the stack with at least {} board states!",
                num_seeded
            ),
        };
        num_seeded -= 1;
        let mut new_boards = Vec::new();
        for roll in board.potential_moves() {
            let next_board = board.update(&roll);
            new_boards.push((next_board, depth - 1));
            num_seeded += 1;
        }

        {
            let mut stack = stack.lock().unwrap();
            stack.append(&mut new_boards);
        }
    }
}

fn start_round_worker(
    worker_num: u8,
    round_stack: Arc<Mutex<Vec<(Board, u8)>>>,
    game_stack: Arc<Mutex<Vec<(Board, u8)>>>,
    game_positions_accumulator: Arc<Mutex<AtomicPositionAccumulator>>,
    round_positions_accumulator: Arc<Mutex<AtomicPositionAccumulator>>,
    tile_accumulator: Arc<Mutex<AtomicTileAccumulator>>,
) {
    loop {
        let (next_board, depth) = match round_stack.clone().lock().unwrap().pop() {
            Some((board, depth)) => (board, depth),
            None => {
                start_game_worker(game_stack, game_positions_accumulator);
                return;
            }
        };
        update_round_and_game_state(
            next_board,
            depth,
            round_stack.clone(),
            game_stack.clone(),
            game_positions_accumulator.clone(),
            round_positions_accumulator.clone(),
            tile_accumulator.clone(),
        );
    }
}

fn start_game_worker(
    game_stack: Arc<Mutex<Vec<(Board, u8)>>>,
    game_positions_accumulator: Arc<Mutex<AtomicPositionAccumulator>>,
) {
    loop {
        let (next_board, depth) = match game_stack.clone().lock().unwrap().pop() {
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
    round_stack: Arc<Mutex<Vec<(Board, u8)>>>,
    game_stack: Arc<Mutex<Vec<(Board, u8)>>>,
    game_positions_accumulator: Arc<Mutex<AtomicPositionAccumulator>>,
    round_positions_accumulator: Arc<Mutex<AtomicPositionAccumulator>>,
    tile_accumulator: Arc<Mutex<AtomicTileAccumulator>>,
) {
    if depth == 0 {
        let mut round_lock = round_positions_accumulator.lock().unwrap();
        *round_lock += terminal_node_heuristic(board);
        return;
    } else if board.is_terminal() {
        let mut round_lock = round_positions_accumulator.lock().unwrap();
        *round_lock += board.camel_order();
        return;
    }

    if board.all_rolled() {
        let mut game_stack = game_stack.lock().unwrap();
        game_stack.push((board, depth));
        return;
    }

    let mut new_boards = Vec::new();
    for roll in board.potential_moves() {
        let next_board = board.update(&roll);
        new_boards.push((next_board, depth - 1));
    }

    {
        let mut round_stack = round_stack.lock().unwrap();
        round_stack.append(&mut new_boards);
    }
}

fn update_game_state(
    board: Board,
    depth: u8,
    game_stack: Arc<Mutex<Vec<(Board, u8)>>>,
    game_positions_accumulator: Arc<Mutex<AtomicPositionAccumulator>>,
) {
    if depth == 0 {
        let mut round_lock = game_positions_accumulator.lock().unwrap();
        *round_lock += terminal_node_heuristic(board);
        return;
    } else if board.is_terminal() {
        let mut round_lock = game_positions_accumulator.lock().unwrap();
        *round_lock += board.camel_order();
        return;
    }

    let mut new_boards = Vec::new();
    for roll in board.potential_moves() {
        let next_board = board.update(&roll);
        new_boards.push((next_board, depth - 1));
    }

    {
        let mut game_stack = game_stack.lock().unwrap();
        game_stack.append(&mut new_boards);
    }
}

fn terminal_node_heuristic(board: board::Board) -> board::CamelOrder {
    return board.camel_order();
}

fn terminal_round_states_from_board(board: board::Board) -> u32 {
    let num_unrolled = board.num_unrolled() as u32;
    return num_unrolled.pow(constants::MAX_ROLL as u32);
}
