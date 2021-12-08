use std::{collections::LinkedList, ffi::NulError};

const BOARD_SIZE: usize = 16;
const NUM_CAMELS: usize = 5;

#[derive(Copy, Clone)]
struct Camel {
    camel: u8,
    tile: u8,
    position: u8,
    has_rolled: bool,
}

#[derive(Copy, Clone)]
struct Roll {
    camel: u8,
    spaces: u8,
}

#[derive(Copy, Clone)]
struct Board {
    round: u8,
    camels: [Camel; NUM_CAMELS],
    traps: [i8; BOARD_SIZE],
}

struct SolveResult {
    game_winner_odds: [f64; NUM_CAMELS],
    round_winner_odds: [f64; NUM_CAMELS],
    round_tile_odds: [f64; BOARD_SIZE]
}

impl Board {
    fn new(camels: [Camel; NUM_CAMELS], traps: [i8; BOARD_SIZE]) -> Self {
        Board { round: 0, camels: camels, traps: traps }
    }
    fn update(&self, roll: Roll) -> Board {
        let mut new_board = self.clone();
        let mut current_space: Option<u8> = None;
        let mut current_position: Option<u8> = None;
        let mut target_space: Option<u8> = None;
        let mut target_position: u8 = 0;
        //Find the camel in question
        for camel in self.camels {
            if camel.camel == roll.camel {
                current_space = Some(camel.tile);
                current_position = Some(camel.position);
                target_space = Some(camel.tile + roll.spaces);
            }
        }
        //Figure out where we are going to
        for camel in self.camels {
            if camel.tile == target_space.unwrap() {
                if camel.position > target_position {
                    target_position = camel.position + 1;
                }
            }
        }
        //Update each of the camels
        for camel in new_board.camels.iter_mut() {
            if camel.tile == current_space.unwrap() {
                camel.tile = target_space.unwrap();
                camel.position = target_position;
                target_position += 1;
            }
        }
        return new_board
    }

    fn is_terminal(&self) -> bool {
        for camel in self.camels {
        }
        return false
    }

    fn get_permutations(&self) -> Vec<Board> {
        Vec::new()
    }

    fn solve(&self) -> SolveResult {
        let mut num_game_terminal = 0;
        let mut game_winners_accumulator: [i16; NUM_CAMELS] = [0; NUM_CAMELS];

        let mut num_round_terminal = 0;
        let mut round_winners_accumulator: [i16; NUM_CAMELS] = [0; NUM_CAMELS];
        let mut tile_landings_accumulator: [i16; BOARD_SIZE] = [0; BOARD_SIZE];

        let mut stack: LinkedList<&Board> = LinkedList::new();
        stack.push_back(&self);

        while let Some(current_node) = stack.pop_front() {
            for board in current_node.get_permutations() {
                if !board.is_terminal() {
                    stack.push_back(&board);
                } else {
                    num_game_terminal += 1;
                }
            }
        }
        let mut game_winner_odds = [0.0; NUM_CAMELS];
        for (idx, sum) in game_winners_accumulator.into_iter().enumerate() {
            game_winner_odds[idx] = *sum as f64 / num_game_terminal as f64;
        }
        let mut round_winner_odds = [0.0; NUM_CAMELS];
        for (idx, sum) in round_winners_accumulator.into_iter().enumerate() {
            round_winner_odds[idx] = *sum as f64 / num_round_terminal as f64;
        }
        let mut round_tile_odds = [0.0; BOARD_SIZE];
        for (idx, sum) in tile_landings_accumulator.into_iter().enumerate() {
            round_tile_odds[idx] = *sum as f64 / num_round_terminal as f64;
        }
        return SolveResult {
            game_winner_odds: game_winner_odds,
            round_winner_odds: round_winner_odds,
            round_tile_odds: round_tile_odds,
        };
    }
}

fn main() {
    let camels: [Camel; 5] = [
        Camel { camel: 1, tile: 0, position: 0, has_rolled: false },
        Camel { camel: 2, tile: 0, position: 1, has_rolled: false },
        Camel { camel: 3, tile: 2, position: 0, has_rolled: false },
        Camel { camel: 4, tile: 2, position: 1, has_rolled: false },
        Camel { camel: 5, tile: 1, position: 0, has_rolled: false }
    ];
    let traps = [0; 16];
    let mut board = Board::new(camels, traps);
    let roll = Roll {
        camel: 1,
        spaces: 2,
    };
    board.update(roll);
    for camel in board.camels {
        println!("Found camel {} at space {} and position {}", camel.camel, camel.tile, camel.position);
    }
}