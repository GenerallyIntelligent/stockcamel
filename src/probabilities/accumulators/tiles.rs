use crate::constants;
use crossbeam::utils::CachePadded;
use std::ops::{AddAssign, Deref, DerefMut};
use std::sync::{atomic, Arc};

pub struct TileAccumulator([u32; constants::BOARD_SIZE]);

impl TileAccumulator {
    pub fn new() -> Self {
        TileAccumulator([0; constants::BOARD_SIZE])
    }
}

impl Deref for TileAccumulator {
    type Target = [u32; constants::BOARD_SIZE];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TileAccumulator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct AtomicTileAccumulator([CachePadded<atomic::AtomicU32>; constants::BOARD_SIZE]);

impl AtomicTileAccumulator {
    pub fn new() -> Self {
        AtomicTileAccumulator([
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
        ])
    }
}

impl Clone for AtomicTileAccumulator {
    fn clone(&self) -> Self {
        let new_accumulator = AtomicTileAccumulator::new();
        for (idx, val) in self.iter().enumerate() {
            let val = val.load(atomic::Ordering::Relaxed);
            new_accumulator[idx].store(val, atomic::Ordering::Relaxed);
        }
        new_accumulator
    }
}

impl Deref for AtomicTileAccumulator {
    type Target = [CachePadded<atomic::AtomicU32>; constants::BOARD_SIZE];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AtomicTileAccumulator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AddAssign<TileAccumulator> for AtomicTileAccumulator {
    fn add_assign(&mut self, rhs: TileAccumulator) {
        for (idx, val) in rhs.iter().enumerate() {
            self[idx].fetch_add(*val, atomic::Ordering::Relaxed);
        }
    }
}

impl From<&TileAccumulator> for AtomicTileAccumulator {
    fn from(tile_accumulator: &TileAccumulator) -> AtomicTileAccumulator {
        let atomic_tile_accumulator = AtomicTileAccumulator::new();
        for (idx, val) in tile_accumulator.iter().enumerate() {
            atomic_tile_accumulator[idx].store(*val, atomic::Ordering::Relaxed);
        }
        atomic_tile_accumulator
    }
}

impl From<TileAccumulator> for AtomicTileAccumulator {
    fn from(tile_accumulator: TileAccumulator) -> AtomicTileAccumulator {
        (&tile_accumulator).into()
    }
}

impl From<&AtomicTileAccumulator> for TileAccumulator {
    fn from(atomic_tile_accumulator: &AtomicTileAccumulator) -> TileAccumulator {
        let mut tile_accumulator = TileAccumulator::new();
        for (idx, val) in atomic_tile_accumulator.iter().enumerate() {
            tile_accumulator[idx] = val.load(atomic::Ordering::Relaxed);
        }
        tile_accumulator
    }
}

impl From<AtomicTileAccumulator> for TileAccumulator {
    fn from(atomic_tile_accumulator: AtomicTileAccumulator) -> TileAccumulator {
        (&atomic_tile_accumulator).into()
    }
}

impl From<&Arc<AtomicTileAccumulator>> for TileAccumulator {
    fn from(atomic_tile_accumulator: &Arc<AtomicTileAccumulator>) -> TileAccumulator {
        let mut tile_accumulator = TileAccumulator::new();
        for (idx, val) in atomic_tile_accumulator.iter().enumerate() {
            tile_accumulator[idx] = val.load(atomic::Ordering::Relaxed);
        }
        tile_accumulator
    }
}

impl From<Arc<AtomicTileAccumulator>> for TileAccumulator {
    fn from(atomic_tile_accumulator: Arc<AtomicTileAccumulator>) -> TileAccumulator {
        (&atomic_tile_accumulator).into()
    }
}
