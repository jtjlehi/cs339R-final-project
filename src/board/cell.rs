use super::{Board, CellPos, Index};
use crate::UpdateError;
use anyhow::Result;
use im::{hashset::ConsumingIter, HashSet};
use nutype::nutype;
use std::{hash::Hash, iter::successors, ops::Deref};

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
    fn make_concrete_cell(&self, num: CellVal) -> Result<Self> {
        use Cell::*;
        Ok(match self {
            &Concrete(val) if val != num => Concrete(val),
            Possibities(set) if set.contains(&num) => Concrete(num),
            _ => Err(UpdateError::InvalidConcrete)?,
        })
    }
    /// removes the possibility from the list if it is there, creating a new copy as needed
    fn remove_possibility(&self, num: CellVal) -> Self {
        use Cell::*;
        match self {
            Possibities(set) if set.contains(&num) => Possibities(set.without(&num)),
            // clone should be constant time
            Possibities(set) => Possibities(set.clone()),
            &Concrete(val) => Concrete(val),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CellRef<'b> {
    pos: CellPos,
    board: &'b Board,
}
// equality of the board doesn't matter
impl<'b> PartialEq for CellRef<'b> {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}
impl<'b> Eq for CellRef<'b> {}
impl<'b> Hash for CellRef<'b> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let cell: &Cell = self;
        match cell {
            Cell::Concrete(cell_val) => cell_val.hash(state),
            _ => self.pos.hash(state),
        }
    }
}
impl<'b> Deref for CellRef<'b> {
    type Target = Cell;

    fn deref(&self) -> &Self::Target {
        self.board.cell(self.pos)
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
impl<'b> IntoIterator for &'b Board {
    type Item = CellRef<'b>;

    type IntoIter = <Vec<CellRef<'b>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.rows()
            .flat_map(|row| CellSet::from((row, self)))
            .collect::<Vec<_>>()
            .into_iter()
    }
}
impl<'b> CellRef<'b> {}

#[derive(Clone)]
/// An unordered set of cells used for updating
pub(crate) struct CellSet<'b> {
    set: HashSet<CellPos>,
    board: &'b Board,
}
#[derive(Clone)]
struct CellRefSet<'b> {
    set: HashSet<CellRef<'b>>,
    board: &'b Board,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ConcreteSetElement {
    pos: CellPos,
    val: CellVal,
}
impl Hash for ConcreteSetElement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // we don't use position because that isn't what we care about making unique
        self.val.hash(state);
    }
}

#[derive(Clone)]
struct UpdateSets {
    board: Board,
    remove_possible_set: HashSet<CellPos>,
    make_concrete_set: HashSet<CellPos>,
    concrete_set: HashSet<ConcreteSetElement>,
    possible_set: HashSet<CellPos>,
}
impl<'b> TryFrom<CellSet<'b>> for UpdateSets {
    type Error = UpdateError;

    fn try_from(cell_set: CellSet<'b>) -> std::prelude::v1::Result<Self, Self::Error> {
        let board = cell_set.board.clone();
        let (concrete_set, possible_set) = Self::get_initial(cell_set, board.clone())?;

        Ok(UpdateSets {
            board,
            remove_possible_set: HashSet::new(),
            make_concrete_set: HashSet::new(),
            possible_set,
            concrete_set,
        })
    }
}
impl UpdateSets {
    fn get_initial(
        cell_set: CellSet,
        board: Board,
    ) -> Result<(HashSet<ConcreteSetElement>, HashSet<CellPos>), UpdateError> {
        let (concretes, positions): (HashSet<_>, HashSet<_>) = cell_set
            .set
            .clone()
            .into_iter()
            .filter_map(|pos| match board.cell(pos) {
                &Cell::Concrete(val) => Some((ConcreteSetElement { val, pos }, pos)),
                Cell::Possibities(_) => None,
            })
            .unzip();

        let mut concrete_set = HashSet::new();
        for concrete in concretes {
            if concrete_set.insert(concrete).is_some() {
                Err(UpdateError::InvalidConcrete)?
            };
        }
        let possible_set = cell_set.set.clone().difference(positions.clone());
        Ok((concrete_set, possible_set))
    }
    fn update(&self) -> Result<Self, UpdateError> {
        todo!()
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
        Ok(cells) => cells.finished(),
        Err(_) => false,
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

pub(crate) struct CellIter<'b> {
    board: &'b Board,
    iter: ConsumingIter<CellPos>,
}
impl<'b> Iterator for CellIter<'b> {
    type Item = CellRef<'b>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(CellRef {
            pos: self.iter.next()?,
            board: self.board,
        })
    }
}
impl<'b> IntoIterator for CellSet<'b> {
    type Item = CellRef<'b>;
    // may change, this is the placeholder for now
    type IntoIter = CellIter<'b>;

    fn into_iter(self) -> Self::IntoIter {
        CellIter {
            iter: self.set.into_iter(),
            board: self.board,
        }
    }
}

impl Board {
    /// get all of the cells on the board that could possibly be the indicated value
    ///
    /// skip the concrete values
    pub(crate) fn possible_cells_of_num(&self, num: CellVal) -> impl Iterator<Item = CellRef> {
        CellPos::all_cell_pos().filter_map(move |pos| {
            let cell_ref = CellRef { pos, board: self };
            match *cell_ref {
                Cell::Possibities(ref set) if set.contains(&num) => Some(cell_ref),
                _ => None,
            }
        })
    }
    /// iterator over all possible boards where one cell is made concrete
    ///
    /// for each possible cell, all possibilities are iterated over
    pub(crate) fn possible_updates(self) -> impl Iterator<Item = Self> {
        CellPos::all_cell_pos().flat_map(move |pos| pos.make_concrete_boards(self.clone()))
    }
}
impl CellPos {
    fn make_concrete_boards(self, board: Board) -> impl Iterator<Item = Board> {
        let cell_vals = match board.cell(self) {
            Cell::Concrete(_) => HashSet::new(),
            Cell::Possibities(ref set) => set.clone(),
        };
        let update_cell = move |num| {
            board
                .into_iter()
                .filter_map(|board_cell @ CellRef { pos, .. }| {
                    let cell = if pos == self {
                        board_cell.make_concrete_cell(num).ok()?
                    } else if pos.row == self.row || pos.column == self.column {
                        board_cell.remove_possibility(num)
                    } else {
                        (*board_cell).clone()
                    };
                    Some((pos, cell))
                })
                .collect()
        };
        cell_vals.into_iter().map(update_cell)
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
        // impl<'b> CellList<'b> for $name<'b> {
        //     $cell_at
        // }
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
