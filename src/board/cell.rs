use super::{Board, CellPos, Index};
use crate::UpdateError;
use anyhow::Result;
use im::HashSet;
use nutype::nutype;
use std::{hash::Hash, iter::successors};

/// An Index of a board/row/column
#[nutype(
    validate(less = 9),
    derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)
)]
pub struct CellVal(usize);
impl CellVal {
    /// an iterator over all possible cell values
    pub fn cell_vals() -> impl Iterator<Item = Self> {
        (0..).map_while(|i| Self::new(i).ok())
    }
}

/// an immutable set of the possible values (`CellVal`) a Cell can be
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PossibleCells(HashSet<CellVal>);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Cell {
    Concrete(CellVal),
    Possibities(HashSet<CellVal>),
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Possibities(CellVal::cell_vals().collect())
    }
}
impl Cell {
    pub(super) fn new(inner: Option<u8>) -> Result<Self> {
        Ok(match inner {
            None => Cell::Possibities(CellVal::cell_vals().collect()),
            Some(i) => Cell::Concrete(CellVal::new(i as usize)?),
        })
    }
    /// make the cell concrete using the given number
    ///
    /// if the cell has eliminated num as an option, return InvalidConcrete error
    pub(super) fn make_concrete_cell(&self, num: CellVal) -> Result<Self, UpdateError> {
        use Cell::*;
        Ok(match self {
            &Concrete(val) if val != num => Concrete(val),
            Possibities(set) if set.contains(&num) => Concrete(num),
            _ => Err(UpdateError::InvalidConcrete)?,
        })
    }
    /// removes the possibility from the list if it is there, creating a new copy as needed
    pub(super) fn remove_possibility(&self, num: CellVal) -> Self {
        use Cell::*;
        match self {
            Possibities(set) if set.contains(&num) => Possibities(set.without(&num)),
            // clone should be constant time
            Possibities(set) => Possibities(set.clone()),
            &Concrete(val) => Concrete(val),
        }
    }
    fn possible_is_concrete(&self, pos: CellPos) -> Option<(CellPos, CellVal)> {
        match self {
            Cell::Possibities(set) if set.len() == 1 => {
                set.into_iter().next().map(|&val| (pos, val))
            }
            _ => None,
        }
    }
}

impl FromIterator<(CellPos, Cell)> for Board {
    fn from_iter<T: IntoIterator<Item = (CellPos, Cell)>>(iter: T) -> Self {
        let mut board: Board = Default::default();
        for (pos, cell) in iter {
            *board.mut_cell(pos) = cell;
        }
        board
    }
}

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
                Cell::Possibities(_) => None,
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

macro_rules! cell_list {
    ($name:ident($single:ident, $many:ident) {$cell_at:item}) => {
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        pub(crate) struct $name<'b> {
            index: Index,
            board: &'b Board,
        }
        impl<'b> $name<'b> {
            $cell_at
        }
        impl<'b> From<($name<'b>, &'b Board)> for CellSet<'b> {
            fn from(value: ($name<'b>, &'b Board)) -> Self {
                Self {
                    set: Index::indexes().map(|i| value.0.cell_at(i)).collect(),
                    board: value.1,
                }
            }
        }
        impl Hash for $name<'_> {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.index.hash(state);
            }
        }
        impl Board {
            pub(crate) fn $single(&self, index: Index) -> $name {
                $name { index, board: self }
            }
            pub(crate) fn $many(&self) -> impl Iterator<Item = $name> {
                Index::indexes().map(|index| self.$single(index))
            }
        }
    };
}

cell_list!(Row(row, rows) {
    fn cell_at(&self, index: Index) -> CellPos {
        CellPos {
            row: self.index,
            column: index,
        }
    }
});

cell_list!(Column(column, columns) {
    fn cell_at(&self, index: Index) -> CellPos {
        CellPos {
            column: self.index,
            row: index,
        }
    }
});

cell_list!(House(house, houses) {
    /// houses are ordered left to right top to bottom
    /// (so 4 is the center house)
    fn cell_at(&self, index: Index) -> CellPos {
        let house = self.index.into_inner();
        let i = index.into_inner();
        CellPos {
            column: Index::new((house % 3) * 3 + (i % 3)).unwrap(),
            row: Index::new((house / 3) * 3 + (i/ 3)).unwrap(),
        }
    }
});

#[cfg(test)]
mod test {
    use super::{Board, CellPos, Index};
    use crate::board::cell::House;

    #[test]
    fn house_cell_at_works() {
        let board: Board = Default::default();

        let house = House {
            index: Index::new(3).unwrap(),
            board: &board,
        };

        assert_eq!(
            house.cell_at(Index::new(5).unwrap()),
            CellPos {
                row: Index::new(4).unwrap(),
                column: Index::new(2).unwrap(),
            }
        )
    }
}
