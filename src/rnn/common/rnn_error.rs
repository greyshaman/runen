use std::error::Error;

/// Runen library errors
#[derive(Debug)]
pub enum RnnError {
    /// Already has key when attempt insert element into Map or Set
    OccupiedKey,

    /// Not found key when try to use it to access element in Map or Set
    IdNotFound,

    /// Error on attempt to create unique entity with used Id
    IdBusy(String),

    /// Not found matched data by provided pattern
    PatternNotFound,

    /// Expected data not present
    NotPresent(String),

    /// Not supported argument value
    NotSupportedArgValue,

    /// When create entity in container which should be single
    OnlySingleAllowed,

    /// When connection to self not allowed
    ClosedLoop,
}

impl std::fmt::Display for RnnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Error for RnnError {}
