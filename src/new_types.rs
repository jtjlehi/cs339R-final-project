use crate::UpdateError;

/// An Index of a board/row/column
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub(crate) struct Index(usize);
impl Index {
    pub fn build(i: usize) -> Result<Self, UpdateError> {
        if i >= 9 {
            Err(UpdateError::InvalidIndex)
        } else {
            Ok(Self(i))
        }
    }
    pub fn indexes() -> impl Iterator<Item = Self> {
        (0..).map_while(|i| Self::build(i).ok())
    }
    pub fn inner(&self) -> usize {
        self.0
    }
}
/// a newtype CellVall representing the value a cell can be (1-9)
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct CellVal(usize);
impl CellVal {
    pub fn inner(&self) -> usize {
        self.0
    }
    /// attempts to build the given number into a cell value
    pub fn build(i: usize) -> Result<Self, UpdateError> {
        if i > 9 || i == 0 {
            Err(UpdateError::InvalidCellVal)
        } else {
            Ok(Self(i))
        }
    }
    /// an iterator over all possible cell values
    pub fn cell_vals() -> impl Iterator<Item = Self> {
        (0..).map_while(|i| Self::build(i).ok())
    }
}
