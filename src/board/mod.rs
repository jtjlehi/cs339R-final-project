mod cell;
mod cell_set;

use std::fmt;

use anyhow::Result;
use cell::Cell;
use im::HashSet;
use nutype::nutype;
use thiserror::Error;

/// a newtype CellVall representing the value a cell can be (1-9)
#[nutype(
    validate(less = 9),
    derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)
)]
pub(crate) struct Index(usize);
impl Index {
    pub fn indexes() -> impl Iterator<Item = Self> {
        (0..).map_while(|i| Self::new(i).ok())
    }
}

pub(crate) use cell::{Column, House, Row, ToSet};

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
        let board_vec: Vec<[Cell; 9]> = vec![vec![Default::default(); 9].try_into().unwrap(); 9];
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
    pub(crate) fn is_finished(&self) -> bool {
        println!("is it finished?");
        CellPos::all_cell_pos().all(|pos| match self.cell(pos) {
            Cell::Concrete(_) => true,
            Cell::Possibilities(set) => {
                println!("no we found some possibilities at {pos:?}, {set:?}");
                false
            }
        })
    }
}
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct CellPos {
    row: Index,
    column: Index,
}
impl fmt::Debug for CellPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CellPos")
            .field(&self.row.into_inner())
            .field(&self.column.into_inner())
            .finish()
    }
}
impl CellPos {
    fn all_cell_pos() -> impl Iterator<Item = Self> {
        Index::indexes().flat_map(|row| Index::indexes().map(move |column| CellPos { row, column }))
    }
    fn make_concrete_boards(self, board: Board) -> impl Iterator<Item = Board> {
        match board.cell(self) {
            Cell::Concrete(_) => HashSet::new(),
            Cell::Possibilities(set) => set.clone(),
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

#[cfg(test)]
mod macros {
    use super::Board;

    macro_rules! board {
        ($rows:tt) => (crate::board::macros::make_board(board!(init $rows)));
        (init [ $($row:tt)* ]) => (vec![$( board_row!($row) ),*]);
    }
    macro_rules! board_row {
        ([$($cell:tt),*]) => (vec![$( board_cell!($cell) ),*]);
    }
    macro_rules! board_cell {
        (?)=> (crate::board::cell::macros::cell!(? 1, 2, 3, 4, 5, 6, 7, 8, 9));
        ({ $($possibility:expr),* }) => (crate::board::cell::macros::cell!(? $( $possibility ),*));
        ($concrete:expr) => (crate::board::cell::macros::cell!( $concrete ));
    }

    macro_rules! pos {
        ($row:expr, $column:expr) => {
            CellPos {
                row: crate::board::cell::macros::index!($row),
                column: crate::board::cell::macros::index!($column),
            }
        };
        (iter $row:expr, { $( $column:expr ),* }) => {
            [$(pos!($row, $column)),*].into_iter()
        };
        (iter $row:expr, $column:expr) => (std::iter::once(pos!($row, $column)));
        () => {
            crate::board::macros::pos!(1, 2)
        };
    }
    pub(super) use {board, board_cell, board_row, pos};

    pub(super) fn make_board(b: Vec<Vec<super::Cell>>) -> Board {
        let mut final_board: Board = Default::default();

        for r in 0..9 {
            if let Some(row) = b.get(r) {
                for c in 0..9 {
                    if let Some(cell) = row.get(c) {
                        final_board.0[r][c] = cell.clone();
                    }
                }
            }
        }
        final_board
    }
}
