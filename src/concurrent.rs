use std::sync::atomic;
use std::ops::Index;
use crate::constants;

pub struct PositionAccumulator {
    positions: [[atomic::AtomicU32; constants::NUM_CAMELS]; constants::NUM_CAMELS],
}

impl PositionAccumulator {
    pub fn new() -> Self {
        PositionAccumulator {
            positions: [
                [atomic::AtomicU32::new(0), atomic::AtomicU32::new(0), atomic::AtomicU32::new(0), atomic::AtomicU32::new(0), atomic::AtomicU32::new(0)],
                [atomic::AtomicU32::new(0), atomic::AtomicU32::new(0), atomic::AtomicU32::new(0), atomic::AtomicU32::new(0), atomic::AtomicU32::new(0)],
                [atomic::AtomicU32::new(0), atomic::AtomicU32::new(0), atomic::AtomicU32::new(0), atomic::AtomicU32::new(0), atomic::AtomicU32::new(0)],
                [atomic::AtomicU32::new(0), atomic::AtomicU32::new(0), atomic::AtomicU32::new(0), atomic::AtomicU32::new(0), atomic::AtomicU32::new(0)],
                [atomic::AtomicU32::new(0), atomic::AtomicU32::new(0), atomic::AtomicU32::new(0), atomic::AtomicU32::new(0), atomic::AtomicU32::new(0)]
            ]
        }
    }
}

impl Index<usize> for PositionAccumulator
{
    type Output = [atomic::AtomicU32; constants::NUM_CAMELS];
    fn index(&self, index: usize) -> &Self::Output {
        &self.positions[index]
    }
}

pub struct TileAccumulator {
    tiles: [atomic::AtomicU32; constants::BOARD_SIZE],
}

impl TileAccumulator {
    pub fn new() -> Self {
        TileAccumulator {
            tiles: [
                atomic::AtomicU32::new(0),
                atomic::AtomicU32::new(0),
                atomic::AtomicU32::new(0),
                atomic::AtomicU32::new(0),
                atomic::AtomicU32::new(0),
                atomic::AtomicU32::new(0),
                atomic::AtomicU32::new(0),
                atomic::AtomicU32::new(0),
                atomic::AtomicU32::new(0),
                atomic::AtomicU32::new(0),
                atomic::AtomicU32::new(0),
                atomic::AtomicU32::new(0),
                atomic::AtomicU32::new(0),
                atomic::AtomicU32::new(0),
                atomic::AtomicU32::new(0),
                atomic::AtomicU32::new(0),
            ]
        }
    }
}

impl Index<usize> for TileAccumulator
{
    type Output = atomic::AtomicU32;
    fn index(&self, index: usize) -> &Self::Output {
        &self.tiles[index]
    }
}
