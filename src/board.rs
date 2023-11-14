use crate::cell::{Cell, CellList, CellVal, UpdateError};
use im::HashSet;
use std::{
    hash::Hash,
    iter::{once, repeat, successors},
    ops::ControlFlow,
};

use crate::cell::CellRef;

/// An Index of a board/row/column
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub(crate) struct Index(usize);
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub(crate) struct InvalidIndex;
impl Index {
    pub(crate) fn build(i: usize) -> Result<Self, InvalidIndex> {
        if i >= 9 {
            Err(InvalidIndex)
        } else {
            Ok(Self(i))
        }
    }
    pub(crate) fn indexes() -> impl Iterator<Item = Self> {
        (0..).map_while(|i| Self::build(i).ok())
    }
}
/// Represents the 9 by 9 board
///
/// the internal representation of the board is not determined for sure yet
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Board([[Cell; 9]; 9]);

impl Default for Board {
    fn default() -> Self {
        let cell = Default::default();
        let row: Vec<_> = repeat(cell).take(9).collect();
        let board_vec: Vec<_> = repeat(row.try_into().unwrap()).take(9).collect();
        Board(board_vec.try_into().unwrap())
    }
}

type Solution = Result<Board, UpdateError>;

type ControlSolution = ControlFlow<Board, Result<Board, UpdateError>>;

#[derive(Clone)]
pub enum BoardState {
    Finished(Board),
    Valid(Board),
    PartiallyValid(Board),
    Err(UpdateError),
}

impl From<BoardState> for ControlSolution {
    fn from(value: BoardState) -> Self {
        match value {
            BoardState::Finished(board) => ControlFlow::Break(board),
            BoardState::Valid(board) => ControlFlow::Break(board),
            BoardState::PartiallyValid(board) => ControlFlow::Continue(Ok(board)),
            BoardState::Err(why) => ControlFlow::Continue(Err(why)),
        }
    }
}
impl From<ControlSolution> for BoardState {
    fn from(value: ControlSolution) -> Self {
        match value {
            ControlFlow::Break(board) => BoardState::Finished(board),
            ControlFlow::Continue(Ok(board)) => BoardState::Valid(board),
            ControlFlow::Continue(Err(why)) => BoardState::Err(why),
        }
    }
}
impl BoardState {
    fn ok(&self) -> Option<Board> {
        match self {
            Self::Finished(board) | Self::Valid(board) => Some(board.clone()),
            Self::Err(_) | Self::PartiallyValid(_) => None,
        }
    }
}

trait TryUntil: Iterator
where
    Self: Sized,
{
    /// tries each board in the iterator until one is finished or the end is reached
    ///
    /// returns the default if none of them work
    #[inline]
    fn try_board_until(&mut self, f: impl Fn(&Self::Item) -> BoardState) -> BoardState {
        let init = Err(UpdateError::InitError);
        self.try_fold(init, |_, x| -> ControlSolution { f(&x).into() })
            .into()
    }
}

impl<T, I: Iterator<Item = T>> TryUntil for I {}

impl Board {
    pub(crate) fn cell(&self, row: Index, column: Index) -> &Cell {
        // won't fail because Index must be between 0 and 9
        &self.0[row.0][column.0]
    }
    fn mut_cell(&mut self, row: Index, column: Index) -> &mut Cell {
        &mut self.0[row.0][column.0]
    }
    fn update_cell(&self, row: Index, column: Index, val: Cell) -> Self {
        // the biggest cost of cloning is the hashset for possible cells
        // I'm using im to reduce this cost
        let mut new_board = self.clone();
        new_board.0[row.0][column.0] = val;
        new_board
    }
    /// Attempt to solve the given board
    ///
    /// we don't mutate the Board so we don't have to implement our own stack for backtracking
    pub fn solve(&self) -> BoardState {
        // make sure the board is valid before starting
        // if it is, we return early
        let valid_board = match self.validate() {
            BoardState::Valid(board) => board,
            // if there is an error or it is finished, return
            // this way errors are filtered out and such
            possible_board => return possible_board,
        };
        // temp variable created to satisfy the borrow checker
        let mut possible = CellVal::cell_vals().flat_map(|num| {
            valid_board
                .rows()
                .filter_map(move |row| row.possible_cells_of_num(num))
                // we are testing each cell as value -> the row doesn't matter -> flattening is OK
                .flat_map(|cell_set| cell_set.into_iter())
                // if making concrete fails, don't use the board
                .filter_map(move |cell| cell.make_concrete(num).ok())
        });
        possible.try_board_until(Self::solve)
    }
    /// verifies that all of the rows and columns and houses are valid
    /// ## Rules
    ///
    /// - for each row, column, and house:
    ///   - there can only be one concrete instance of each cell value 1-9
    ///   - for each value:
    ///     - if it can only exist in one cell, that cell has that concrete value
    ///     - it must be able to exist
    ///   - for each cell
    ///     - if it can only have one value, it has that value
    ///     - it must be able to exist
    pub(crate) fn validate(&self) -> BoardState {
        use BoardState::PartiallyValid;
        // loop through until it becomes valid, finished, or an error
        successors(Some(PartiallyValid(self.clone())), |board| match board {
            PartiallyValid(board) => Some(board.validate_helper()),
            board_state => Some(board_state.clone()),
        })
        .try_board_until(|board_state| board_state.clone())
    }
    /// single pass of validation marking if any changes were made along the way
    fn validate_helper(&self) -> BoardState {
        todo!()
    }
    pub(crate) fn row(&self, row_index: Index) -> Row {
        Row {
            row_index,
            board: self,
        }
    }
    pub(crate) fn rows(&self) -> impl Iterator<Item = Row> {
        Index::indexes().map(|row_index| self.row(row_index))
    }
    pub(crate) fn houses(&self) -> HashSet<House> {
        todo!()
    }
    pub(crate) fn columns(&self) -> HashSet<Column> {
        todo!()
    }
}
impl<'b> IntoIterator for &'b Board {
    type Item = CellRef<'b>;

    type IntoIter = <Vec<CellRef<'b>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        Index::indexes()
            .flat_map(|row| {
                Index::indexes().map(move |column| CellRef {
                    row,
                    column,
                    board: self,
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}
impl<'b> FromIterator<(CellRef<'b>, Cell)> for Board {
    fn from_iter<T: IntoIterator<Item = (CellRef<'b>, Cell)>>(iter: T) -> Self {
        let mut board: Board = Default::default();
        for (CellRef { row, column, .. }, cell) in iter {
            *board.mut_cell(row, column) = cell;
        }
        board
    }
}

/// A row of a `Board`
///
/// each row must have one and only one instance of each number 1-9
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct Row<'b> {
    row_index: Index,
    board: &'b Board,
}

impl Hash for Row<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.row_index.hash(state);
    }
}

impl<'b> CellList for Row<'b> {
    fn cell_at(&self, index: Index) -> CellRef {
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
pub struct Column;

impl CellList for Column {
    fn cell_at(&self, index: Index) -> CellRef {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct House;

impl CellList for House {
    fn cell_at(&self, index: Index) -> CellRef {
        todo!()
    }
}

#[derive(Clone)]
struct State(Option<UpdateError>, Board);
