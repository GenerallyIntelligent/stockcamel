use crate::{board, constants};
use crossbeam::utils::CachePadded;
use std::ops::{Add, AddAssign, Deref, DerefMut};
use std::sync::{atomic, Arc};

#[derive(Clone, Copy)]
pub struct PositionAccumulator([[u64; constants::NUM_CAMELS]; constants::NUM_CAMELS]);

impl PositionAccumulator {
    pub fn new() -> Self {
        PositionAccumulator([[0; constants::NUM_CAMELS]; constants::NUM_CAMELS])
    }

    pub fn count_terminal(&self) -> u64 {
        let mut num_terminal = 0;
        for position_num in self[0] {
            num_terminal += position_num;
        }
        return num_terminal;
    }
}

impl Deref for PositionAccumulator {
    type Target = [[u64; constants::NUM_CAMELS]; constants::NUM_CAMELS];
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

impl AddAssign for PositionAccumulator {
    fn add_assign(&mut self, other: PositionAccumulator) {
        for (x, row) in other.iter().enumerate() {
            for (y, val) in row.iter().enumerate() {
                self[x][y] += *val;
            }
        }
    }
}

impl From<board::CamelOrder> for PositionAccumulator {
    fn from(camel_order: board::CamelOrder) -> Self {
        let mut new_position_accumulator = PositionAccumulator::new();
        for (position, camel_num) in camel_order.iter().enumerate() {
            new_position_accumulator[*camel_num][position] += 1;
        }
        new_position_accumulator
    }
}

pub struct AtomicPositionAccumulator(
    [[CachePadded<atomic::AtomicU64>; constants::NUM_CAMELS]; constants::NUM_CAMELS],
);

impl AtomicPositionAccumulator {
    pub fn new() -> Self {
        AtomicPositionAccumulator([
            [
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
            ],
            [
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
            ],
            [
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
            ],
            [
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
            ],
            [
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
                CachePadded::new(atomic::AtomicU64::new(0)),
            ],
        ])
    }

    pub fn count_terminal(&self) -> u64 {
        let mut num_terminal = 0;
        for position_num in &self[0] {
            num_terminal += position_num.load(atomic::Ordering::Relaxed);
        }
        return num_terminal;
    }

    pub fn add(&self, position_accumulator: PositionAccumulator) {
        for (x, vector) in position_accumulator.iter().enumerate() {
            for (y, val) in vector.iter().enumerate() {
                self[x][y].fetch_add(*val, atomic::Ordering::Relaxed);
            }
        }
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
    type Target = [[CachePadded<atomic::AtomicU64>; constants::NUM_CAMELS]; constants::NUM_CAMELS];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AtomicPositionAccumulator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
