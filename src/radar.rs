// region:    --- generic POI

use crate::ov2::{POIRecord, POIRecordBuilder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Radar {
    pub id: String,
    #[serde(rename = "type")]
    pub radar_type: String,
    #[serde(rename = "lat")]
    pub latitude: f32,
    #[serde(rename = "lng")]
    pub longitude: f32,
}

// endregion: --- generic POI

// impl Into<POIRecord> for Radar {
//     fn into(self) -> POIRecord {
//         POIRecordBuilder::default()
//             .latitude(self.latitude)
//             .longitude(self.longitude)
//             .label(self.id)
//             .build()
//             .unwrap()
//     }
// }

impl From<Radar> for POIRecord {
    fn from(val: Radar) -> Self {
        POIRecordBuilder::default()
            .latitude(val.latitude)
            .longitude(val.longitude)
            .label(val.id)
            .build()
            .unwrap()
    }
}
