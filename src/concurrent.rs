use std::sync::atomic;
use std::ops::Index;

use crossbeam::utils::CachePadded;

use crate::constants;

pub struct PositionAccumulator {
    pub positions: [[CachePadded<atomic::AtomicU32>; constants::NUM_CAMELS]; constants::NUM_CAMELS],
}

impl PositionAccumulator {
    pub fn new() -> Self {
        PositionAccumulator {
            positions: [
                [CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0))],
                [CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0))],
                [CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0))],
                [CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0))],
                [CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0)), CachePadded::new(atomic::AtomicU32::new(0))]
            ]
        }
    }
}

impl Index<usize> for PositionAccumulator
{
    type Output = [CachePadded<atomic::AtomicU32>; constants::NUM_CAMELS];
    fn index(&self, index: usize) -> &Self::Output {
        &self.positions[index]
    }
}

pub struct TileAccumulator {
    pub tiles: [CachePadded<atomic::AtomicU32>; constants::BOARD_SIZE],
}

impl TileAccumulator {
    pub fn new() -> Self {
        TileAccumulator {
            tiles: [
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
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
