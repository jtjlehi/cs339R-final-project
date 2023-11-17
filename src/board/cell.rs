use crate::board::Board;
use crate::new_types::{CellVal, Index};
use crate::UpdateError;
use im::HashSet;
use std::{hash::Hash, ops::Deref};

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
    /// make the cell concrete using the given number
    ///
    /// if the cell has eliminated num as an option, return InvalidConcrete error
    pub(crate) fn make_concrete_cell(&self, num: CellVal) -> Result<Self, UpdateError> {
        use Cell::*;
        Ok(match self {
            &Concrete(val) if val != num => Concrete(val),
            Possibities(set) if set.contains(&num) => Concrete(num),
            _ => Err(UpdateError::InvalidConcrete)?,
        })
    }
    /// removes the possibility from the list if it is there, creating a new copy as needed
    pub(crate) fn remove_possibility(&self, num: CellVal) -> Self {
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
    row: Index,
    column: Index,
    board: &'b Board,
}
// equality of the board doesn't matter
impl<'b> PartialEq for CellRef<'b> {
    fn eq(&self, other: &Self) -> bool {
        self.row == other.row && self.column == other.column
    }
}
impl<'b> Eq for CellRef<'b> {}
impl<'b> Hash for CellRef<'b> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.row.hash(state);
        self.column.hash(state);
    }
}
impl<'b> Deref for CellRef<'b> {
    type Target = Cell;

    fn deref(&self) -> &Self::Target {
        self.board.cell(self.row, self.column)
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
impl<'b> IntoIterator for &'b Board {
    type Item = CellRef<'b>;

    type IntoIter = <Vec<CellRef<'b>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.rows()
            .flat_map(|row| row.all_cells())
            .collect::<Vec<_>>()
            .into_iter()
    }
}
// |cell_ref| {
//                 let cell = if cell_ref == self {
//                     cell_ref.make_concrete_cell(num)?
//                 } else if cell_ref.row == self.row || cell_ref.column == self.column {
//                     cell_ref.remove_possibility(num)
//                 } else {
//                     (*cell_ref).clone()
//                 };
//                 Ok((cell_ref, cell))
//             }
impl<'b> CellRef<'b> {
    /// attempt to make the cell concrete, updating the board as needed
    pub(crate) fn make_concrete(self, num: CellVal) -> Result<Board, UpdateError> {
        self.board
            .into_iter()
            .map(|cell_ref| {
                let cell = if cell_ref == self {
                    cell_ref.make_concrete_cell(num)?
                } else if cell_ref.row == self.row || cell_ref.column == self.column {
                    cell_ref.remove_possibility(num)
                } else {
                    (*cell_ref).clone()
                };
                Ok((cell_ref, cell))
            })
            .collect()
    }
}

/// a CellList is the representation of the cells in a row/column/house
///
/// a CellList provides:
/// - ways to update the cell values while maintaining certain rules
/// - ways to query the CellList
///
/// ##Queries
///
/// different queries provide different ways of looking at the information in the cellList.
///
/// ## Rules
///
/// The following rules must be met by a CellList:
/// - there can only be one concrete instance of each cell value 1-9
/// - if a value can only exist in one cell, that cell has that concrete value
/// - if a cell can only have one value, that cell has that value
/// - if a value cannot exist, the cellList is invalid
/// - if a cell cannot have any values, the cellList is invalid
///
/// ## Updating
///
/// All updating functions are fallible. The only way for an update to succeed is if all of the
/// rules are still satisfied at the end of the update.
pub(crate) trait CellList
where
    Self: Sized,
{
    /// provide some way to order the cells
    ///
    /// 0 indexed access of cell
    fn cell_at(&self, index: Index) -> Result<CellRef, UpdateError>;

    /// a list of all the cells in order specified by `cell_at`
    ///
    /// while it is assumed to be ordered in a determined manner, it may not be if cell_at is
    /// determined
    #[inline]
    fn all_cells<'b>(self) -> CellSet<'b> {
        todo!()
    }
    /// gets all cells that meet predicate (including concrete)
    #[inline]
    fn cells_that(&self, predicate: impl FnOnce(CellRef) -> bool) -> CellSet {
        todo!()
    }
    /// get all cells which could be the specified number
    #[inline]
    fn cells_of_num(&self, num: CellVal) -> CellSet {
        todo!()
    }
    /// if num has no concrete instance, return CellSet of cells where it is possible
    /// if num has a concrete instance, return none
    #[inline]
    fn possible_cells_of_num<'b>(self, num: CellVal) -> Option<CellSet<'b>> {
        todo!()
    }
    /// boolean saying if list has a concrete version of the number
    #[inline]
    fn has_concrete(&self, num: Index) -> bool {
        todo!()
    }

    /// gives all cells that are in both cell_lists
    fn intersect<C: CellList>(&self, other: &C) -> CellSet {
        todo!()
    }

    /// gives cells that are in self but not the other cellList
    fn difference<C: CellList>(&self, other: &C) -> CellSet {
        todo!()
    }

    // -- updates --

    /// update cell at index so choice is not an option
    fn remove_cell_choice(&self, index: Index, choice: CellVal) -> Result<Self, UpdateError> {
        todo!()
    }

    /// update cell to be the concrete value
    fn choose_cell(&self, index: Index, choice: CellVal) -> Result<Self, UpdateError> {
        todo!()
    }
    /// check to make sure the cell_list is valid
    fn valid_cell_list(&self) -> Result<Self, UpdateError> {
        todo!()
    }
}
macro_rules! cell_list {
    ($name:ident($single:ident, $many:ident) {$cell_at:item}) => {
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        pub(crate) struct $name<'b> {
            index: Index,
            board: &'b Board,
        }
        impl Hash for $name<'_> {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.index.hash(state);
            }
        }
        impl<'b> CellList for $name<'b> {
            $cell_at
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
    fn cell_at(&self, index: Index) -> Result<CellRef, UpdateError> {
        Ok(CellRef {
            row: self.index,
            column: index,
            board: self.board,
        })
    }
});

cell_list!(Column(column, columns) {
    fn cell_at(&self, index: Index) -> Result<CellRef, UpdateError> {
        Ok(CellRef {
            column: self.index,
            row: index,
            board: self.board,
        })
    }
});

cell_list!(House(house, houses) {
    /// houses are ordered left to right top to bottom
    /// (so 4 is the center house)
    fn cell_at(&self, index: Index) -> Result<CellRef, UpdateError> {
        let house = self.index.inner();
        let i = self.index.inner();
        Ok(CellRef {
            column: Index::build((house % 3) * 3 + (i % 3))?,
            row: Index::build((house / 3) * 3 + (i/ 3))?,
            board: self.board,
        })
    }
});

/// An unordered set of cells used for updating
pub(crate) struct CellSet<'b>(pub(crate) HashSet<CellRef<'b>>);

impl<'b> IntoIterator for CellSet<'b> {
    type Item = CellRef<'b>;
    // may change, this is the placeholder for now
    type IntoIter = im::hashset::ConsumingIter<CellRef<'b>>; // <HashSet<CellRef<'b>> as IntoIterator>::IntoIter

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod test {
    use crate::board::cell::House;
    use crate::new_types::Index;

    use super::{Board, CellList, CellRef};

    #[test]
    fn house_cell_at_works() {
        let board: Board = Default::default();

        let house = House {
            index: Index::build(3).unwrap(),
            board: &board,
        };

        assert_eq!(
            house.cell_at(Index::build(5).unwrap()),
            Ok(CellRef {
                row: Index::build(4).unwrap(),
                column: Index::build(2).unwrap(),
                board: &board
            })
        )
    }
}
