use std::collections::LinkedList;
use std::fmt;

const BOARD_SIZE: usize = 16;
const NUM_CAMELS: usize = 5;
const MAX_ROLL: u8 = 3;

#[derive(Copy, Clone)]
struct Roll {
    camel: u8,
    spaces: u8,
}

#[derive(Copy, Clone)]
struct Board {
    round: u8,
    positions: [[u8; NUM_CAMELS]; BOARD_SIZE + 1],
    rolls: [bool; NUM_CAMELS],
    oasis: [bool; BOARD_SIZE],
    desert: [bool; BOARD_SIZE],
}

struct SolveResult {
    game_position_odds: [[f64; NUM_CAMELS]; NUM_CAMELS], //Odds of a camel getting a position, indexed by camel then position
    round_position_odds: [[f64; NUM_CAMELS]; NUM_CAMELS],
    round_tile_odds: [f64; BOARD_SIZE]
}

impl Board {
    fn new(positions: [[u8; NUM_CAMELS]; BOARD_SIZE + 1], rolls: [bool; NUM_CAMELS], oasis: [bool; BOARD_SIZE], desert: [bool; BOARD_SIZE]) -> Self {
        Board {
            round: 0,
            positions: positions,
            rolls: rolls,
            oasis: oasis,
            desert: desert
        }
    }
    
    fn update(&self, roll: Roll) -> Board {
        let mut new_board = self.clone();
        let (current_tile, current_position) = self.get_location(roll.camel);
        let mut target_tile = usize::min(current_tile + roll.spaces as usize, 16);

        if target_tile >= 16 || !self.desert[target_tile] {
            if !(target_tile >= 16) && self.oasis[target_tile] {
                target_tile += 1;
            }
            let mut target_position = 0;
            for camel_num in self.positions[target_tile] {
                if camel_num <= 0 {
                    break
                }
                target_position += 1;
            }
            let moving_stack_height = 5 - usize::max(current_position, target_position);
            let moving_slice = &self.positions[current_tile][current_position..current_position + moving_stack_height];
            new_board.positions[target_tile][target_position..target_position + moving_stack_height].clone_from_slice(moving_slice);
            let static_slice = &self.positions[target_tile][target_position..target_position + moving_stack_height];
            new_board.positions[current_tile][current_position..current_position + moving_stack_height].clone_from_slice(static_slice);
        } else {
            target_tile -= 1;
            let mut stack_height = 0;
            for camel_num in self.positions[current_tile] {
                if camel_num <= 0 {
                    break
                }
                stack_height += 1;
            }
            let moving_stack_height = stack_height - current_position;
            let static_slice = &self.positions[target_tile + 1][5 - moving_stack_height..5];
            new_board.positions[current_tile][current_position..current_position + moving_stack_height].clone_from_slice(static_slice);
            let preexisting_slice_height = 5 - moving_stack_height;
            let preexisting_stack = new_board.positions[target_tile];
            new_board.positions[target_tile][moving_stack_height..moving_stack_height + preexisting_slice_height].clone_from_slice(&preexisting_stack[0..preexisting_slice_height]);
            let moving_slice = &self.positions[current_tile][current_position..current_position + moving_stack_height];
            new_board.positions[target_tile][0..moving_stack_height].clone_from_slice(moving_slice);
        }
        
        if new_board.all_rolled() {
            new_board.rolls = [false; NUM_CAMELS];
            new_board.round += 1;
        }
        new_board.rolls[roll.camel as usize - 1] = true;
        return new_board
    }

    fn is_terminal(&self) -> bool {
        for camel in self.positions[BOARD_SIZE] {
            if camel > 0 {
                return true
            }
        }
        return false
    }

    fn get_location(&self, camel: u8) -> (usize, usize) {
        for (tile, stack) in self.positions.iter().enumerate() {
            for (position, candidate_camel) in stack.iter().enumerate() {
                if camel == *candidate_camel {
                    return (tile, position)
                }
            }
        }
        panic!("Tried to find a camel which does not exist!");
    }

    fn camel_order(&self) -> [u8; NUM_CAMELS] {
        let mut camel_order = [0; NUM_CAMELS];
        let mut idx = 5;
        for tile in self.positions {
            for camel in tile {
                if camel > 0 {
                    idx -= 1;
                    camel_order[idx] = camel;
                }
            }
        }
        return camel_order
    }

    fn all_rolled(&self) -> bool {
        for has_rolled in self.rolls {
            if !has_rolled {
                return false
            }
        }
        return true
    }

    fn potential_moves(&self) -> Vec<Roll> {
        let mut potential_moves = Vec::new();
        let camels_all_rolled = self.all_rolled();
        for (camel_num, has_rolled) in self.rolls.iter().enumerate() {
            if camels_all_rolled || !has_rolled {
                for die_roll in 1..(MAX_ROLL + 1) {
                    let roll = Roll {
                        camel: camel_num as u8 + 1,
                        spaces: die_roll,
                    };
                    potential_moves.push(roll);
                }
                
            }
        }
        return potential_moves
    }

    fn solve_game(&self, depth: u8) {

    }

