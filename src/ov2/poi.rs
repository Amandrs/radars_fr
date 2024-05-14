#![allow(unused)]

use binary_rw::{BinaryReader, BinaryWriter};
use encoding::all::ISO_8859_1;
use encoding::{EncoderTrap, Encoding};
use num_enum::TryFromPrimitive;
use serde::de::{SeqAccess, Visitor};
use serde::Deserialize;
use serde::{ser::SerializeStruct, Serialize};
use std::ffi::CString;
use std::io::{BufWriter, Write};
use tracing_subscriber::field::display::Messages;

use crate::ov2::{Error, Result};

use super::F32_TO_I32_MULTIPLIER;

// region:    --- Lat Lon

// struct POILatitude {
//     lat: u32
// }

// struct POILongitude {
//     lon: u32
// }

// impl POILatitude {

// }

// impl POILongitude {

// }

// trait POILatLong {

// }
// endregion: --- Lat Lon

// region:    --- POIRecord
#[derive(Debug, Clone)]
pub struct POIRecord {
    record_type: POIRecordType,
    record_length: i32,
    longitude: i32,
    latitude: i32,
    label: String,
    label_vec_u8: Vec<u8>,
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct SkipperRecord {
    record_type: POIRecordType, // must be Type_1
    block_length: i32,
    NE_longitude: i32,
    NE_latitude: i32,
    SW_longitude: i32,
    SW_latitude: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum POIRecordType {
    Type_0 = 0_u8,
    Type_1 = 1_u8,
    Type_2 = 2_u8,
    Type_3 = 3_u8,
}

impl POIRecord {
    pub fn new(
        record_type: POIRecordType,
        longitude: i32,
        latitude: i32,
        label: String,
    ) -> Result<POIRecord> {
        let value = match CString::new(label.clone()) {
            Ok(v) => v,
            Err(_) => return Err(Error::FailPOIRecordNew),
        };
        let length = 13 + value.count_bytes() + 1; //header + message + null
        Ok(POIRecord {
            record_type,
            record_length: length as i32,
            longitude,
            latitude,
            label,
            label_vec_u8: value.into_bytes_with_nul(),
        })
    }

    // TODO: to_binary
    // understand why ownership is taken ... ???
    // to_le_ => Little Endian
    pub fn to_binary(&self) -> Vec<u8> {
        let buffer = Vec::<u8>::with_capacity(self.record_length as usize);
        let mut buffer_w = BufWriter::new(buffer);
        let _ = buffer_w.write(&[self.record_type as u8]);
        let _ = buffer_w.write(&self.record_length.to_le_bytes());
        let _ = buffer_w.write(&self.longitude.to_le_bytes());
        let _ = buffer_w.write(&self.latitude.to_le_bytes());
        let _ = buffer_w.write(&self.label_vec_u8);
        buffer_w.buffer().to_vec()
    }

    pub fn write(&self, writer: &mut BinaryWriter) {
        let _ = writer.write_u8(self.record_type as u8);
        let _ = writer.write_i32(self.record_length);
        let _ = writer.write_i32(self.longitude);
        let _ = writer.write_i32(self.latitude);
        for v in &self.label_vec_u8 {
            let _ = writer.write_u8(v);
        }
    }

    pub fn read_from(reader: &mut BinaryReader) -> Result<POIRecord> {
        let t = reader.read_u8().expect("Fail to read u8");
        let s = reader.read_i32().expect("Fail to read i32");
        let lon = reader.read_i32().expect("Fail to read i32");
        let lat = reader.read_i32().expect("Fail to read i32");
        let mut m = Vec::<u8>::with_capacity((s - 13) as usize);
        // reads all the record bytes (to position correctly the head)
        for _ in 13..s {
            m.push(reader.read_u8().expect("Fail to read u8"));
        }
        // removes ending null
        m.pop();
        let msg = std::str::from_utf8(&m).expect("Invalid utf8");
        POIRecord::new(
            POIRecordType::try_from(t).expect("Invalid record type"),
            lon,
            lat,
            msg.to_string(),
        )
    }
}

// endregion: --- POIRecord

// region:    --- Skipper record

impl SkipperRecord {
    pub fn new(records_size: usize, sw: (f64, f64), ne: (f64, f64)) -> Self {
        let block_length = 21 + records_size as i32;

        SkipperRecord {
            record_type: POIRecordType::Type_1,
            block_length,
            NE_longitude: (ne.0 as f32 * F32_TO_I32_MULTIPLIER) as i32,
            NE_latitude: (ne.1 as f32 * F32_TO_I32_MULTIPLIER) as i32,
            SW_longitude: (sw.0 as f32 * F32_TO_I32_MULTIPLIER) as i32,
            SW_latitude: (sw.1 as f32 * F32_TO_I32_MULTIPLIER) as i32,
        }
    }

    pub fn to_binary(&self) -> Vec<u8> {
        let buffer = Vec::<u8>::with_capacity(21_usize);
        let mut buffer_w = BufWriter::new(buffer);
        let _ = buffer_w.write(&[self.record_type as u8]);
        let _ = buffer_w.write(&self.block_length.to_le_bytes());
        let _ = buffer_w.write(&self.NE_longitude.to_le_bytes());
        let _ = buffer_w.write(&self.NE_latitude.to_le_bytes());
        let _ = buffer_w.write(&self.SW_longitude.to_le_bytes());
        let _ = buffer_w.write(&self.SW_latitude.to_le_bytes());
        buffer_w.buffer().to_vec()
    }
}

// endregion: --- Skipper record

/* region:    --- Serde POIRecord

impl Serialize for POIRecord {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let mut state = serializer.serialize_struct("POIRecord", 4)?;
        state.serialize_field("record_type",&self.record_type)?;
        state.serialize_field("record_length", &self.record_length)?;
        state.serialize_field("longitude", &self.longitude)?;
        state.serialize_field("latitude", &self.latitude)?;
        match ISO_8859_1.encode(&self.name , EncoderTrap::Strict) {
            Ok(v) => {
                let mut encoded_with_null_ending = Vec::from(v);
                encoded_with_null_ending.push(0);
                state.serialize_field("name", &encoded_with_null_ending);
            },
            Err(e) => {
                return Err(serde::ser::Error::custom("name is not ISO_8859_1 encodable"))
            }
        }
        state.end()
    }
}

// https://serde.rs/deserialize-struct.html
impl<'de> Deserialize<'de>  for POIRecord {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {

        #[derive(Deserialize)]
        #[allow(non_camel_case_types)]
        enum Field {
            record_type,
            record_length,
            longitude,
            latitude,
            name,
            value
        }

        struct POIRecordVisitor;

        impl<'de> Visitor<'de> for POIRecordVisitor {
            type Value = POIRecord;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct POIRecord")
            }
            fn visit_seq<V>(self, mut seq: V) -> core::result::Result<POIRecord, V::Error>
            where
                V: SeqAccess<'de>,
            {
                seq.next_element()
            }

        }

        // Deserialize
        const FIELDS: &[&str] = &["record_type","record_length","longitude","latitude","name","value"];
        deserializer.deserialize_struct("POIRecord", FIELDS, POIRecordVisitor)
    }
}


// endregion: --- Serde POIRecord */

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {

    use crate::ov2::poi::{POIRecord, POIRecordType};
    use binary_rw::SeekStream;

    #[test]
    fn test_create_POIRecord() {
        let r = POIRecord::new(POIRecordType::Type_2, 1000, 5000, "test".into()).unwrap();
        let bin = r.clone().to_binary(); // TODO: understand why ownership is taken ...
        assert_eq!(r.record_type, POIRecordType::Type_2);
        assert_eq!(r.record_type as u8, 2_u8);
        assert_eq!(r.longitude, 1000);
        assert_eq!(r.latitude, 5000);
        assert_eq!(r.label_vec_u8, vec![116, 101, 115, 116, 0]);
        assert_eq!(
            bin,
            vec![2, 18, 0, 0, 0, 232, 3, 0, 0, 136, 19, 0, 0, 116, 101, 115, 116, 0]
        );

        // Pour mémoire:    use std::io::{BufWriter,Write};
        // let mut buffer = [0u8; 30];
        // let mut buffer_w = BufWriter::new(buffer.as_mut());
        // buffer_w.write(&[r.record_type as u8]);
        // buffer_w.write(&r.record_length.to_be_bytes());
        // buffer_w.write(&r.longitude.to_be_bytes());
        // buffer_w.write(&r.latitude.to_be_bytes());
        // buffer_w.write(&r.value);
    }

    use binary_rw::MemoryStream;
    use encoding::all::ISO_8859_1;

    #[test]
    fn test_create_POIRecord_with_binary_rw() {
        let r = POIRecord::new(POIRecordType::Type_2, 42526, 45361, "tëst".into()).unwrap();
        println!("{r:?}");

        let mut m = MemoryStream::new();
        let mut w = binary_rw::BinaryWriter::new(&mut m, binary_rw::Endian::default());

        r.write(&mut w);
        // let _ = w.write_u8(r.record_type as u8);
        // let _ = w.write_i32(r.record_length);
        // let _ = w.write_i32(r.longitude);
        // let _ = w.write_i32(r.latitude);
        // for v in r.value {
        //     let _ = w.write_u8(v);
        // }

        // to start of memory stream
        let _ = m.seek(0);

        let mut r = binary_rw::BinaryReader::new(&mut m, binary_rw::Endian::default());
        // let t = r.read_u8().expect("Fail to read u8");
        // let s = r.read_i32().expect("Fail to read i32");
        // let lon = r.read_i32().expect("Fail to read i32");
        // let lat = r.read_i32().expect("Fail to read i32");
        // let mut m = Vec::<u8>::with_capacity((s-13) as usize);
        // for _ in 13..s {
        //     m.push(r.read_u8().expect("Fail to read u8"));
        // }
        // let msg = std::str::from_utf8(&m).expect("Invalid utf8");
        // println!("Type/Size/Lon/Lat/m : {}/{}/{}/{}/{}",t,s,lon,lat,msg);
        match POIRecord::read_from(&mut r) {
            Ok(_) => assert_eq!(true, true, "OK"),
            Err(_) => assert_eq!(false, true, "POIRecord read_from failed"),
        }
    }

    use std::ffi::CString;
    #[test]
    fn test_cstring_with_string_ending_with_null() {
        let msg = vec![116, 195, 171, 115, 116, 0]; // ending with null
        let msg_string = std::str::from_utf8(&msg).expect("Invalid utf8");
        //println!("{:?} -> {}", &msg, &msg_string);

        // to cstring
        let msg_cstring = CString::new(msg_string);
        match msg_cstring {
            Ok(_) => assert_eq!(true, false, "There should be an NulError"),
            Err(_NulError) => assert_eq!(true, true),
        }
        //println!("{:?}",msg_cstring);
    }

    use encoding::{EncoderTrap, Encoding};

    #[test]
    fn test_iso_encoding() {
        let msg = "test".to_string();
        let mut res = ISO_8859_1.encode(&msg, EncoderTrap::Strict).unwrap();
        res.push(0); // add null ending
        assert_eq!(res, vec![116, 101, 115, 116, 0]);
    }

    // use serde_test::{Token, assert_tokens};
    // #[test]
    // fn test_encoding_POIRecord() {
    //     let r = POIRecord{
    //         record_type : POIRecordType::Type_0,
    //         record_length: 18,
    //         longitude: 1,
    //         latitude: 2,
    //         name: "test".to_string(),
    //         value: Vec::new()
    //     };
    //     assert_tokens(&r, &[]);
    // }
}
