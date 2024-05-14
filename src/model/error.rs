use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub enum Error {
    // models errors
    FailToDeserializeRadar(String),
    // Json
    JsonError(String),
}

// region:    --- Error Boilerplate

// to be able to print content for errors (and serialize for json entries)
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {} // to be able to use the ? for printing

// endregion: --- Error Boilerplate
