mod cell;
mod cell_set;

use anyhow::Result;
use cell::Cell;
use im::HashSet;
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

pub(crate) use cell_set::CellSet;

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
                    Cell::Possibilities(_) => None,
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
    /// get the cell at the indicated position
    fn cell(&self, CellPos { row, column }: CellPos) -> &Cell {
        // won't fail because Index must be between 0 and 9
        &self.0[row.into_inner()][column.into_inner()]
    }
    fn mut_cell(&mut self, CellPos { row, column }: CellPos) -> &mut Cell {
        &mut self.0[row.into_inner()][column.into_inner()]
    }
    /// iterator over all possible boards where one cell is made concrete
    ///
    /// for each possible cell, all possibilities are iterated over
    pub(crate) fn possible_updates(self) -> impl Iterator<Item = Self> {
        CellPos::all_cell_pos().flat_map(move |pos| pos.make_concrete_boards(self.clone()))
    }
}
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct CellPos {
    row: Index,
    column: Index,
}
impl CellPos {
    fn all_cell_pos() -> impl Iterator<Item = Self> {
        Index::indexes().flat_map(|row| Index::indexes().map(move |column| CellPos { row, column }))
    }
    fn make_concrete_boards(self, board: Board) -> impl Iterator<Item = Board> {
        match board.cell(self) {
            Cell::Concrete(_) => HashSet::new(),
            Cell::Possibilities(ref set) => set.clone(),
        }
        .into_iter()
        .map(move |num| {
            CellPos::all_cell_pos()
                .filter_map(|pos| {
                    let cell = if pos == self {
                        board.cell(pos).make_concrete_cell(num).ok()?
                    } else if pos.row == self.row || pos.column == self.column {
                        board.cell(pos).remove_possibility(num)
                    } else {
                        board.cell(pos).clone()
                    };
                    Some((pos, cell))
                })
                .collect()
        })
    }
}
