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

#[derive(Clone)]
struct ConcreteSet(HashSet<CellVal>);
impl ConcreteSet {
    fn insert(&mut self, val: CellVal) -> Result<(), UpdateError> {
        if self.0.insert(val).is_some() {
            Err(UpdateError::InvalidConcrete)?
        };
        Ok(())
    }
}

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
        let (concretes, positions): (HashSet<_>, HashSet<_>) = cell_set
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
    ) -> UpdateCells {
        concrete_set
            .0
            .iter()
            .flat_map(|&val| possible_set.iter().map(move |&pos| (pos, val)))
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
            board,
            // there aren't any more HashSet changes to make
            make_concrete_set: HashSet::new(),
            remove_possible_set: Self::get_remove_possible_set(&concrete_set, &possible_set),
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

#[derive(Clone)]
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
            remove_possible_set: Self::get_remove_possible_set(&concrete_set, &possible_set),
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
