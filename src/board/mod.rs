mod cell;

use crate::new_types::{CellVal, Index};
use cell::Cell;
pub(crate) use cell::CellList;
use std::iter::repeat;

/// Represents the 9 by 9 board
///
/// the internal representation of the board is not determined for sure yet
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Board([[Cell; 9]; 9]);
impl From<Board> for [[Option<usize>; 9]; 9] {
    fn from(value: Board) -> Self {
        let mut arr: [[Option<usize>; 9]; 9] = Default::default();
        for (r, row) in value.0.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                arr[r][c] = match cell {
                    Cell::Concrete(cell_val) => Some(cell_val.inner()),
                    Cell::Possibities(_) => None,
                };
            }
        }
        arr
    }
}

impl Default for Board {
    fn default() -> Self {
        let cell: Cell = Default::default();
        let row: Vec<_> = repeat(cell).take(9).collect();
        let board_vec: Vec<_> = repeat(row.try_into().unwrap()).take(9).collect();
        Board(board_vec.try_into().unwrap())
    }
}

impl Board {
    pub fn build(lines: Vec<Vec<Option<u8>>>) -> Result<Self, String> {
        let mut board: Board = Default::default();
        if lines.len() != 9 {
            Err("invalid number of rows")?
        }
        for (r, row) in lines.iter().enumerate() {
            if row.len() != 9 {
                Err(format!("invalid number of cells in row {r}"))?
            }
            for (c, cell) in row.iter().enumerate() {
                board.0[r][c] = match cell {
                    None => Default::default(),
                    Some(i) => Cell::Concrete(
                        CellVal::build(*i as usize).map_err(|_| "invalid cell value")?,
                    ),
                };
            }
        }
        Ok(board)
    }
    pub(crate) fn cell(&self, row: Index, column: Index) -> &Cell {
        // won't fail because Index must be between 0 and 9
        &self.0[row.inner()][column.inner()]
    }
    fn mut_cell(&mut self, row: Index, column: Index) -> &mut Cell {
        &mut self.0[row.inner()][column.inner()]
    }
    fn update_cell(&self, row: Index, column: Index, val: Cell) -> Self {
        // the biggest cost of cloning is the hashset for possible cells
        // I'm using im to reduce this cost
        let mut new_board = self.clone();
        new_board.0[row.inner()][column.inner()] = val;
        new_board
    }
}
