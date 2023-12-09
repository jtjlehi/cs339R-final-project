use super::{Board, CellPos, Index};
use crate::UpdateError;
use anyhow::Result;
use im::HashSet;
use nutype::nutype;
use std::hash::Hash;

/// An Index of a board/row/column
#[nutype(
    validate(less_or_equal = 9, greater = 0),
    derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)
)]
pub struct CellVal(usize);
impl CellVal {
    /// an iterator over all possible cell values
    pub fn cell_vals() -> impl Iterator<Item = Self> {
        (1..).map_while(|i| Self::new(i).ok())
    }
}

/// an immutable set of the possible values (`CellVal`) a Cell can be
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PossibleCells(HashSet<CellVal>);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Cell {
    Concrete(CellVal),
    Possibilities(HashSet<CellVal>),
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Possibilities(CellVal::cell_vals().collect())
    }
}
impl Cell {
    pub(super) fn new(inner: Option<u8>) -> Result<Self> {
        Ok(match inner {
            None => Cell::Possibilities(CellVal::cell_vals().collect()),
            Some(i) => Cell::Concrete(CellVal::new(i as usize)?),
        })
    }
    /// make the cell concrete using the given number
    ///
    /// if the cell has eliminated num as an option, return InvalidConcrete error
    pub(super) fn make_concrete_cell(&self, num: CellVal) -> Result<Self, UpdateError> {
        use Cell::*;
        Ok(match self {
            &Concrete(val) if val == num => Concrete(val),
            Possibilities(set) if set.contains(&num) => Concrete(num),
            _ => Err(UpdateError::InvalidConcrete)?,
        })
    }
    /// removes the possibility from the list if it is there, creating a new copy as needed
    pub(super) fn remove_possibility(&self, num: CellVal) -> Self {
        use Cell::*;
        match self {
            Possibilities(set) if set.contains(&num) => Possibilities(set.without(&num)),
            // clone should be constant time
            Possibilities(set) => Possibilities(set.clone()),
            &Concrete(val) => Concrete(val),
        }
    }
    pub(super) fn remove_possibilities(
        &self,
        vals: &HashSet<CellVal>,
    ) -> Result<Self, UpdateError> {
        use Cell::*;
        Ok(match self {
            Possibilities(set) if set.is_empty() => Err(UpdateError::Impossible)?,
            // clone should be constant time
            Possibilities(set) => {
                let out = set.clone().relative_complement(vals.clone());
                if out.is_empty() {
                    Err(UpdateError::Impossible)?
                } else {
                    Possibilities(out)
                }
            }
            &Concrete(val) => Concrete(val),
        })
    }
    pub(super) fn possible_is_concrete(&self) -> Option<CellVal> {
        match self {
            Cell::Possibilities(set) if set.len() == 1 => set.clone().into_iter().next(),
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
pub(crate) trait ToSet {
    fn cell_at(i: Index, j: Index) -> CellPos;
    fn to_set(i: Index) -> im::HashSet<CellPos> {
        Index::indexes()
            .map(|j| Self::cell_at(i, j))
            .collect::<im::HashSet<CellPos>>()
    }
}

pub(crate) struct Row;
impl ToSet for Row {
    fn cell_at(i: Index, j: Index) -> CellPos {
        CellPos { row: i, column: j }
    }
}

pub(crate) struct Column;
impl ToSet for Column {
    fn cell_at(i: Index, j: Index) -> CellPos {
        CellPos { column: i, row: j }
    }
}

pub(crate) struct House;
impl ToSet for House {
    /// houses are ordered left to right top to bottom
    /// (so 4 is the center house)
    fn cell_at(i: Index, j: Index) -> CellPos {
        let house = i.into_inner();
        let j = j.into_inner();
        CellPos {
            column: Index::new((house % 3) * 3 + (j % 3)).unwrap(),
            row: Index::new((house / 3) * 3 + (j / 3)).unwrap(),
        }
    }
}

#[cfg(test)]
pub(super) mod macros {
    macro_rules! cell_val {
        ($num:expr) => {
            crate::board::cell::CellVal::new($num).unwrap()
        };
    }
    macro_rules! index {
        ($num:expr) => {
            Index::new($num).unwrap()
        };
    }
    macro_rules! cell {
        (? $($val:expr),* ) => {
            crate::board::cell::Cell::Possibilities(im::hashset![$(cell_val!($val)),*])
        };
        ($val:expr) => {
            crate::board::cell::Cell::Concrete(cell_val!($val))
        };
    }
    pub(crate) use {cell, cell_val, index};
}

#[cfg(test)]
mod test {
    use super::{macros::*, *};
    use crate::board::cell::House;
    use crate::board::macros::*;

    #[test]
    fn make_concrete_throws_error_for_different_val() {
        let cell = macros::cell!(1);
        assert_eq!(
            cell.make_concrete_cell(cell_val!(3)),
            Err(UpdateError::InvalidConcrete)
        );
    }
    #[test]
    fn make_concrete_keeps_same_val() {
        let cell = cell!(1);
        assert_eq!(cell.make_concrete_cell(cell_val!(1)), Ok(cell!(1)));
    }
    #[test]
    fn make_concrete_makes_concrete() {
        let cell = cell!(? 3, 4, 8);
        assert_eq!(cell.make_concrete_cell(cell_val!(3)), Ok(cell!(3)));
    }
    #[test]
    fn make_concrete_fails_if_not_possible() {
        let cell = cell!(? 1, 5, 8, 9);
        assert_eq!(
            cell.make_concrete_cell(cell_val!(3)),
            Err(UpdateError::InvalidConcrete)
        );
    }

    #[test]
    fn remove_possibility_does_nothing_for_concrete() {
        let cell = cell!(6);
        assert_eq!(cell.remove_possibility(cell_val!(6)), cell!(6));
        assert_eq!(cell.remove_possibility(cell_val!(8)), cell!(6));
    }
    #[test]
    fn remove_possibility_removes_possibility() {
        let cell = cell!(? 5, 7);
        assert_eq!(cell.remove_possibility(cell_val!(5)), cell!(? 7));
    }
    #[test]
    fn remove_possibilities_does_nothing_if_not_needed() {
        let cell = cell!(? 5, 7);
        assert_eq!(cell.remove_possibility(cell_val!(2)), cell!(? 5, 7));
    }

    #[test]
    fn possible_is_concrete_gets_correct_val() {
        // possibilities
        let cell = cell!(? 1);
        assert_eq!(cell.possible_is_concrete(), Some(cell_val!(1)))
    }
    #[test]
    fn possible_is_concrete_returns_none_for_concrete() {
        let cell = cell!(3);
        assert_eq!(cell.possible_is_concrete(), None);
    }
    #[test]
    fn possible_is_concrete_returns_none_for_more_then_one() {
        let cell = cell!(? 3, 5);
        assert_eq!(cell.possible_is_concrete(), None)
    }

    // macro_rules! test_cell_list {
    //     ($test_name:ident => $name:ident($single:ident)) => {
    //         #[test]
    //         fn $test_name() {
    //             let b = board!([]);
    //             let single = b.$single(index!(1));
    //             assert_eq!(
    //                 single,
    //                 $name {
    //                     index: index!(1),
    //                     board: &b
    //                 }
    //             );
    //         }
    //     };
    // }
    // test_cell_list!(rows_works => Row(row));
    // test_cell_list!(columns_works => Column(column));
    // test_cell_list!(houses_works => House(house));
    //
    #[test]
    fn house_cell_at_works() {
        assert_eq!(House::cell_at(index!(3), index!(5)), pos!(4, 2))
    }
}
