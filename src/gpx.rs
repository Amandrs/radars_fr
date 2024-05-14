/*
GPX schema
https://www.topografix.com/GPX/1/1/

*/

use polars::prelude::*;
use std::str::FromStr;

pub const GPX_HEADER: &str = r#"<gpx version="1.1" xmlns="http://www.topografix.com/GPX/1/1" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/1/1/gpx.xsd">"#;

pub const GPX_FOOTER: &str = "</gpx>";

#[derive(Debug)]
pub struct WayPoint {
    longitude: f32,
    latitude: f32,
    name: String,
}

impl WayPoint {
    // pub fn new(longitude: f32, latitude: f32, name: &str) -> Self {
    //     WayPoint {
    //         longitude,
    //         latitude,
    //         name: name.to_string(),
    //     }
    // }

    pub fn to_gpx(&self) -> String {
        format!(
            "<wpt lat=\"{}\" lon=\"{}\">\n\t<name>{}</name>\n\t<type>radar</type>\n</wpt>\n",
            self.latitude,
            self.longitude,
            self.name.clone()
        )
    }
}

pub fn save_waypoints(df: &DataFrame) -> Result<String> {
    let records: std::result::Result<Vec<WayPoint>,()> = df.clone()
        .select(["id","type","lat","lng"])?
        .into_struct("waypoint")
        .into_iter()
        .map( |s| {
            match s {
                [AnyValue::String(name), AnyValue::String(_), AnyValue::Float64(lat), AnyValue::Float64(lng)] => {
                    //println!("Found {} {} {}",*name,lat,lng); 
                    Ok(
                        WayPoint {
                            longitude : *lng as f32,
                            latitude : *lat as f32,
                            name: String::from(*name)
                        }
                    )
                },
                _ => { println!("Error in {:?}",s); Err(()) }
            }

        })
        .collect();

    let mut result = String::from_str(GPX_HEADER).unwrap();
    result += "\n";
    for wpt in records.unwrap() {
        result += &wpt.to_gpx();
    }
    result += GPX_FOOTER;
    result += "\n";
    Ok(result)
}

// region:    --- Error

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
#[allow(non_snake_case)]
pub enum Error {
    BadDataFrameFormat(polars::error::PolarsError),
    FailWayPoint,
}

impl From<polars::error::PolarsError> for Error {
    fn from(val: polars::error::PolarsError) -> Self {
        Self::BadDataFrameFormat(val)
    }
}

// region       : --- Error boilerplate
impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion    : --- Error boilerplate

// endregion: --- Error
