
use std::fmt;
use std::sync::atomic;
use std::thread;
use std::sync::{Arc, Mutex, RwLock};

use crate::constants;
use crate::board;
use crate::concurrent::{TileAccumulator, PositionAccumulator};

#[derive(Copy, Clone)]
pub struct CamelOdds {
    pub odds: [[f64; constants::NUM_CAMELS]; constants::NUM_CAMELS], //Odds of a camel getting a position, indexed by camel then position
}

#[derive(Copy, Clone)]
pub struct TileOdds {
    pub odds: [f64; constants::BOARD_SIZE],
}

impl fmt::Display for CamelOdds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: <5}", "Camel")?;
        for i in 1..self.odds.len() + 1 {
            write!(f, " | Pos {: <1}", i)?;
        }
        write!(f, "\n")?;
        for (camel_number, odds) in self.odds.iter().enumerate() {
            write!(f, "{: <5}", camel_number + 1)?;
            for odd in odds.iter() {
                write!(f, " | {:.3}", odd)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl fmt::Display for TileOdds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: <5}", "Tile")?;
        for i in 1..self.odds.len() + 1 {
            write!(f, " | {: <4}", i)?;
        }
        write!(f, "\n")?;
        write!(f, "{: <5}", "Odds")?;
        for odd in self.odds.iter() {
            write!(f, " | {:.2}", odd)?;
        }
        write!(f, "\n")?;
        Ok(())
    }
}

