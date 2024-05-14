use crate::gpx;

#[derive(Debug)]
pub enum Error {
    // std::fs
    StdIO(String),
    // polars
    Polars(String),
    // // models,
    // ModelError(model::Error),
    // serde
    Json(String),
    // Reqwest
    Reqwest(reqwest::Error),
    // GPX
    Gpx(gpx::Error),
    //
    UnknownOutputFormat(String),
}

// region:    --- Froms
impl From<std::io::Error> for Error {
    fn from(val: std::io::Error) -> Self {
        Self::StdIO(val.to_string())
    }
}
impl From<polars::error::PolarsError> for Error {
    fn from(val: polars::error::PolarsError) -> Self {
        Self::Polars(val.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(val: serde_json::Error) -> Self {
        Self::Json(val.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(val: reqwest::Error) -> Self {
        Self::Reqwest(val)
    }
}
impl From<gpx::Error> for Error {
    fn from(val: gpx::Error) -> Self {
        Self::Gpx(val)
    }
}

// impl From<model::Error> for Error {
// 	fn from(val: crate::model::error::Error) -> Self {
// 		Self::ModelError(val)
// 	}
// }

// endregion: --- Froms

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate
