use super::{
    cell::{Cell, CellVal, ToSet},
    Board, CellPos, Index,
};
use crate::UpdateError;
use anyhow::Result;
use im::HashSet;

type PossibleSet = HashSet<CellPos>;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ConcreteSet(HashSet<CellVal>);
impl ConcreteSet {
    fn insert(&mut self, val: CellVal) -> Result<(), UpdateError> {
        if self.0.contains(&val) {
            Err(UpdateError::InvalidConcrete)?;
        } else {
            self.0.insert(val);
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct UpdateSets<'b> {
    board: &'b mut Board,
    concrete_set: ConcreteSet,
    possible_set: PossibleSet,
}
impl<'b> UpdateSets<'b> {
    fn update(&mut self) -> Result<(), UpdateError> {
        let mut new_concretes = im::hashset![];
        // make possible changes
        for &pos in &self.possible_set {
            *self.board.mut_cell(pos) = self
                .board
                .cell(pos)
                .remove_possibilities(&self.concrete_set.0)?;
            // make concrete changes
            if let Some(val) = self.board.cell(pos).possible_is_concrete() {
                self.concrete_set.insert(val)?;
                new_concretes.insert(pos);
                *self.board.mut_cell(pos) = self.board.cell(pos).make_concrete_cell(val)?;
            }
        }
        self.possible_set = self.possible_set.clone().relative_complement(new_concretes);
        Ok(())
    }
    fn finished(&self) -> bool {
        self.possible_set
            .iter()
            .all(|&pos| match self.board.cell(pos) {
                Cell::Concrete(_) => true,
                Cell::Possibilities(set) => {
                    set.len() != 1
                        && set
                            .clone()
                            .intersection(self.concrete_set.0.clone())
                            .is_empty()
                }
            })
    }
}

#[derive(PartialEq, Eq, Debug)]
/// An unordered set of cells used for updating
pub(crate) struct CellSet<'b> {
    set: HashSet<CellPos>,
    board: &'b mut Board,
}

impl<'b> CellSet<'b> {
    /// checks that there are no duplicates or potential duplicates
    pub(crate) fn check_and_update(mut self) -> Result<(), UpdateError> {
        let mut update_sets = self.get_update_set()?;
        while !update_sets.finished() {
            update_sets.update()?;
        }
        Ok(())
    }
    /// gets the initial possible and concrete sets for the cell_set
    fn get_update_set(&mut self) -> Result<UpdateSets, UpdateError> {
        let mut concrete_set = ConcreteSet(HashSet::new());
        let mut possible_set = HashSet::new();
        for &pos in &self.set {
            match self.board.cell(pos) {
                &Cell::Concrete(val) => concrete_set.insert(val)?,
                Cell::Possibilities(_) => {
                    possible_set.insert(pos);
                }
            }
        }
        Ok(UpdateSets {
            board: self.board,
            possible_set,
            concrete_set,
        })
    }
}
impl Board {
    pub(crate) fn get_set<C: ToSet>(&mut self, index: Index) -> CellSet {
        CellSet {
            set: C::to_set(index),
            board: self,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::board::cell::macros::*;
    use crate::board::macros::*;
    use crate::board::{Column, House, Row};

    macro_rules! cell_set {
        (row($row:expr, $board:ident)) => {
            CellSet {
                set: (0..9).map(|i| pos!($row, i)).collect(),
                board: &mut $board,
            }
        };
        (column($column:expr, $board:ident)) => {
            CellSet {
                set: (0..9).map(|i| pos!(i, $column)).collect(),
                board: &mut $board,
            }
        };
        (house($board:ident)) => {
            CellSet {
                set: im::hashset![
                    pos!(0, 0),
                    pos!(0, 1),
                    pos!(0, 2),
                    pos!(1, 0),
                    pos!(1, 1),
                    pos!(1, 2),
                    pos!(2, 0),
                    pos!(2, 1),
                    pos!(2, 2)
                ],
                board: &mut $board,
            }
        };
    }
    macro_rules! concrete_set {
        [$( $val:expr ),*] => {
            ConcreteSet(im::hashset![$( cell_val!($val) ),*])
        };
    }

    #[test]
    fn get_cell_set_from_row_works() {
        let mut board = board!([]);
        let mut board_out = board!([]);

        assert_eq!(
            board.get_set::<Row>(index!(5)),
            cell_set!(row(5, board_out))
        );
    }
    #[test]
    fn get_cell_set_from_column_works() {
        let mut board = board!([]);
        let mut board_out = board!([]);

        assert_eq!(
            board.get_set::<Column>(index!(5)),
            cell_set!(column(5, board_out))
        );
    }
    #[test]
    fn get_cell_set_from_houses_works() {
        let mut board = board!([]);
        let mut board_out = board!([]);

        assert_eq!(
            board.get_set::<House>(index!(0)),
            cell_set!(house(board_out))
        );
    }

    #[test]
    fn concrete_insert_fails_if_exists() {
        assert_eq!(
            concrete_set![1, 2, 3, 4, 8, 9].insert(cell_val!(9)),
            Err(UpdateError::InvalidConcrete)
        );
    }
    #[test]
    fn concrete_insert_succeeds_if_not_exists() {
        assert_eq!(concrete_set![1, 2, 3, 4, 8, 9].insert(cell_val!(7)), Ok(()));
    }

    #[test]
    fn can_build_valid_update_set_with_all_possible() {
        let mut board = board!([]);
        let mut out_board = board!([]);
        let mut cell_set = cell_set!(row(1, board));
        let possible_set = cell_set.set.clone();

        assert_eq!(
            cell_set.get_update_set().unwrap(),
            UpdateSets {
                board: &mut out_board,
                possible_set,
                concrete_set: ConcreteSet(im::hashset![]),
            }
        )
    }
    #[test]
    fn can_build_valid_update_sets_with_concretes_no_make_concrete() {
        let mut board = board!([[3, 2, ?, {4, 5, 7}, ?, 7, 1, {4, 5}, {4, 5, 9}]]);
        let mut cell_set = cell_set!(row(0, board));

        let update_sets = cell_set.get_update_set().unwrap();

        assert_eq!(
            update_sets.possible_set,
            pos!(iter 0, {2, 3, 4, 7, 8}).collect(),
            "possible_set was incorrect"
        );

        assert_eq!(
            update_sets.concrete_set,
            concrete_set![3, 2, 7, 1],
            "concrete_set was incorrect"
        );
    }
    #[test]
    fn can_build_valid_update_sets_with_make_concrete() {
        let mut board = board!([[3, 2, ?, { 9, 7 }, ?, 7, 1, { 4 }, { 4, 5, 9 }]]);
        let mut cell_set = cell_set!(row(0, board));

        let update_sets = cell_set.get_update_set().unwrap();

        assert_eq!(
            update_sets.possible_set,
            pos!(iter 0, {2, 3, 4, 7, 8}).collect(),
            "possible_set was incorrect"
        );
        assert_eq!(
            update_sets.concrete_set,
            concrete_set![3, 2, 7, 1],
            "concrete_set was incorrect"
        );
    }
    #[test]
    fn cell_list_to_update_sets_errors_when_board_invalid() {
        let mut board = board!([[3, 2, ?, { 9, 7 }, ?, 3, 1, { 4 }, { 4, 5, 9 }]]);
        let mut cell_set = cell_set!(row(0, board));

        assert_eq!(cell_set.get_update_set(), Err(UpdateError::InvalidConcrete));
    }

    #[test]
    fn update_removes_possible() {
        let possible_set: HashSet<_> = pos!(iter 0, {2, 3, 4, 7, 8}).collect();
        let concrete_set = concrete_set![3, 2, 7, 1];
        let mut board = board!([[3, 2, ?, {4, 5, 7}, ?, 7, 1, {4, 5}, {4, 5, 9}]]);
        let mut updated = UpdateSets {
            board: &mut board,
            possible_set: possible_set.clone(),
            concrete_set: concrete_set.clone(),
        };
        updated.update().unwrap();

        assert_eq!(
            updated.board,
            &mut board!([[3, 2, { 4, 5, 6, 8, 9 }, { 4, 5 }, { 4, 5, 6, 8, 9 }, 7, 1, { 4, 5 }, { 4, 5, 9 }]])
        );
        assert_eq!(updated.possible_set, possible_set);
        assert_eq!(updated.concrete_set, concrete_set);
    }

    #[test]
    fn update_errors_when_overlapping_make_concrete() {
        let mut update_sets = UpdateSets {
            board: &mut board!([[1, 2, 3, { 4 }, { 4 }, 5, 6, 7, 8]]),
            possible_set: pos!(iter 0, { 3, 4  }).collect(),
            concrete_set: concrete_set![1, 2, 3, 5, 6, 7, 8],
        };
        assert_eq!(update_sets.update(), Err(UpdateError::Impossible));
    }
    #[test]
    fn update_errors_when_no_possibility_left() {
        let mut update_sets = UpdateSets {
            board: &mut board!([[1, 2, 3, 4, { 4, 5 }, 5, 6, 7, 8]]),
            possible_set: pos!(iter 0, 4).collect(),
            concrete_set: concrete_set![1, 2, 3, 4, 5, 6, 7, 8],
        };
        assert_eq!(update_sets.update(), Err(UpdateError::Impossible));
    }

    #[test]
    fn check_and_update_terminates_when_initial_board_is_finished() {
        let mut board = board!([[1, 2, 3, 4, 9, 5, 6, 7, 8]]);
        let out_board = board!([[1, 2, 3, 4, 9, 5, 6, 7, 8]]);
        let cell_set = cell_set!(row(0, board));
        cell_set.check_and_update().unwrap();

        assert_eq!(board, out_board);
    }
    #[test]
    fn check_and_update_terminates_with_initial_error() {
        let mut board = board!([[1, 2, 3, 4, 5, 5, 6, 7, 8]]);
        let cell_set = cell_set!(row(0, board));

        assert!(cell_set.check_and_update().is_err());
    }
    #[test]
    fn check_and_update_finds_errors() {
        let mut board = board!([[1, 2, 3, 4, { 4 }, 5, 6, 7, 8]]);
        let cell_set = cell_set!(row(0, board));

        assert!(cell_set.check_and_update().is_err());
    }
    #[test]
    fn check_and_update_finds_errors_2() {
        let mut board = board!([[1, 2, 3, 4, { 7, 4, 5 }, { 5, 7 }, { 6 }, { 6, 7 }, 8]]);
        let cell_set = cell_set!(row(0, board));

        assert!(cell_set.check_and_update().is_err());
    }
    #[test]
    fn check_and_update_terminates() {
        let mut board = board!([[
            { 1, 7 },
            { 1, 2 },
            { 1, 2, 3 },
            { 1, 2, 3, 4 },
            { 6, 4, 5 },
            ?,
            { 7, 8 },
            { 8, 9 },
            { 9 }
        ]]);
        let cell_set = cell_set!(row(0, board));
        cell_set.check_and_update().unwrap();

        assert_eq!(board, board!([[1, 2, 3, 4, { 5, 6 }, { 5, 6 }, 7, 8, 9]]));
    }
}