    fn solve(&self, depth: u8) -> SolveResult {
        let current_round = self.round;

        let mut num_game_terminal = 0;
        let mut game_position_accumulator: [[u64; NUM_CAMELS]; NUM_CAMELS] = [[0; NUM_CAMELS]; NUM_CAMELS];

        let mut num_round_terminal = 0;
        let mut round_position_accumulator: [[u16; NUM_CAMELS]; NUM_CAMELS] = [[0; NUM_CAMELS]; NUM_CAMELS];
        let mut tile_landings_accumulator: [u16; BOARD_SIZE] = [0; BOARD_SIZE];

        let mut stack: LinkedList<Board> = LinkedList::new();
        stack.push_back(*self);

        while let Some(current_node) = stack.pop_front() {
            for roll in current_node.potential_moves() {
                let board = current_node.update(roll);
                if !board.is_terminal() {
                    if board.round - current_round >= depth {
                        num_game_terminal += 1;
                        let positions = board.camel_order();
                        for (position, camel_num) in positions.iter().enumerate() {
                            game_position_accumulator[*camel_num as usize - 1][position] += 1;
                        }
                    } else {
                        if board.round == current_round {
                            let (tile, _) = self.get_location(roll.camel);
                            let landed_tile = tile + roll.spaces as usize;
                            tile_landings_accumulator[landed_tile] += 1;
                            if board.all_rolled() {
                                num_round_terminal += 1;
                                let positions = board.camel_order();
                                for (position, camel_num) in positions.iter().enumerate() {
                                    round_position_accumulator[*camel_num as usize - 1][position] += 1;
                                }
                            }
                        }
                        stack.push_front(board);
                    }
                } else {
                    num_game_terminal += 1;
                    let positions = board.camel_order();
                    for (position, camel_num) in positions.iter().enumerate() {
                        game_position_accumulator[*camel_num as usize - 1][position] += 1;
                    }
                }
            }
        }
        let mut game_position_odds = [[0.0; NUM_CAMELS]; NUM_CAMELS];
        for (x, vector) in game_position_accumulator.iter().enumerate() {
            for (y, sum) in vector.iter().enumerate() {
                game_position_odds[x][y] = *sum as f64 / num_game_terminal as f64;
            }
        }
        let mut round_position_odds = [[0.0; NUM_CAMELS]; NUM_CAMELS];
        for (x, vector) in round_position_accumulator.iter().enumerate() {
            for (y, sum) in vector.iter().enumerate() {
                round_position_odds[x][y] = *sum as f64 / num_round_terminal as f64;
            }
        }
        let mut round_tile_odds = [0.0; BOARD_SIZE];
        for (idx, sum) in tile_landings_accumulator.iter().enumerate() {
            round_tile_odds[idx] = *sum as f64 / num_round_terminal as f64;
        }
        return SolveResult {
            game_position_odds: game_position_odds,
            round_position_odds: round_position_odds,
            round_tile_odds: round_tile_odds,
        };
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (tile, camels) in self.positions.iter().enumerate() {
            if tile < 16 {
                write!(f, "|")?;
                for camel in camels {
                    if *camel == 0 {
                        write!(f, " |")?;
                    } else {
                        write!(f, "{}|", camel)?;
                    }
                }
                if self.oasis[tile] {
                    write!(f, "[+]")?;
                } else if self.desert[tile] {
                    write!(f, "[-]")?;
                } else {

                }
                write!(f, "\n")?;
            } else {
                write!(f, "#")?;
                for camel in camels {
                    if *camel == 0 {
                        write!(f, " #")?;
                    } else {
                        write!(f, "{}#", camel)?;
                    }
                }
                write!(f, "\n")?;
            }
            
        }
        Ok(())
    }
}

fn main() {
    let mut positions = [[0; NUM_CAMELS]; BOARD_SIZE + 1];
    for index in 0..NUM_CAMELS {
        positions[0][index] = index as u8 + 1;
    }
    let rolls = [false; 5];
    let oasis = [false; 16];
    let mut desert = [false; 16];
    desert[2] = true;
    let board = Board::new(positions, rolls, oasis, desert);
    // let roll = Roll {
    //     camel: 1,
    //     spaces: 1,
    // };
    // println!("{} {}", roll.camel, roll.spaces);
    // board = board.update(roll);
    // println!("{}", board);
    // let roll = Roll {
    //     camel: 5,
    //     spaces: 3,
    // };
    // println!("{} {}", roll.camel, roll.spaces);
    // board = board.update(roll);
    // println!("{}", board);
    // let roll = Roll {
    //     camel: 2,
    //     spaces: 1,
    // };
    // println!("{} {}", roll.camel, roll.spaces);
    // board = board.update(roll);
    // println!("{}", board);

    // for _ in 0..100000 {
    //     println!("Start");
    //     let mut board = Board::new(positions, rolls, oasis, desert);
    //     for _ in 0..10 {
    //         if board.is_terminal() {
    //             break
    //         }
    //         let roll = Roll {
    //             camel: rand::thread_rng().gen_range(1..6),
    //             spaces: rand::thread_rng().gen_range(1..4),
    //         };
    //         println!("{} {}", roll.camel, roll.spaces);
    //         board = board.update(roll);
    //         println!("{}", board);
    //     }
    // }
    board.solve(2);
}