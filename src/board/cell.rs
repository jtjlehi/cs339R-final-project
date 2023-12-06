use super::{Board, CellPos, Index};
use crate::UpdateError;
use anyhow::Result;
use im::HashSet;
use nutype::nutype;
use std::hash::Hash;

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
    pub(super) fn possible_is_concrete(&self, pos: CellPos) -> Option<(CellPos, CellVal)> {
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
pub(super) trait CellAt {
    fn cell_at(&self, index: Index) -> CellPos;
}

macro_rules! cell_list {
    ($name:ident($single:ident, $many:ident) {$cell_at:item}) => {
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        pub(crate) struct $name<'b> {
            index: Index,
            board: &'b Board,
        }
        impl<'b> CellAt for $name<'b> {
            $cell_at
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
    use super::{Board, CellAt, CellPos, Index};
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
