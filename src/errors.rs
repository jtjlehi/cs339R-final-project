use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum UpdateError {
    #[error("found and incorrect concrete value in a cell")]
    InvalidConcrete,
    #[error("tried to set a concrete value where there already was one")]
    MultipleConcrete,
    #[error("we didn't get past take off")]
    InitError,
    #[error("didn't finish")]
    Incomplete,
}
