use crate::{
    board::{self, Column, House, Index, Row},
    Board, UpdateError,
};
use std::ops::ControlFlow;

type ControlSolution = ControlFlow<Board, Result<Board, UpdateError>>;

impl Board {
    /// Attempt to solve the given board
    ///
    /// we recur so we don't have to implement our own stack for backtracking
    pub fn solve(self) -> Result<Board, UpdateError> {
        println!("solve");
        match self.clone().validate() {
            BoardState::Valid(board) | BoardState::PartiallyValid(board) => {
                println!("valid board");
                let mut err = Err(UpdateError::InitError);
                for board in board.possible_updates() {
                    println!("possible_updates");
                    match board.solve() {
                        Ok(board) => return Ok(board),
                        error => err = error,
                    };
                }
                err
            }
            BoardState::Finished(board) => Ok(board),
            BoardState::Err(err) => Err(err),
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
        let mut init = BoardState::Valid(self);

        loop {
            let board = init
                .validate_cell_lists::<Row>()
                .validate_cell_lists::<House>()
                .validate_cell_lists::<Column>();
            break match board {
                board @ (BoardState::Finished(_) | BoardState::Err(_)) => board,
                BoardState::Valid(board) | BoardState::PartiallyValid(board)
                    if board.is_finished() =>
                {
                    BoardState::Finished(board)
                }
                board @ BoardState::Valid(_) => board,
                board => {
                    init = board;
                    continue;
                }
            };
        }
    }
}

#[derive(Clone)]
pub enum BoardState {
    Finished(Board),
    Valid(Board),
    PartiallyValid(Board),
    Err(UpdateError),
}
impl BoardState {
    fn validate_cell_lists<C: board::ToSet>(&mut self) -> BoardState {
        let validate = |board: &mut Board| {
            Index::indexes().try_for_each(|i| board.get_set::<C>(i).check_and_update())
        };
        match self {
            board @ (Self::Finished(_) | Self::Err(_)) => board.clone(),
            Self::Valid(board) => {
                let old = board.clone();
                let out = validate(board);
                let new = board.clone();
                match out {
                    Ok(()) if new == old => Self::Valid(new),
                    Ok(()) => Self::PartiallyValid(new),
                    Err(err) => Self::Err(err),
                }
            }
            Self::PartiallyValid(board) => match validate(board) {
                Ok(()) => Self::PartiallyValid(board.clone()),
                Err(err) => Self::Err(err),
            },
        }
    }
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
