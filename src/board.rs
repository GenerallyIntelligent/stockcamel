use crate::constants;
use std::fmt;

pub type Camel = u8;
pub type Camels = [Camel; constants::NUM_CAMELS];
pub type Terrain = [bool; constants::BOARD_SIZE];
pub type CamelOrder = [usize; constants::NUM_CAMELS];

const TILE_MASK: u8 = 240;
const POSITION_MASK: u8 = 14;
const ROLL_MASK: u8 = 1;
const WINNING_CAMEL: u8 = 255;

#[derive(Copy, Clone)]
pub struct Board {
    pub camels: [Camel; constants::NUM_CAMELS],
    pub oasis: Terrain,
    pub desert: Terrain,
}

#[derive(Copy, Clone)]
pub struct Roll {
    pub camel: usize,
    pub tiles: u8,
}

impl Board {
    pub fn new(camels: Camels, oasis: Terrain, desert: Terrain) -> Self {
        Board {
            camels: camels,
            oasis: oasis,
            desert: desert,
        }
    }

    pub fn update(&self, roll: &Roll) -> Board {
        let mut new_board = self.clone();
        let camel = self.camels[roll.camel];
        let current_tile = (camel & TILE_MASK) >> 4;
        let current_position = (camel & POSITION_MASK) >> 1;

        let mut target_tile = current_tile + roll.tiles;
        if target_tile >= 16 {
            panic!("HAVE NOT HANDLED WINNING!");
            // This is a winning roll
        }

        let mut camels_updating = [usize::MAX; constants::NUM_CAMELS];
        camels_updating[current_position as usize] = roll.camel;
        let mut target_position = 0;

        if self.desert[target_tile as usize] {
            target_tile = target_tile - 1;
            let mut displaced_camels = Vec::with_capacity(4);
            let mut num_updating_camels: u8 = 0;
            for (camel_num, camel) in self.camels.iter().enumerate() {
                let camel_tile = (camel & TILE_MASK) >> 4;
                if camel_tile == current_tile {
                    let camel_position = (camel & POSITION_MASK) >> 1;
                    if camel_position > current_position {
                        camels_updating[camel_position as usize] = camel_num;
                        num_updating_camels += 1;
                    }
                } else if camel_tile == target_tile {
                    displaced_camels.push(camel_num);
                }
            }
            for camel_num in displaced_camels {
                new_board.camels[camel_num] = self.camels[camel_num] + (num_updating_camels << 1)
            }
        } else {
            if self.oasis[target_tile as usize] {
                target_tile = target_tile + 1;
            }
            for (camel_num, camel) in self.camels.iter().enumerate() {
                let camel_tile = (camel & TILE_MASK) >> 4;
                if camel_tile == target_tile {
                    let camel_position = (camel & POSITION_MASK) >> 1;
                    target_position = target_position.max(camel_position + 1);
                } else if camel_tile == current_tile {
                    let camel_position = (camel & POSITION_MASK) >> 1;
                    if camel_position > current_position {
                        camels_updating[camel_position as usize] = camel_num;
                    }
                }
            }
        }
        for camel_num in camels_updating {
            if camel_num == usize::MAX {
                continue;
            }
            new_board.camels[camel_num] = 0
                | (target_position << 1)
                | (target_tile << 4)
                | (self.camels[camel_num] & ROLL_MASK);
            target_position += 1;
        }
        new_board.camels[roll.camel] |= 1;
        return new_board;
    }

    pub fn update_with_target(&self, roll: &Roll) -> (Board, usize) {
        let camel = self.camels[roll.camel];
        let current_tile = (camel & TILE_MASK) >> 4;
        let original_target_tile = current_tile + roll.tiles;
        let board = self.update(roll);
        return (board, original_target_tile as usize);
    }

    // Uses the optimal sorting graph for 5 elements to find the permutation
    // This relies on the fact that the camels will sort by tiles, then position
    pub fn camel_order(&self) -> CamelOrder {
        // Camel numbers indexed by the position
        let mut permutation: [usize; constants::NUM_CAMELS] = [0, 1, 2, 3, 4];

        fn compare_swap(
            permutation: &mut [usize],
            camels: &Camels,
            large_index: usize,
            small_index: usize,
        ) {
            if camels[permutation[large_index]] < camels[permutation[small_index]] {
                permutation.swap(large_index, small_index);
            }
        }

        // One of the optimal sorting graphs for 5 elements
        compare_swap(&mut permutation, &self.camels, 0, 1);
        compare_swap(&mut permutation, &self.camels, 2, 3);
        compare_swap(&mut permutation, &self.camels, 0, 2);
        compare_swap(&mut permutation, &self.camels, 1, 4);
        compare_swap(&mut permutation, &self.camels, 0, 1);
        compare_swap(&mut permutation, &self.camels, 2, 3);
        compare_swap(&mut permutation, &self.camels, 1, 2);
        compare_swap(&mut permutation, &self.camels, 3, 4);
        compare_swap(&mut permutation, &self.camels, 2, 3);

        return permutation;
    }

    pub fn is_terminal(&self) -> bool {
        for camel in self.camels {
            if camel == WINNING_CAMEL {
                return true;
            }
        }
        return false;
    }

    pub fn all_rolled(&self) -> bool {
        for camel in self.camels {
            if (camel & ROLL_MASK) == 0 {
                return false;
            }
        }
        return true;
    }

    pub fn num_unrolled(&self) -> u8 {
        let mut num_unrolled = 0;
        for camel in self.camels {
            if (camel & ROLL_MASK) == 0 {
                num_unrolled += 1;
            }
        }
        num_unrolled
    }

    pub fn potential_moves(&self) -> Vec<Roll> {
        let mut potential_moves = Vec::new();
        for (camel_num, camel) in self.camels.iter().enumerate() {
            if (camel & ROLL_MASK) == 0 {
                for die_roll in 1..(constants::MAX_ROLL + 1) {
                    let roll = Roll {
                        camel: camel_num,
                        tiles: die_roll,
                    };
                    potential_moves.push(roll);
                }
            }
        }
        return potential_moves;
    }

    pub fn hash() -> [u8; constants::NUM_CAMELS + 4] {
        return [0; constants::NUM_CAMELS + 4];
    }
}

// TODO: Fix the display implementation for boards...
// impl fmt::Display for Board {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         for (camel_num, camels) in self.camels.iter().enumerate() {
//             if tile < 16 {
//                 write!(f, "|")?;
//                 for camel in camels {
//                     if *camel == 0 {
//                         write!(f, " |")?;
//                     } else {
//                         write!(f, "{}|", camel)?;
//                     }
//                 }
//                 if self.oasis[tile] {
//                     write!(f, "[+]")?;
//                 } else if self.desert[tile] {
//                     write!(f, "[-]")?;
//                 } else {
//                 }
//                 write!(f, "\n")?;
//             } else {
//                 write!(f, "#")?;
//                 for camel in camels {
//                     if *camel == 0 {
//                         write!(f, " #")?;
//                     } else {
//                         write!(f, "{}#", camel)?;
//                     }
//                 }
//                 write!(f, "\n")?;
//             }
//         }
//         Ok(())
//     }
// }
