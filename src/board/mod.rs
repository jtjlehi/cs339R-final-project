mod cell;

use anyhow::Result;
use cell::Cell;
use nutype::nutype;
use std::iter::repeat;
use thiserror::Error;

/// a newtype CellVall representing the value a cell can be (1-9)
#[nutype(
    validate(less_or_equal = 9, greater = 0),
    derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)
)]
struct Index(usize);
impl Index {
    pub fn indexes() -> impl Iterator<Item = Self> {
        (0..).map_while(|i| Self::new(i).ok())
    }
}

pub(crate) use cell::{CellList, CellVal};

#[derive(Error, Debug)]
enum BuildError {
    #[error("invalid number of rows")]
    RowCount,
    #[error("invalid number of cells in row {0}")]
    CellCount(usize),
}

/// Represents the 9 by 9 board
///
/// the internal representation of the board is not determined for sure yet
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Board([[Cell; 9]; 9]);

impl Default for Board {
    fn default() -> Self {
        let row: Vec<Cell> = repeat(Default::default()).take(9).collect();
        let board_vec: Vec<_> = repeat(row.try_into().unwrap()).take(9).collect();
        Board(board_vec.try_into().unwrap())
    }
}
impl From<Board> for [[Option<usize>; 9]; 9] {
    fn from(value: Board) -> Self {
        let mut arr: [[Option<usize>; 9]; 9] = Default::default();
        for (r, row) in value.0.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                arr[r][c] = match cell {
                    Cell::Concrete(cell_val) => Some(cell_val.into_inner()),
                    Cell::Possibities(_) => None,
                };
            }
        }
        arr
    }
}

impl Board {
    pub fn build(lines: Vec<Vec<Option<u8>>>) -> Result<Self> {
        let mut board: Board = Default::default();
        if lines.len() != 9 {
            Err(BuildError::RowCount)?
        }
        for (r, row) in lines.iter().enumerate() {
            if row.len() != 9 {
                Err(BuildError::CellCount(r))?
            }
            for (c, cell) in row.iter().enumerate() {
                board.0[r][c] = Cell::new(*cell)?;
            }
        }
        Ok(board)
    }
    /// get the cell at row, column
    ///
    /// used by `CellRef`s
    fn cell(&self, row: Index, column: Index) -> &Cell {
        // won't fail because Index must be between 0 and 9
        &self.0[row.into_inner()][column.into_inner()]
    }
    fn mut_cell(&mut self, row: Index, column: Index) -> &mut Cell {
        &mut self.0[row.into_inner()][column.into_inner()]
    }
}
