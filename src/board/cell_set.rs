use super::{
    cell::{Cell, CellAt, CellVal},
    Board, CellPos, Index,
};
use crate::UpdateError;
use anyhow::Result;
use im::HashSet;
use std::iter::successors;

type UpdateCells = HashSet<(CellPos, CellVal)>;
type PossibleSet = HashSet<CellPos>;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ConcreteSet(HashSet<CellVal>);
impl ConcreteSet {
    fn insert(&mut self, val: CellVal) -> Result<(), UpdateError> {
        if self.0.insert(val).is_some() {
            Err(UpdateError::InvalidConcrete)?
        };
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct UpdateSets {
    board: Board,
    remove_possible_set: UpdateCells,
    make_concrete_set: UpdateCells,
    concrete_set: ConcreteSet,
    possible_set: PossibleSet,
}
impl UpdateSets {
    /// gets the initial possible and concrete sets for the cell_set
    fn get_initial(
        cell_set: CellSet,
        board: Board,
    ) -> Result<(ConcreteSet, PossibleSet), UpdateError> {
        let (concretes, positions): (Vec<_>, HashSet<_>) = cell_set
            .set
            .clone()
            .into_iter()
            .filter_map(|pos| match board.cell(pos) {
                &Cell::Concrete(val) => Some((val, pos)),
                Cell::Possibilities(_) => None,
            })
            .unzip();

        let mut concrete_set = ConcreteSet(HashSet::new());
        for concrete in concretes {
            concrete_set.insert(concrete)?
        }
        let possible_set = cell_set.set.clone().difference(positions.clone());
        Ok((concrete_set, possible_set))
    }

    fn get_make_concrete_set(possible_set: &PossibleSet, board: &Board) -> UpdateCells {
        possible_set
            .iter()
            .filter_map(|&pos| board.cell(pos).possible_is_concrete(pos))
            .collect()
    }
    /// gets the set of a all possible value removals that need to be done
    ///
    /// used to initialize the structure
    fn get_remove_possible_set(
        concrete_set: &ConcreteSet,
        possible_set: &PossibleSet,
        board: &Board,
    ) -> UpdateCells {
        concrete_set
            .0
            .iter()
            .flat_map(|val| {
                possible_set
                    .iter()
                    .filter_map(move |&pos| match board.cell(pos) {
                        Cell::Possibilities(values) if values.contains(val) => Some((pos, *val)),
                        _ => None,
                    })
            })
            .collect()
    }

    fn update(&self) -> Result<Self, UpdateError> {
        let mut make_concrete_set = self.make_concrete_set.clone();
        let mut board = self.board.clone();
        // make possible changes
        for &(pos, val) in &self.remove_possible_set {
            *board.mut_cell(pos) = board.cell(pos).remove_possibility(val);
            if let Some(update) = board.cell(pos).possible_is_concrete(pos) {
                make_concrete_set.insert(update);
            }
        }
        // make concrete changes
        let mut concrete_set = self.concrete_set.clone();
        let mut possible_set = self.possible_set.clone();
        for &(pos, val) in &make_concrete_set {
            concrete_set.insert(val)?;
            possible_set.remove(&pos);
            *board.mut_cell(pos) = board.cell(pos).make_concrete_cell(val)?;
        }
        Ok(Self {
            // there aren't any more HashSet changes to make
            make_concrete_set: HashSet::new(),
            remove_possible_set: Self::get_remove_possible_set(
                &concrete_set,
                &possible_set,
                &board,
            ),
            board,
            concrete_set,
            possible_set,
        })
    }
    fn finished(&self) -> bool {
        self.remove_possible_set.is_empty() && self.make_concrete_set.is_empty()
    }
}
fn update(cells: &Result<UpdateSets, UpdateError>) -> Option<Result<UpdateSets, UpdateError>> {
    match cells {
        Ok(cells) => Some(cells.update()),
        Err(errs) => Some(Err(*errs)),
    }
}
fn not_finished(cells: &Result<UpdateSets, UpdateError>) -> bool {
    match cells {
        Ok(cells) => !cells.finished(),
        Err(_) => false,
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
/// An unordered set of cells used for updating
pub(crate) struct CellSet<'b> {
    set: HashSet<CellPos>,
    board: &'b Board,
}
impl<'b> TryFrom<CellSet<'b>> for UpdateSets {
    type Error = UpdateError;

    fn try_from(cell_set: CellSet<'b>) -> Result<Self, Self::Error> {
        let board = cell_set.board.clone();
        let (concrete_set, possible_set) = Self::get_initial(cell_set, board.clone())?;

        Ok(UpdateSets {
            remove_possible_set: Self::get_remove_possible_set(
                &concrete_set,
                &possible_set,
                &board,
            ),
            make_concrete_set: Self::get_make_concrete_set(&possible_set, &board),
            board,
            possible_set,
            concrete_set,
        })
    }
}
impl<'b> CellSet<'b> {
    /// checks that there are no duplicates or potential duplicates
    pub(crate) fn values_can_exist(self) -> Result<Board> {
        let update_sets: UpdateSets = self.try_into()?;
        let set = successors(Some(Ok(update_sets)), update)
            .take_while(not_finished)
            .last()
            .unwrap()?;
        Ok(set.board)
    }
}
impl<'b, T: CellAt> From<(T, &'b Board)> for CellSet<'b> {
    fn from(value: (T, &'b Board)) -> Self {
        Self {
            set: Index::indexes().map(|i| value.0.cell_at(i)).collect(),
            board: value.1,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::board::cell::macros::*;
    use crate::board::macros::*;

    macro_rules! cell_set {
        (row($row:expr, $board:ident)) => {
            CellSet {
                set: (0..9).map(|i| pos!($row, i)).collect(),
                board: &$board,
            }
        };
        (column($column:expr, $board:ident)) => {
            CellSet {
                set: (0..9).map(|i| pos!(i, $column)).collect(),
                board: &$board,
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
                board: &$board,
            }
        };
    }
    macro_rules! update_cells {
        ($row:expr => {$(
            $columns:tt => $cell_values:tt
        ),*}) => {
            im::HashSet::new()
                $(.union(
                    im::HashSet::<(CellPos, CellVal)>::unions(
                        pos!(iter $row, $columns).map(|pos| update_cells!(pos, $cell_values))
                    )
                ))*
        };
        ($pos:expr, { $( $cell_val:expr ),* }) => {
            im::hashset![$( ($pos, cell_val!($cell_val)) ),*]
        };
        ($pos:expr, $cell_val:expr) => {
            im::hashset![($pos, cell_val!($cell_val))]
        };
    }
    macro_rules! concrete_set {
        [$( $val:expr ),*] => {
            ConcreteSet(im::hashset![$( cell_val!($val) ),*])
        };
    }

    #[test]
    fn get_cell_set_from_row_works() {
        let board = board!([]);

        let row = board.row(index!(5));
        assert_eq!(CellSet::from((row, &board)), cell_set!(row(5, board)));
    }
    #[test]
    fn get_cell_set_from_column_works() {
        let board: Board = board!([]);

        let column = board.column(index!(5));
        assert_eq!(CellSet::from((column, &board)), cell_set!(column(5, board)));
    }
    #[test]
    fn get_cell_set_from_houses_works() {
        let board: Board = board!([]);

        let house = board.house(index!(0));
        assert_eq!(CellSet::from((house, &board)), cell_set!(house(board)));
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
        let board = board!([]);
        let cell_set = cell_set!(row(1, board));
        let possible_set = cell_set.set.clone();

        assert_eq!(
            UpdateSets::try_from(cell_set).unwrap(),
            UpdateSets {
                board,
                remove_possible_set: im::hashset![],
                make_concrete_set: im::hashset![],
                possible_set,
                concrete_set: ConcreteSet(im::hashset![]),
            }
        )
    }
    #[test]
    fn can_build_valid_update_sets_with_concretes_no_make_concrete() {
        let board = board!([[3, 2, ?, {4, 5, 7}, ?, 7, 1, {4, 5}, {4, 5, 9}]]);
        let cell_set = cell_set!(row(0, board));

        let update_sets = UpdateSets::try_from(cell_set).unwrap();

        assert_eq!(
            update_sets.possible_set,
            pos!(iter 0, {2, 3, 4, 7, 8}).collect(),
            "possible_set was incorrect"
        );
        assert_eq!(
            update_sets.remove_possible_set,
            update_cells!(0 => { 3 => 7, { 2, 4 } => {3, 2, 7, 1}  }),
            "remove_possible_set was incorrect"
        );
        assert_eq!(
            update_sets.concrete_set,
            concrete_set![3, 2, 7, 1],
            "concrete_set was incorrect"
        );
        assert_eq!(
            update_sets.make_concrete_set,
            im::hashset![],
            "make_concrete_set was incorrect"
        );
    }
    #[test]
    fn can_build_valid_update_sets_with_make_concrete() {
        let board = board!([[3, 2, ?, { 9, 7 }, ?, 7, 1, { 4 }, { 4, 5, 9 }]]);
        let cell_set = cell_set!(row(0, board));

        let update_sets = UpdateSets::try_from(cell_set).unwrap();

        assert_eq!(
            update_sets.possible_set,
            pos!(iter 0, {2, 3, 4, 7, 8}).collect(),
            "possible_set was incorrect"
        );
        assert_eq!(
            update_sets.remove_possible_set,
            update_cells!(0 => { 3 => 7, { 2, 4 } => {3, 2, 7, 1}  }),
            "remove_possible_set was incorrect"
        );
        assert_eq!(
            update_sets.concrete_set,
            concrete_set![3, 2, 7, 1],
            "concrete_set was incorrect"
        );
        assert_eq!(
            update_sets.make_concrete_set,
            update_cells!(0 => { 7 => 4 }),
            "make_concrete_set was incorrect"
        );
    }
    #[test]
    fn cell_list_to_update_sets_errors_when_board_invalid() {
        let board = board!([[3, 2, ?, { 9, 7 }, ?, 3, 1, { 4 }, { 4, 5, 9 }]]);
        let cell_set = cell_set!(row(0, board));

        assert_eq!(
            UpdateSets::try_from(cell_set),
            Err(UpdateError::InvalidConcrete)
        );
    }

    #[test]
    fn update_removes_possible() {
        let possible_set: HashSet<_> = pos!(iter 0, {2, 3, 4, 7, 8}).collect();
        let concrete_set = concrete_set![3, 2, 7, 1];
        let update_sets = UpdateSets {
            board: board!([[3, 2, ?, {4, 5, 7}, ?, 7, 1, {4, 5}, {4, 5, 9}]]),
            possible_set: possible_set.clone(),
            remove_possible_set: update_cells!(0 => {
                3 => 7,
                { 2, 4 } =>  { 3, 2, 7, 1 }
            }),
            concrete_set: concrete_set.clone(),
            make_concrete_set: im::hashset![],
        };
        let updated = update_sets.update().unwrap();

        assert_eq!(
            updated.board,
            board!([[3, 2, { 4, 5, 6, 8, 9 }, { 4, 5 }, { 4, 5, 6, 8, 9 }, 7, 1, { 4, 5 }, { 4, 5, 9 }]])
        );
        assert_eq!(updated.possible_set, possible_set);
        assert_eq!(updated.concrete_set, concrete_set);
        assert_eq!(updated.remove_possible_set, im::hashset![]);
        assert_eq!(updated.make_concrete_set, im::hashset![]);
    }
    #[test]
    fn update_with_make_concrete() {
        let possible_set: HashSet<_> = pos!(iter 0, {2, 3, 4, 7, 8}).collect();
        let concrete_set = concrete_set![3, 2, 7, 1];
        let update_sets = UpdateSets {
            board: board!([[3, 2, ?, { 9, 7 }, ?, 7, 1, { 4 }, { 4, 5, 9 }]]),
            possible_set: possible_set.clone(),
            remove_possible_set: update_cells!(0 => {
                3 => 7,
                { 2, 4 } =>  { 3, 2, 7, 1 }
            }),
            concrete_set: concrete_set.clone(),
            make_concrete_set: update_cells!(0 => { 7 => 4 }),
        };
        let updated = update_sets.update().unwrap();

        assert_eq!(
            updated.board,
            board!([[3, 2, { 4, 5, 6, 8, 9 }, 9, { 4, 5, 6, 8, 9 }, 7, 1, 4, { 4, 5, 9 }]])
        );
        assert_eq!(updated.possible_set, pos!(iter 0, { 2, 4, 8 }).collect());
        assert_eq!(updated.concrete_set, concrete_set![3, 2, 9, 7, 1, 4]);
        assert_eq!(
            updated.remove_possible_set,
            update_cells!(0 => { { 2, 4, 8 } => { 4, 9 } })
        );
        assert_eq!(updated.make_concrete_set, im::hashset![]);
    }
    #[test]
    fn update_errors_when_overlapping_make_concrete() {
        let update_sets = UpdateSets {
            board: board!([[1, 2, 3, { 4 }, { 4 }, 5, 6, 7, 8]]),
            possible_set: pos!(iter 0, 4).collect(),
            remove_possible_set: im::hashset![],
            concrete_set: concrete_set![1, 2, 3, 5, 6, 7, 8],
            make_concrete_set: update_cells!(0 => { { 3, 4 } => 4 }),
        };
        assert_eq!(update_sets.update(), Err(UpdateError::InvalidConcrete));
    }
    #[test]
    fn update_errors_when_no_possibility_left() {
        let update_sets = UpdateSets {
            board: board!([[1, 2, 3, 4, { 4, 5 }, 5, 6, 7, 8]]),
            possible_set: pos!(iter 0, 4).collect(),
            remove_possible_set: update_cells!(0 => { 4 => { 4, 5 } }),
            concrete_set: concrete_set![1, 2, 3, 5, 6, 7, 8],
            make_concrete_set: im::hashset![],
        };
        assert_eq!(update_sets.update(), Err(UpdateError::InvalidConcrete));
    }
}
