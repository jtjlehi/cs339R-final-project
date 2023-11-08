mod cell;

use std::collections::HashSet;

/// Represents the 9 by 9 board
///
/// the internal representation of the board is not determined for sure yet
struct Board;

impl Board {
    pub fn rows(&self) -> HashSet<Row> {
        todo!()
    }
    pub fn houses(&self) -> HashSet<House> {
        todo!()
    }
    pub fn columns(&self) -> HashSet<Column> {
        todo!()
    }
}

/// A row of a `Board`
///
/// each row must have one and only one instance of each number 1-9
struct Row;

/// A column of a `Board`
///
/// each row must have one and only one instance of each number 1-9
struct Column;

struct House;
