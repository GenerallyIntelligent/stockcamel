use crate::board;
use crate::probabilities::accumulators::*;
use std::collections::HashMap;

pub type RoundTranspositionTable =
    ProbabilitiesTranspositionTable<(PositionAccumulator, PositionAccumulator, TileAccumulator)>;
pub type GameTranspositionTable = ProbabilitiesTranspositionTable<PositionAccumulator>;

pub struct ProbabilitiesTranspositionTable<T> {
    map: HashMap<u32, (T, u8)>,
    modulo: u32,
}

impl<T> ProbabilitiesTranspositionTable<T> {
    pub fn new(capacity: u32) -> Self {
        ProbabilitiesTranspositionTable {
            map: HashMap::new(),
            modulo: capacity,
        }
    }

    pub fn check(&self, board: &board::Board) -> Option<T> {
        None
    }

    pub fn update(&self, board: &board::Board, depth: u8, store_value: T) {}
}

fn hash_board(board: board::Board) -> u128 {
    todo!();
}
