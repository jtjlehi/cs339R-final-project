mod cell;

use cell::{CellList, CellSet, UpdateError};
use std::{collections::HashSet, ops::ControlFlow};

use crate::cell::Cell;

/// Represents the 9 by 9 board
///
/// the internal representation of the board is not determined for sure yet
pub struct Board;

impl Board {
    fn rows(&self) -> HashSet<Row> {
        todo!()
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
struct Row;

impl CellList for Row {
    fn cell_at(&self, index: usize) -> &cell::Cell {
        todo!()
    }
}

/// A column of a `Board`
///
/// each row must have one and only one instance of each number 1-9
struct Column;

impl CellList for Column {
    fn cell_at(&self, index: usize) -> &cell::Cell {
        todo!()
    }
}

struct House;

impl CellList for House {
    fn cell_at(&self, index: usize) -> &cell::Cell {
        todo!()
    }
}

type State<'a> = (Option<UpdateError>, Box<Board>);
pub fn solve(board: Box<Board>) -> Result<Box<Board>, UpdateError> {
    #[inline]
    fn try_cell<'a>(num: usize) -> impl Fn(State<'a>, Cell) -> ControlFlow<Box<Board>, State<'a>> {
        move |state, cell| {
            cell.to_concrete(&state.1, num)
                // break if solve was successful
                .and_then(|board| solve(Box::new(board)))
                .map(ControlFlow::Break)
                // if to_concrete or solve failed, convert to state, using previous board
                .map_err(|why| (Some(why), state.1))
                .unwrap_or_else(ControlFlow::Continue)
        }
    }

    // board stays the same through the entire iteration
    let out = (1..9).try_fold((None, board), |state, num| {
        // get rows from the board
        (state.1.rows().into_iter())
            // filter out the rows that have a concrete version of `num`,
            // and get only cells that can be `num`
            .filter_map(|row: Row| row.possible_cells_of_num(num))
            .try_fold(state, |state, row| {
                // try the cell to see
                row.into_iter().try_fold(state, try_cell(num))
            })
    });
    match out {
        ControlFlow::Break(board) => Ok(board),
        ControlFlow::Continue((None, board)) => Ok(board),
        ControlFlow::Continue((Some(why), _)) => Err(why),
    }
}
