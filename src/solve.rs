use crate::{board::CellSet, Board, UpdateError};
use std::{iter::successors, ops::ControlFlow};

type ControlSolution = ControlFlow<Board, Result<Board, UpdateError>>;

impl Board {
    /// Attempt to solve the given board
    ///
    /// we don't mutate the Board so we don't have to implement our own stack for backtracking
    pub fn solve(self) -> BoardState {
        match self.validate() {
            BoardState::Valid(board) => board.possible_updates().try_board_until(Self::solve),
            possible_board => possible_board,
        }
    }
    /// verifies that all of the rows, columns, and houses are valid
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
    pub(crate) fn validate(self) -> BoardState {
        use BoardState::PartiallyValid;
        let init = Some(PartiallyValid(self.clone()));
        // loop through until it becomes valid, finished, or an error
        successors(init, |board| match board {
            PartiallyValid(board) => Some(board.validate_helper()),
            board_state => Some(board_state.clone()),
        })
        .try_board_until(|board_state| board_state)
    }
    /// single pass of validation marking if any changes were made along the way
    fn validate_helper(&self) -> BoardState {
        self.rows()
            .try_board_until(|row| self.validate_cell_set(row))
            .and_then(|board| {
                board
                    .columns()
                    .try_board_until(|column| board.validate_cell_set(column))
            })
            .and_then(|board| {
                board
                    .houses()
                    .try_board_until(|houses| board.validate_cell_set(houses))
            })
    }
    /// Validate that for the cell list:
    ///
    /// there can only be one concrete instance of each cell value 1-9
    /// cell_list
    /// for each value:
    ///  - if it can only exist in one cell, that cell has that concrete value
    ///  - it must be able to exist
    /// for each cell
    ///  - if it can only have one value, it has that value
    ///  - it must be able to exist
    fn validate_cell_set<'b, C>(&'b self, cell_list: C) -> BoardState
    where
        CellSet<'b>: From<(C, &'b Board)>,
    {
        let set = CellSet::from((cell_list, self));
        let _ = set.values_can_exist();
        todo!()
    }
}

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
    /// if the board is finished or valid, return it
    ///
    /// if not: don't
    fn ok(&self) -> Option<Board> {
        match self {
            Self::Finished(board) | Self::Valid(board) => Some(board.clone()),
            Self::Err(_) | Self::PartiallyValid(_) => None,
        }
    }
    /// update the board using the provided function
    fn and_then(&self, f: impl FnOnce(&Board) -> Self) -> Self {
        match self {
            BoardState::Valid(board) | BoardState::PartiallyValid(board) => f(board),
            // for errors and finished, pass it on
            state => state.clone(),
        }
    }
}

/// an Iterator method to express the concept of continually trying the method with each element
/// until a good value is returned
trait TryUntil: Iterator
where
    Self: Sized,
{
    /// tries each board in the iterator until one is finished or the end is reached
    ///
    /// returns the default if none of them work
    #[inline]
    fn try_board_until(&mut self, f: impl Fn(Self::Item) -> BoardState) -> BoardState {
        let init = Err(UpdateError::InitError);
        self.try_fold(init, |_, x| -> ControlSolution { f(x).into() })
            .into()
    }
}

impl<T, I: Iterator<Item = T>> TryUntil for I {}
