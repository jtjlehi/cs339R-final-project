use super::{Board, CellPos, Index};
use crate::UpdateError;
use anyhow::Result;
use im::{hashset::ConsumingIter, HashSet};
use nutype::nutype;
use std::{hash::Hash, ops::Deref};

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
        self.pos.hash(state);
    }
}
impl<'b> Deref for CellRef<'b> {
    type Target = Cell;

    fn deref(&self) -> &Self::Target {
        self.board.cell(self.pos)
    }
}
impl<'b> FromIterator<(CellRef<'b>, Cell)> for Board {
    fn from_iter<T: IntoIterator<Item = (CellRef<'b>, Cell)>>(iter: T) -> Self {
        let mut board: Board = Default::default();
        for (CellRef { pos, .. }, cell) in iter {
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
            .flat_map(|row| row.all_cells())
            .collect::<Vec<_>>()
            .into_iter()
    }
}
impl<'b> CellRef<'b> {
    /// attempt to make the cell concrete, updating the board as needed
    pub(crate) fn make_concrete(self, num: CellVal) -> Result<Board> {
        self.board
            .into_iter()
            .map(|cell_ref| {
                let cell = if cell_ref == self {
                    cell_ref.make_concrete_cell(num)?
                } else if cell_ref.pos.row == self.pos.row || cell_ref.pos.column == self.pos.column
                {
                    cell_ref.remove_possibility(num)
                } else {
                    (*cell_ref).clone()
                };
                Ok((cell_ref, cell))
            })
            .collect()
    }
}

#[derive(Clone)]
/// An unordered set of cells used for updating
pub(crate) struct CellSet<'b> {
    set: HashSet<CellPos>,
    board: &'b Board,
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
pub(crate) trait CellList<'b>
where
    Self: Sized + Clone,
{
    /// provide some way to order the cells
    ///
    /// 0 indexed access of cell
    fn cell_at(&self, index: Index) -> CellPos;

    /// a list of all the cells in order specified by `cell_at`
    ///
    /// while it is assumed to be ordered in a determined manner, it may not be if cell_at is
    /// determined
    #[inline]
    fn all_cells(self) -> CellSet<'b> {
        todo!()
    }
    /// get all cells which could be the specified number
    #[inline]
    fn cells_of_num(self, _num: CellVal) -> CellSet<'b> {
        todo!()
    }
    /// if num has no concrete instance, return CellSet of cells where it is possible
    /// if num has a concrete instance, return none
    #[inline]
    fn possible_cells_of_num(self, _num: CellVal) -> Option<CellSet<'b>> {
        todo!()
    }
    /// boolean saying if list has a concrete version of the number
    #[inline]
    fn has_concrete(&self, _num: Index) -> bool {
        todo!()
    }

    /// gives all cells that are in both cell_lists
    fn intersect<C: CellList<'b>>(&self, _other: &C) -> CellSet {
        todo!()
    }

    /// gives cells that are in self but not the other cellList
    fn difference<C: CellList<'b>>(&self, _other: &C) -> CellSet {
        todo!()
    }

    // -- updates --

    /// update cell at index so choice is not an option
    fn remove_cell_choice(&self, _index: Index, _choice: CellVal) -> Result<Self> {
        todo!()
    }

    /// update cell to be the concrete value
    fn choose_cell(&self, _index: Index, _choice: CellVal) -> Result<Self> {
        todo!()
    }
    /// check to make sure the cell_list is valid
    fn valid_cell_list(&self) -> Result<Self> {
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
        impl<'b> CellList<'b> for $name<'b> {
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
    use super::{Board, CellList, CellPos, Index};
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
