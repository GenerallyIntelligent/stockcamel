use crate::constants;

pub type Camel = u8;
const ROLL_MASK: u8 = 1;

pub fn camel_has_rolled(camel: &Camel) -> bool {
    (camel & ROLL_MASK) == 1
}

pub fn camel_has_finished(camel: &Camel) -> bool {
    ((camel >> 1) / constants::NUM_CAMELS as u8) > constants::BOARD_SIZE as u8
}

pub fn camel_tile_and_position(camel: &Camel) -> (usize, usize) {
    let tile_and_pos = camel >> 1;
    return (
        tile_and_pos as usize / constants::NUM_CAMELS,
        tile_and_pos as usize % constants::NUM_CAMELS,
    );
}

pub fn camel_tile(camel: &Camel) -> usize {
    return (camel >> 1) as usize / constants::NUM_CAMELS;
}

pub fn camel_position(camel: &Camel) -> usize {
    return (camel >> 1) as usize % constants::NUM_CAMELS;
}

pub fn camel_roll(camel: &Camel) -> u8 {
    return camel & ROLL_MASK;
}

pub fn create_camel(tile: &usize, position: &usize, is_rolled: &u8) -> Camel {
    let tile_and_pos: u8 = ((tile * constants::NUM_CAMELS) + position) as u8;
    return (tile_and_pos << 1) | is_rolled;
}

pub fn update_camel(camel: &Camel, tile: &usize, position: &usize) -> Camel {
    create_camel(tile, position, &camel_roll(camel))
}

pub fn set_roll_true(camel: &Camel) -> Camel {
    return camel | 1;
}

pub fn set_roll_false(camel: &Camel) -> Camel {
    return camel & 254;
}
