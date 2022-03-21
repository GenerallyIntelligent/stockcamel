use crate::constants;
use crate::probabilities::accumulators::{PositionAccumulator, TileAccumulator};
use std::fmt;

use super::accumulators::AtomicPositionAccumulator;

#[derive(Copy, Clone)]
pub struct CamelOdds {
    pub odds: [[f64; constants::NUM_CAMELS]; constants::NUM_CAMELS], //Odds of a camel getting a position, indexed by camel then position
}

#[derive(Copy, Clone)]
pub struct TileOdds {
    pub odds: [f64; constants::BOARD_SIZE],
}

impl CamelOdds {
    pub fn new(accumulator: &PositionAccumulator, num_terminal: &u64) -> Self {
        let mut position_odds = [[0.0; constants::NUM_CAMELS]; constants::NUM_CAMELS];
        let num_terminal = num_terminal.clone();
        for (x, vector) in accumulator.iter().enumerate() {
            for (y, sum) in vector.iter().enumerate() {
                position_odds[x][y] = *sum as f64 / num_terminal as f64;
            }
        }
        CamelOdds {
            odds: position_odds,
        }
    }
}

impl TileOdds {
    pub fn new(accumulator: &TileAccumulator, num_terminal: &u64) -> Self {
        let mut tile_odds = [0.0; constants::BOARD_SIZE];
        for (idx, sum) in accumulator.iter().enumerate() {
            tile_odds[idx] = *sum as f64 / num_terminal.clone() as f64;
        }
        TileOdds { odds: tile_odds }
    }
}

impl From<&AtomicPositionAccumulator> for CamelOdds {
    fn from(accumulator: &AtomicPositionAccumulator) -> Self {
        let acumulator: PositionAccumulator = accumulator.into();
        acumulator.into()
    }
}

impl From<AtomicPositionAccumulator> for CamelOdds {
    fn from(accumulator: AtomicPositionAccumulator) -> Self {
        let acumulator: PositionAccumulator = accumulator.into();
        acumulator.into()
    }
}

impl From<PositionAccumulator> for CamelOdds {
    fn from(accumulator: PositionAccumulator) -> Self {
        let num_terminal = accumulator.count_terminal();
        CamelOdds::new(&accumulator, &num_terminal)
    }
}

impl From<&PositionAccumulator> for CamelOdds {
    fn from(accumulator: &PositionAccumulator) -> Self {
        let num_terminal = accumulator.count_terminal();
        CamelOdds::new(accumulator, &num_terminal)
    }
}

impl fmt::Display for CamelOdds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: <5}", "Camel")?;
        for i in 1..self.odds.len() + 1 {
            write!(f, " | Pos {: <1}", i)?;
        }
        write!(f, "\n")?;
        for (camel_number, odds) in self.odds.iter().enumerate() {
            write!(f, "{: <5}", camel_number + 1)?;
            for odd in odds.iter() {
                write!(f, " | {:.3}", odd)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl fmt::Display for TileOdds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: <5}", "Tile")?;
        for i in 1..self.odds.len() + 1 {
            write!(f, " | {: <4}", i)?;
        }
        write!(f, "\n")?;
        write!(f, "{: <5}", "Odds")?;
        for odd in self.odds.iter() {
            write!(f, " | {:.2}", odd)?;
        }
        write!(f, "\n")?;
        Ok(())
    }
}
