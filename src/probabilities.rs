
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
        for i in 1..self.odds.len() {
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
        for i in 1..self.odds.len() {
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
    let depth = depth - 1;
    let position_accumulator = Arc::new(RwLock::new([[0; constants::NUM_CAMELS]; constants::NUM_CAMELS]));

    let stack = Arc::new(Mutex::new(Vec::new()));
    {
        for roll in board.potential_moves() {
            let (board, _) = board.update(roll);
            stack.lock().unwrap().push((0, board));
        }
    }

    let mut worker_handles = Vec::new();
    for _ in 0..workers {
        let stack = Arc::clone(&stack);
        let position_accumulator = Arc::clone(&position_accumulator);
        let handle = thread::spawn(move || {
            loop {
                let mutex_guard = stack.lock().unwrap().pop();
                let popped_value = mutex_guard.clone();
                drop(mutex_guard);
                match popped_value {
                    Some((current_depth, current_node)) => {
                        for roll in current_node.potential_moves() {
                            let (next_node, _) = current_node.update(roll);
                            if !next_node.is_terminal() && current_depth < depth {
                                stack.lock().unwrap().push((current_depth + 1, next_node));
                            } else {
                                let positions = next_node.camel_order();
                                for (position, camel_num) in positions.iter().enumerate() {
                                    position_accumulator.write().unwrap()[*camel_num as usize - 1][position] += 1;
                                }
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

    let position_accumulator = position_accumulator.read().unwrap();
    let mut num_terminal = 0;
    for position_num in position_accumulator[0] {
        num_terminal += position_num;
    }
    let mut position_odds = [[0.0; constants::NUM_CAMELS]; constants::NUM_CAMELS];
    for (x, vector) in position_accumulator.iter().enumerate() {
        for (y, sum) in vector.iter().enumerate() {
            position_odds[x][y] = *sum as f64 / num_terminal as f64;
        }
    }
    CamelOdds {odds: position_odds}
}

pub fn solve_round_from(board: board::Board, workers: u8) -> (CamelOdds, TileOdds) {
    let position_accumulator = Arc::new(RwLock::new([[0; constants::NUM_CAMELS]; constants::NUM_CAMELS]));
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
                                    position_accumulator.write().unwrap()[*camel_num as usize - 1][position] += 1;
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

    let position_accumulator = position_accumulator.read().unwrap();

    let mut num_terminal = 0;
    for position_num in position_accumulator[0] {
        num_terminal += position_num;
    }
    let mut position_odds = [[0.0; constants::NUM_CAMELS]; constants::NUM_CAMELS];
    for (x, vector) in position_accumulator.iter().enumerate() {
        for (y, sum) in vector.iter().enumerate() {
            position_odds[x][y] = *sum as f64 / num_terminal as f64;
        }
    }
    let mut total_tile_landings = 0;
    for num_landings in tile_landings_accumulator.tiles.iter() {
        let num_landings: u32 = num_landings.load(atomic::Ordering::Relaxed);
        total_tile_landings += num_landings;
    }
    let mut tile_odds = [0.0; constants::BOARD_SIZE];
    for (idx, sum) in tile_landings_accumulator.tiles.iter().enumerate() {
        let sum: u32 = sum.load(atomic::Ordering::Relaxed);
        tile_odds[idx] = sum as f64 / num_terminal as f64;
    }
    println!("{}", num_terminal);
    println!("{}", total_tile_landings);
    (CamelOdds{odds: position_odds}, TileOdds{odds: tile_odds})
}