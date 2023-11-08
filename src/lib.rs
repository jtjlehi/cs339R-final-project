mod cell;

use cell::{Cell, CellList, UpdateError};
use std::{collections::HashSet, hash::Hash, ops::ControlFlow};

use crate::cell::CellRef;

/// Represents the 9 by 9 board
///
/// the internal representation of the board is not determined for sure yet
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Board([[Cell; 9]; 9]);

impl Board {
    fn rows(&self) -> HashSet<Row> {
        (0..9)
            .map(|row_index| Row {
                row_index,
                board: self,
            })
            .collect()
    }
    fn houses(&self) -> HashSet<House> {
        todo!()
    }
    fn columns(&self) -> HashSet<Column> {
        todo!()
    }
}

/// A row of a `Board`
///
/// each row must have one and only one instance of each number 1-9
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Row<'b> {
    row_index: usize,
    board: &'b Board,
}
impl Hash for Row<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.row_index.hash(state);
    }
}

impl<'b> CellList for Row<'b> {
    fn cell_at(&self, index: usize) -> CellRef {
        CellRef {
            row: self.row_index,
            column: index,
            board: self.board,
        }
    }
}

/// A column of a `Board`
///
/// each row must have one and only one instance of each number 1-9
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
struct Column;

impl CellList for Column {
    fn cell_at(&self, index: usize) -> CellRef {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
struct House;

impl CellList for House {
    fn cell_at(&self, index: usize) -> CellRef {
        todo!()
    }
}

#[derive(Clone, Copy)]
struct State<'a>(Option<UpdateError>, &'a Board);
#[inline]
fn try_cell<'a>(num: usize) -> impl Fn(State<'a>, CellRef) -> ControlFlow<Board, State<'a>> {
    #[inline]
    move |state, cell| {
        cell.to_concrete(num)
            // break if solve was successful
            .and_then(solve)
            .map(ControlFlow::Break)
            // if to_concrete or solve failed, convert to state, using previous board
            .map_err(|why| State(Some(why), state.1))
            .unwrap_or_else(ControlFlow::Continue)
    }
}
pub fn solve(board: Board) -> Result<Board, UpdateError> {
    // board stays the same through the entire iteration
    let out = (1..9).try_fold(State(None, &board), |state, num| {
        // get rows from the board
        (state.1.rows().iter())
            // filter out the rows that have a concrete version of `num`,
            // and get only cells that can be `num`
            .filter_map(|row: &Row| row.possible_cells_of_num(num))
            .try_fold(state, |state, row| {
                // try the cell to see
                row.into_iter().try_fold(state, try_cell(num))
            })
    });
    match out {
        ControlFlow::Break(board) => Ok(board),
        // if we get to the end of the iteration, it means we didn't change board
        // this can be proven by the fact that inside state, board is a shared reference
        // because we don't change it, we can return the original board
        ControlFlow::Continue(State(None, _)) => Ok(board),
        ControlFlow::Continue(State(Some(why), _)) => Err(why),
    }
}
