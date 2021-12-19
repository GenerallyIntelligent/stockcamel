use std::fmt;
use crate::constants;

#[derive(Copy, Clone)]
pub struct Roll {
    camel: u8,
    spaces: u8,
}

#[derive(Copy, Clone)]
pub struct Board {
    positions: [[u8; constants::NUM_CAMELS]; constants::BOARD_SIZE + 1],
    rolls: [bool; constants::NUM_CAMELS],
    oasis: [bool; constants::BOARD_SIZE],
    desert: [bool; constants::BOARD_SIZE],
}

impl Board {
    pub fn new(positions: [[u8; constants::NUM_CAMELS]; constants::BOARD_SIZE + 1], rolls: [bool; constants::NUM_CAMELS], oasis: [bool; constants::BOARD_SIZE], desert: [bool; constants::BOARD_SIZE]) -> Self {
        Board {
            positions: positions,
            rolls: rolls,
            oasis: oasis,
            desert: desert
        }
    }
    
    pub fn update(&self, roll: Roll) -> (Board, usize) {
        let mut new_board = self.clone();
        let (current_tile, current_position) = self.find_camel(roll.camel);
        let original_target_tile = usize::min(current_tile + roll.spaces as usize, 16);
        let mut target_tile = original_target_tile;

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
            new_board.rolls = [false; constants::NUM_CAMELS];
        }
        new_board.rolls[roll.camel as usize - 1] = true;
        return (new_board, original_target_tile);
    }

    pub fn is_terminal(&self) -> bool {
        for camel in self.positions[constants::BOARD_SIZE] {
            if camel > 0 {
                return true
            }
        }
        return false
    }

    pub fn find_camel(&self, camel: u8) -> (usize, usize) {
        for (tile, stack) in self.positions.iter().enumerate() {
            for (position, candidate_camel) in stack.iter().enumerate() {
                if camel == *candidate_camel {
                    return (tile, position)
                }
            }
        }
        panic!("Tried to find a camel which does not exist!");
    }

    pub fn camel_order(&self) -> [u8; constants::NUM_CAMELS] {
        let mut camel_order = [0; constants::NUM_CAMELS];
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

    pub fn all_rolled(&self) -> bool {
        for has_rolled in self.rolls {
            if !has_rolled {
                return false
            }
        }
        return true
    }

    pub fn potential_moves(&self) -> Vec<Roll> {
        let mut potential_moves = Vec::new();
        let camels_all_rolled = self.all_rolled();
        for (camel_num, has_rolled) in self.rolls.iter().enumerate() {
            if camels_all_rolled || !has_rolled {
                for die_roll in 1..(constants::MAX_ROLL + 1) {
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