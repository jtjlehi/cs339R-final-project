#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum UpdateError {
    InvalidConcrete,
    InvalidCellVal,
    MultipleConcrete,
    InitError,
    InvalidIndex,
}
