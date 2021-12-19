use std::fmt;
use crate::constants;

#[derive(Copy, Clone)]
pub struct CamelOdds {
    pub odds: [[f64; constants::NUM_CAMELS]; constants::NUM_CAMELS], //Odds of a camel getting a position, indexed by camel then position
}

#[derive(Copy, Clone)]
pub struct TileOdds {
    pub odds: [f64; constants::BOARD_SIZE],
}

impl fmt::Display for CamelOdds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: <5}", "Camel")?;
        for i in 1..self.odds.len() {
            write!(f, " | Pos {: <1}", i)?;
        }
        write!(f, "\n")?;
        for (camel_number, odds) in self.odds.iter().enumerate() {
            write!(f, "{: <5}", camel_number + 1)?;
            for (position, odd) in odds.iter().enumerate() {
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
        for i in 1..self.odds.len() {
            write!(f, " | {: <4}", i)?;
        }
        write!(f, "\n")?;
        write!(f, "{: <5}", "Odds")?;
        for (tile_number, odd) in self.odds.iter().enumerate() {
            write!(f, " | {:.2}", odd)?;
        }
        write!(f, "\n")?;
        Ok(())
    }
}