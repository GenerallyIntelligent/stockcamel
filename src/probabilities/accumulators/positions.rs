use crate::{board, constants};
use crossbeam::utils::CachePadded;
use std::ops::{Add, AddAssign, Deref, DerefMut};
use std::sync::atomic;

pub struct PositionAccumulator([[u32; constants::NUM_CAMELS]; constants::NUM_CAMELS]);

impl PositionAccumulator {
    pub fn new() -> Self {
        PositionAccumulator([[0; constants::NUM_CAMELS]; constants::NUM_CAMELS])
    }

    pub fn count_terminal(&self) -> u32 {
        let mut num_terminal = 0;
        for position_num in self[0] {
            num_terminal += position_num;
        }
        num_terminal
    }
}

impl Deref for PositionAccumulator {
    type Target = [[u32; constants::NUM_CAMELS]; constants::NUM_CAMELS];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PositionAccumulator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Add for PositionAccumulator {
    type Output = PositionAccumulator;
    fn add(self, rhs: PositionAccumulator) -> Self::Output {
        let mut result = PositionAccumulator::new();
        for (x, (row_one, row_two)) in self.iter().zip(rhs.iter()).enumerate() {
            for (y, (val_one, val_two)) in row_one.iter().zip(row_two.iter()).enumerate() {
                result[x][y] = val_one + val_two;
            }
        }
        result
    }
}

pub struct AtomicPositionAccumulator(
    [[CachePadded<atomic::AtomicU32>; constants::NUM_CAMELS]; constants::NUM_CAMELS],
);

impl AtomicPositionAccumulator {
    pub fn new() -> Self {
        AtomicPositionAccumulator([
            [
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
            ],
            [
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
            ],
            [
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
            ],
            [
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
            ],
            [
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
                CachePadded::new(atomic::AtomicU32::new(0)),
            ],
        ])
    }

    pub fn count_terminal(&self) -> u32 {
        let mut num_terminal = 0;
        for position_num in &self[0] {
            num_terminal += position_num.load(atomic::Ordering::Relaxed);
        }
        num_terminal
    }
}

impl Clone for AtomicPositionAccumulator {
    fn clone(&self) -> Self {
        let new_accumulator = AtomicPositionAccumulator::new();
        for (x, vector) in self.iter().enumerate() {
            for (y, val) in vector.iter().enumerate() {
                let val = val.load(atomic::Ordering::Relaxed);
                new_accumulator[x][y].store(val, atomic::Ordering::Relaxed);
            }
        }
        new_accumulator
    }
}

impl Deref for AtomicPositionAccumulator {
    type Target = [[CachePadded<atomic::AtomicU32>; constants::NUM_CAMELS]; constants::NUM_CAMELS];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AtomicPositionAccumulator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AddAssign<board::CamelOrder> for AtomicPositionAccumulator {
    fn add_assign(&mut self, rhs: board::CamelOrder) {
        for (position, camel) in rhs.iter().enumerate() {
            self[*camel as usize - 1][position].fetch_add(1, atomic::Ordering::Relaxed);
        }
    }
}

impl AddAssign<PositionAccumulator> for AtomicPositionAccumulator {
    fn add_assign(&mut self, rhs: PositionAccumulator) {
        for (x, vector) in rhs.iter().enumerate() {
            for (y, val) in vector.iter().enumerate() {
                self[x][y].fetch_add(*val, atomic::Ordering::Relaxed);
            }
        }
    }
}

impl From<&PositionAccumulator> for AtomicPositionAccumulator {
    fn from(position_accumulator: &PositionAccumulator) -> AtomicPositionAccumulator {
        let atomic_position_accumulator = AtomicPositionAccumulator::new();
        for (x, row) in position_accumulator.iter().enumerate() {
            for (y, val) in row.iter().enumerate() {
                atomic_position_accumulator[x][y].store(*val, atomic::Ordering::Relaxed);
            }
        }
        atomic_position_accumulator
    }
}

impl From<PositionAccumulator> for AtomicPositionAccumulator {
    fn from(position_accumulator: PositionAccumulator) -> AtomicPositionAccumulator {
        (&position_accumulator).into()
    }
}

impl From<&AtomicPositionAccumulator> for PositionAccumulator {
    fn from(atomic_position_accumulator: &AtomicPositionAccumulator) -> PositionAccumulator {
        let mut position_accumulator = PositionAccumulator::new();
        for (x, row) in atomic_position_accumulator.iter().enumerate() {
            for (y, val) in row.iter().enumerate() {
                position_accumulator[x][y] = val.load(atomic::Ordering::Relaxed);
            }
        }
        position_accumulator
    }
}

impl From<AtomicPositionAccumulator> for PositionAccumulator {
    fn from(atomic_position_accumulator: AtomicPositionAccumulator) -> PositionAccumulator {
        (&atomic_position_accumulator).into()
    }
}
