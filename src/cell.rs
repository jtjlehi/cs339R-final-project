pub(crate) enum UpdateError {}
pub(crate) enum Cell {}
/// a CellList is the representation of the cells in a row/column/house
///
/// a CellList provides:
/// - ways to update the cell values while maintaining certain rules
/// - ways to query the CellList
///
/// ## Queries
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
    fn cell_at(&self, index: usize) -> &Cell;
    // /// provide a way to get a mutable cell
    // ///
    // /// cell_at(n) & mut_cell_at(n) should both return the same cell
    // fn mut_cell_at(&mut self, index: usize) -> &mut Cell;

    // -- queries --

    /// a list of all the cells in order specified by `cell_at`
    ///
    /// while it is assumed to be ordered in a determined manner, it may not be if cell_at is
    /// determined
    fn all_cells(&self) -> Vec<&Cell> {
        todo!()
    }
    /// gets all cells that meet predicate (including concrete)
    fn cells_that(&self, predicate: impl FnOnce(&Cell) -> bool) -> CellSet {
        todo!()
    }
    /// get all cells which could be the specified number
    ///
    /// *passing a number greater than 8 returns an empty set*
    fn cells_of_num(&self, num: usize) -> CellSet {
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
    fn remove_cell_choice(&self, index: usize, choice: usize) -> Result<Self, UpdateError> {
        todo!()
    }

    /// update cell to be the concrete value
    fn choose_cell(&self, index: usize, choice: usize) -> Result<Self, UpdateError> {
        todo!()
    }
}
/// helper function used by CellList functions to verify it is in a valid state
///
/// also updates cell list (if possible) so some of the rules which can be true, are
fn valid_cell_list<C: CellList>(cell_list: &C) -> Result<C, UpdateError> {
    todo!()
}

/// An unordered set of cells used for updating
pub(crate) struct CellSet;