pub fn solve_game_from(board: board::Board, depth: u8, workers: u8) -> CamelOdds {
    let depth = depth - 2;
    let position_accumulator = Arc::new(PositionAccumulator::new());

    // May want to change to the parking lot mutex
    let shared_stack: Arc<RwLock<Vec<(u8, board::Board)>>> = Arc::new(RwLock::new(Vec::new()));
    {
        for roll in board.potential_moves() {
            let (board, _) = board.update(roll);
            shared_stack.write().unwrap().push((0, board));
        }
    }

    let mut worker_handles = Vec::new();
    for worker_id in 0..workers {
        let shared_stack = Arc::clone(&shared_stack);
        let position_accumulator = Arc::clone(&position_accumulator);
        let handle = thread::spawn(move || {
            // println!("Starting worker {}", worker_id);
            let mut stack: Vec<(u8, board::Board)> = Vec::new();
            loop {
                let stack_result = stack.pop();
                match stack_result {
                    Some((current_depth, current_node)) => {
                        // println!("Found something on local stack");
                        for roll in current_node.potential_moves() {
                            let (next_node, _) = current_node.update(roll);
                            if !next_node.is_terminal() && current_depth < depth {
                                stack.push((current_depth + 1, next_node));
                            } else {
                                let positions = next_node.camel_order();
                                for (position, camel_num) in positions.iter().enumerate() {
                                    position_accumulator[*camel_num as usize - 1][position].fetch_add(1, atomic::Ordering::Relaxed);
                                }
                            }
                        }
                    },
                    None => {
                        // println!("Found nothing on local stack");
                        let mutex_out = shared_stack.read().unwrap();
                        let len = mutex_out.len();
                        drop(mutex_out);
                        match len {
                            0 => {
                                // println!("Found nothing on shared stack");
                                // println!("Worker {} joining", worker_id);
                                break
                            },
                            1 => {
                                // println!("Found one on shared stack");
                                let current_depth;
                                let current_node;
                                let mutex_out = shared_stack.write().unwrap().pop();
                                let shared_stack_result = mutex_out.clone();
                                drop(mutex_out);
                                match shared_stack_result {
                                    Some((d, n)) => {
                                        current_depth = d;
                                        current_node = n;
                                    },
                                    None => continue
                                }
                                for roll in current_node.potential_moves() {
                                    let (next_node, _) = current_node.update(roll);
                                    if !next_node.is_terminal() && current_depth < depth {
                                        shared_stack.write().unwrap().push((current_depth + 1, next_node));
                                    } else {
                                        let positions = next_node.camel_order();
                                        for (position, camel_num) in positions.iter().enumerate() {
                                            position_accumulator[*camel_num as usize - 1][position].fetch_add(1, atomic::Ordering::Relaxed);
                                        }
                                    }
                                }
                            }
                            _ => {
                                // println!("Found many on shared stack");
                                let current_depth;
                                let current_node;
                                match shared_stack.write().unwrap().pop() {
                                    Some((d, n)) => {
                                        // println!("Got node from shared stack");
                                        current_depth = d;
                                        current_node = n;
                                    },
                                    None => {
                                        // println!("Shared stack is empty");
                                        continue
                                    }
                                }
                                for roll in current_node.potential_moves() {
                                    let (next_node, _) = current_node.update(roll);
                                    if !next_node.is_terminal() && current_depth < depth {
                                        stack.push((current_depth + 1, next_node));
                                    } else {
                                        let positions = next_node.camel_order();
                                        for (position, camel_num) in positions.iter().enumerate() {
                                            position_accumulator[*camel_num as usize - 1][position].fetch_add(1, atomic::Ordering::Relaxed);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
        worker_handles.push(handle);
    }

    for handle in worker_handles {
        handle.join().unwrap();
    }

    let mut num_terminal = 0;
    for position_num in &position_accumulator[0] {
        num_terminal += position_num.load(atomic::Ordering::Relaxed);
    }
    let mut position_odds = [[0.0; constants::NUM_CAMELS]; constants::NUM_CAMELS];
    for (x, vector) in position_accumulator.positions.iter().enumerate() {
        for (y, sum) in vector.iter().enumerate() {
            let sum = sum.load(atomic::Ordering::Relaxed);
            position_odds[x][y] = sum as f64 / num_terminal as f64;
        }
    }
    CamelOdds {odds: position_odds}
}

pub fn solve_round_from(board: board::Board, workers: u8) -> (CamelOdds, TileOdds) {
    let position_accumulator = Arc::new(PositionAccumulator::new());
    let tile_landings_accumulator = Arc::new(TileAccumulator::new());

    let stack = Arc::new(Mutex::new(Vec::new()));
    {
        for roll in board.potential_moves() {
            let (board, landing_position) = board.update(roll);
            let mut tile_landings: [u16; constants::BOARD_SIZE] = [0; constants::BOARD_SIZE];
            tile_landings[landing_position] += 1;
            stack.lock().unwrap().push((tile_landings, board));
        }
    }

    let mut worker_handles = Vec::new();
    for _ in 0..workers {
        let stack = Arc::clone(&stack);
        let position_accumulator = Arc::clone(&position_accumulator);
        let tile_landings_accumulator = Arc::clone(&tile_landings_accumulator);
        let handle = thread::spawn(move || {
            loop {
                let mutex_guard = stack.lock().unwrap().pop();
                let popped_value = mutex_guard.clone();
                drop(mutex_guard);
                match popped_value {
                    Some((tile_landings, current_node)) => {
                        for roll in current_node.potential_moves() {
                            let (next_node, landing_position) = current_node.update(roll);
                            let mut next_tile_landings = tile_landings.clone();
                            next_tile_landings[landing_position] += 1;
                            if next_node.all_rolled() || next_node.is_terminal() {
                                let positions = next_node.camel_order();
                                for (position, camel_num) in positions.iter().enumerate() {
                                    position_accumulator[*camel_num as usize - 1][position].fetch_add(1, atomic::Ordering::Relaxed);
                                }
                                for (idx, value) in next_tile_landings.iter().enumerate() {
                                    tile_landings_accumulator[idx].fetch_add(*value as u32, atomic::Ordering::Relaxed);
                                }
                            } else {
                                stack.lock().unwrap().push((next_tile_landings, next_node));
                            }
                        }
                    },
                    None => break,
                }
            }
        });
        worker_handles.push(handle);
    }

    for handle in worker_handles {
        handle.join().unwrap();
    }

    let mut num_terminal = 0;
    for position_num in &position_accumulator[0] {
        num_terminal += position_num.load(atomic::Ordering::Relaxed);
    }
    let mut position_odds = [[0.0; constants::NUM_CAMELS]; constants::NUM_CAMELS];
    for (x, vector) in position_accumulator.positions.iter().enumerate() {
        for (y, sum) in vector.iter().enumerate() {
            let sum = sum.load(atomic::Ordering::Relaxed);
            position_odds[x][y] = sum as f64 / num_terminal as f64;
        }
    }
    let mut tile_odds = [0.0; constants::BOARD_SIZE];
    for (idx, sum) in tile_landings_accumulator.tiles.iter().enumerate() {
        let sum: u32 = sum.load(atomic::Ordering::Relaxed);
        tile_odds[idx] = sum as f64 / num_terminal as f64;
    }
    (CamelOdds{odds: position_odds}, TileOdds{odds: tile_odds})
}