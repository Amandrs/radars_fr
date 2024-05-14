mod error;
mod poi;

pub use crate::ov2::poi::{POIRecord, SkipperRecord}; // export at module level

use crate::ov2::poi::POIRecordType;

// use and export
pub use self::error::{Error, Result};

// could use derive_builder (but wanted to write it on my own)
// https://docs.rs/derive_builder/latest/derive_builder/

// region:    --- POIBuilder
#[derive(Debug)]
pub struct POIRecordBuilder {
    record_type: POIRecordType,
    longitude: i32,
    latitude: i32,
    label: String,
}

const F32_TO_I32_MULTIPLIER: f32 = 100_000.;

impl POIRecordBuilder {
    pub fn new() -> POIRecordBuilder {
        POIRecordBuilder {
            record_type: POIRecordType::Type_2,
            longitude: 100_000,
            latitude: 5_000,
            label: "test".into(),
        }
    }

    pub fn default() -> Self {
        POIRecordBuilder::new()
    }

    pub fn longitude(&mut self, longitude: f32) -> &mut Self {
        self.longitude = (longitude * F32_TO_I32_MULTIPLIER) as i32;
        self
    }

    pub fn latitude(&mut self, latitude: f32) -> &mut Self {
        self.latitude = (latitude * F32_TO_I32_MULTIPLIER) as i32;
        self
    }

    pub fn label(&mut self, label: String) -> &mut Self {
        self.label = label;
        self
    }

    pub fn build(&mut self) -> Result<POIRecord> {
        let poi = POIRecord::new(
            self.record_type,
            self.longitude,
            self.latitude,
            self.label.clone(),
        );

        match poi {
            Ok(_) => poi,
            Err(_) => Err(Error::FailPOIRecordBuilderBuid),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {

    use crate::ov2::{POIRecordBuilder, POIRecordType};

    #[test]
    fn test_build_POIRecordBuilder_default() {
        let b = POIRecordBuilder::default();
        assert_eq!(b.label, "test".to_string());
        assert_eq!(b.latitude, 5000);
        assert_eq!(b.longitude, 100000);
        assert_eq!(b.record_type, POIRecordType::Type_2);
    }

    #[test]
    fn test_build_POIRecordBuilder_default_build() {
        let p = POIRecordBuilder::default().build().unwrap();
        assert_eq!(
            std::any::type_name_of_val(&p),
            "radars::ov2::poi::POIRecord"
        );
        let b = p.to_binary();
        println!("{:?}", &b);
        assert_eq!(b[0], 2); // type
        assert_eq!(b[1], 18); // length from little-endian
        assert_eq!(b[13..17], [116, 101, 115, 116]); //test
    }

    #[test]
    fn test_build_POIRecordBuilder_chain_set() {
        let mut b = POIRecordBuilder::default();
        b.label("toto".into())
            .longitude(51.32_f32)
            .latitude(8.3_f32);
        assert_eq!(b.longitude, 5132000);
        assert_eq!(b.latitude, 830000);
        assert_eq!(b.label, "toto".to_string());
        let p = b.build().unwrap();
        let bin = p.to_binary();
        assert_eq!(bin[13..17], [116, 111, 116, 111]); //toto
    }
}
