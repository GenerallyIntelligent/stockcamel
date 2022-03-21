pub type Camel = u8;
const TILE_MASK: u8 = 240;
const POSITION_MASK: u8 = 14;
const ROLL_MASK: u8 = 1;
const WINNING_CAMEL: u8 = 255;

pub fn camel_has_rolled(camel: &Camel) -> bool {
    (camel & ROLL_MASK) == 1
}

pub fn camel_has_finished(camel: &Camel) -> bool {
    camel == &WINNING_CAMEL
}

pub fn camel_tile_and_position(camel: &Camel) -> (usize, usize) {
    let tile = ((camel & TILE_MASK) >> 4).into();
    let position = ((camel & POSITION_MASK) >> 1).into();
    return (tile, position);
}

pub fn camel_tile(camel: &Camel) -> usize {
    return ((camel & TILE_MASK) >> 4).into();
}

pub fn camel_position(camel: &Camel) -> usize {
    return ((camel & POSITION_MASK) >> 1).into();
}

pub fn camel_roll(camel: &Camel) -> u8 {
    return camel & ROLL_MASK;
}

pub fn create_camel(tile: &usize, position: &usize, is_rolled: &u8) -> Camel {
    return 0 | ((*tile as u8) << 4) | ((*position as u8) << 1) | is_rolled;
}

pub fn update_camel(camel: &Camel, tile: &usize, position: &usize) -> Camel {
    create_camel(tile, position, &camel_roll(camel))
}
