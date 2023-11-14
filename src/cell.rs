use crate::board::{Board, BoardState, Index};
use im::{hashset, HashSet};
use std::{default, hash::Hash, ops::Deref};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum UpdateError {
    InvalidConcrete,
    InvalidCellVal,
    MultipleConcrete,
    InitError,
}

/// a newtype CellVall representing the value a cell can be (1-9)
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct CellVal(usize);
impl CellVal {
    pub(crate) fn inner(&self) -> usize {
        self.0
    }
    /// attempts to build the given number into a cell value
    pub(crate) fn build(i: usize) -> Result<Self, UpdateError> {
        if i > 9 || i == 0 {
            Err(UpdateError::InvalidCellVal)
        } else {
            Ok(Self(i))
        }
    }
    /// an iterator over all possible cell values
    pub(crate) fn cell_vals() -> impl Iterator<Item = Self> {
        (0..).map_while(|i| Self::build(i).ok())
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
    /// make the cell concrete using the given number
    ///
    /// if the cell has eliminated num as an option, return InvalidConcrete error
    fn make_concrete_cell(&self, num: CellVal) -> Result<Self, UpdateError> {
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
    pub(crate) row: Index,
    pub(crate) column: Index,
    pub(crate) board: &'b Board,
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

impl<'b> CellRef<'b> {
    /// attempt to make the cell concrete, updating the board as needed
    pub(crate) fn make_concrete(self, num: CellVal) -> BoardState {
        match self
            .board
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
            .collect::<Result<Board, _>>()
        {
            Ok(board) => BoardState::Valid(board),
            Err(why) => BoardState::Err(why),
        }
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
    fn cell_at(&self, index: Index) -> CellRef;

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
/// An unordered set of cells used for updating
pub struct CellSet<'b>(pub(crate) HashSet<CellRef<'b>>);

impl<'b> IntoIterator for CellSet<'b> {
    type Item = CellRef<'b>;
    // may change, this is the placeholder for now
    type IntoIter = im::hashset::ConsumingIter<CellRef<'b>>; // <HashSet<CellRef<'b>> as IntoIterator>::IntoIter

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
