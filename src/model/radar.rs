use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;
use serde_json::Value;

use crate::model::{Error, Result};
use chrono::{DateTime, Utc};
/*
//use geo_types::Point;
//use wkt::TryFromWkt;

https://stackoverflow.com/questions/61831962/deserializing-a-datetime-from-a-string-millisecond-timestamp-with-serde
#[serde_as(as= "TimestampSeconds<String,Flexible>")]

https://stackoverflow.com/questions/75527167/serde-deserialize-string-into-u64
use serde_with::formats::Flexible;
use serde_with::DisplayFromStr;
#[serde_as(as = "DisplayFromStr")]

https://docs.rs/serde-aux/latest/serde_aux/field_attributes/index.html

//use serde_with::TimestampSeconds;
// */

/* Radar types

- discriminants : speed distinction between PoidLourd & VehiculeLéger
    rulesmesured' - vitesse_vl / vitesse_pl
- fixes : vitesse_vl
- feux: semaphor
- itinéraires: some radars for 'radartronconkm' kms ...
- niveaux: railroads crossing pn_infraction
- troncons: mean speed on radartronconkm

// */

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RadarType {
    #[serde(rename = "macinename")]
    pub name: String,
    pub radarnamedetails: String,
    #[serde(deserialize_with = "deserialize_datetime_utc_from_seconds")]
    pub changed: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleMeasured {
    #[serde(rename = "macinename")]
    pub name: String,
    #[serde(deserialize_with = "deserialize_datetime_utc_from_seconds")]
    pub changed: DateTime<Utc>,
    pub rulelabel: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Radar {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub nid: u64,
    pub title: String,
    #[serde(deserialize_with = "deserialize_datetime_utc_from_seconds")]
    pub created: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_datetime_utc_from_seconds")]
    pub changed: DateTime<Utc>,
    pub department: String,
    pub radardirection: String,
    pub radarequipment: String,
    pub radarroad: String,
    #[serde(deserialize_with = "wkt::deserialize_wkt")]
    pub radargeolocalisation: geo_types::Point,
    pub radartype: Vec<RadarType>,
    pub rulesmesured: Vec<RuleMeasured>,
    #[serde(default, deserialize_with = "deserialize_optional_f64")]
    pub radartronconkm: Option<f64>,
}

// https://github.com/serde-rs/json/issues/317
fn deserialize_optional_f64<'de, D>(de: D) -> core::result::Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let deser_res: Value = serde::Deserialize::deserialize(de)?;
    //println!("Deserialize option<f64> Value {:?}",deser_res);
    match deser_res {
        Value::Number(v) => Ok(v.as_f64()),
        Value::String(v) => Ok(serde_json::from_str(&v).map_err(|e| {
            serde::de::Error::custom(format!("could not deserialize to optional f64: {}", e))
        })?),
        _ => Ok(None),
    }
}

impl Radar {
    fn create_from_str(content: &str) -> Result<Radar> {
        serde_json::from_str(content).map_err(|e| Error::FailToDeserializeRadar(e.to_string()))
    }
}

// fn from_timestamp_str_to_datetime_utc(tm_str:String) -> Result<DateTime<Utc>> {

//     let i = tm_str.parse::<i64>().expect("invalid i64");
//     let datetime = DateTime::from_timestamp(i,0).expect("invalid timestamp");

//     Ok(datetime)

// }

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {

    use crate::model::error::{Error, Result};
    use crate::model::radar::Radar;
    use serde::Deserialize;
    use serde_json::Value;

    #[test]
    fn test_json_parse_string_to_u64() {
        let c = r#"
        {
            "nid": "23595"
        }
        "#;
        let j: Value = serde_json::from_str(c).expect("Error JSON parsing {err:?}");
        let nb: Result<u64> = match &j["nid"] {
            Value::Number(nb) => nb
                .as_u64()
                .ok_or(Error::JsonError("Can not convert to u64".into())),
            Value::String(nb_str) => nb_str
                .parse::<u64>()
                .map_err(|e| Error::JsonError(e.to_string())),
            _ => Err(Error::JsonError("Can not convert to u64".into())),
        };

        assert_eq!(nb.unwrap(), 23595);
    }

    #[test]
    fn test_radar_fixe_from_str() {
        let c = r#"
        {
            "nid": "23595",
            "uuid": "0920d3b5-63fb-47eb-b820-fa1533d2814a",
            "vid": "23595",
            "langcode": "fr",
            "type": [
                {
                    "target_id": "radar"
                }
            ],
            "revisiontimestamp": "1681740362",
            "revisionuid": [
                {
                    "target_id": "1"
                }
            ],
            "revisionlog": [],
            "status": "1",
            "uid": [
                {
                    "target_id": "1"
                }
            ],
            "title": "130",
            "created": "1681740362",
            "changed": "1681740362",
            "promote": "0",
            "sticky": "0",
            "defaultlangcode": "1",
            "revisiontranslationaffected": "1",
            "path": [
                {
                    "langcode": "fr"
                }
            ],
            "department": "64-PYRENEES-ATLANTIQUES",
            "itineraireentree": [],
            "itinerairesortie": [],
            "radardirection": "BAYONNE VERS LANDES",
            "radarequipment": "MORPHO",
            "radargeolocalisation": "POINT (-1.46733 43.5135)",
            "radarinstalldate": "2003-12-29T16:06:04",
            "radarplace": [],
            "radarroad": "RD810",
            "radartronconkm": [],
            "radartype": [
                {
                    "tid": "2",
                    "uuid": "edeb3129-7171-4f20-9fdc-1a19e24d1726",
                    "revisionid": "2",
                    "langcode": "fr",
                    "vid": [
                        {
                            "target_id": "radar_types"
                        }
                    ],
                    "revisioncreated": [],
                    "revisionuser": [],
                    "revisionlogmessage": [],
                    "status": "1",
                    "name": "Fixes",
                    "description": "C'est le premier type de radar \u00e0 avoir \u00e9t\u00e9 install\u00e9. Il calcule la vitesse du v\u00e9hicule \u00e0 son passage instantan\u00e9ment. \r\n<br>\r\n<br>\r\n<a href=\"http://www.securite-routiere.gouv.fr/dangers-de-la-route/vitesse\" target=\"_blank\">En savoir plus sur les dangers de la vitesse</a>\r\n\r\n",
                    "weight": "0",
                    "parent": [
                        {
                            "target_id": "0"
                        }
                    ],
                    "changed": "1642158442",
                    "defaultlangcode": "1",
                    "revisiontranslationaffected": "1",
                    "path": [
                        {
                            "langcode": "fr"
                        }
                    ],
                    "csvvalue": [
                        {
                            "value": "Equipement Terrain Fixe"
                        },
                        {
                            "value": "Equipement Terrain Tourelle "
                        }
                    ],
                    "illustration": [
                        {
                            "styles": {
                                "large": {
                                    "url": "/sites/default/files/styles/large/public/images/radars/fixe-v2.png?itok=DHZKcwlK",
                                    "height": "129",
                                    "width": "89"
                                },
                                "medium": {
                                    "url": "/sites/default/files/styles/medium/public/images/radars/fixe-v2.png?itok=SBZGx6GL",
                                    "height": "129",
                                    "width": "89"
                                },
                                "thumbnail": {
                                    "url": "/sites/default/files/styles/thumbnail/public/images/radars/fixe-v2.png?itok=QWWSxxr6",
                                    "height": 100,
                                    "width": 69
                                }
                            },
                            "infos": {
                                "fileName": "fixe-v2.png",
                                "fileCreated": "1516024549",
                                "fileSize": "11450",
                                "fileExtension": "png"
                            }
                        }
                    ],
                    "macinename": "fixes",
                    "radarnamedetails": "Radar fixe"
                }
            ],
            "rulesmesured": [
                {
                    "tid": "7",
                    "uuid": "c447707a-c428-4166-ac14-c667a8cdcb41",
                    "revisionid": "7",
                    "langcode": "fr",
                    "vid": [
                        {
                            "target_id": "rules"
                        }
                    ],
                    "revisioncreated": [],
                    "revisionuser": [],
                    "revisionlogmessage": [],
                    "status": "1",
                    "name": "Vitesse VL 80",
                    "description": [
                        {
                            "value": null,
                            "format": null
                        }
                    ],
                    "weight": "0",
                    "parent": [
                        {
                            "target_id": "0"
                        }
                    ],
                    "changed": "1515154470",
                    "defaultlangcode": "1",
                    "revisiontranslationaffected": "1",
                    "path": [
                        {
                            "langcode": "fr"
                        }
                    ],
                    "illustration": [
                        {
                            "styles": {
                                "large": {
                                    "url": "/sites/default/files/styles/large/public/images/regles/vitesse-80.png?itok=TcuP772z",
                                    "height": "70",
                                    "width": "70"
                                },
                                "medium": {
                                    "url": "/sites/default/files/styles/medium/public/images/regles/vitesse-80.png?itok=TK2yJ19C",
                                    "height": "70",
                                    "width": "70"
                                },
                                "thumbnail": {
                                    "url": "/sites/default/files/styles/thumbnail/public/images/regles/vitesse-80.png?itok=thPluA0X",
                                    "height": "70",
                                    "width": "70"
                                }
                            },
                            "infos": {
                                "fileName": "vitesse-80.png",
                                "fileCreated": "1515154465",
                                "fileSize": "4045",
                                "fileExtension": "png"
                            }
                        }
                    ],
                    "macinename": "vitesse_vl_80",
                    "rulelabel": "Vitesse"
                }
            ],
            "traceitineraire": []
        }
        "#;

        match Radar::create_from_str(c) {
            Ok(r) => {
                println!("Radar {:?}", r);
                assert_eq!(true, true)
            }
            _ => assert_eq!(false, true, "Should be a valid radar"),
        };
    }

    #[test]
    fn test_fail_radar_fixe_from_str() {
        let c = r#"
        {
            "nid": "2359A",
            "uuid": "0920d3b5-63fb-47eb-b820-fa1533d2814a",
            "vid": "23595",
        }
        "#;

        match Radar::create_from_str(c) {
            Ok(_) => assert_eq!(false, true, "This should not happen"),
            Err(Error::FailToDeserializeRadar(_)) => assert_eq!(
                true, true,
                "This should fail as model nid is bad format (and model incomplete)"
            ),
            Err(Error::JsonError(_)) => assert_eq!(false, true, "?"),
        };
    }

    #[test]
    fn test_deserialize_optional_f64_from_String() {
        use crate::model::radar::deserialize_optional_f64;

        #[derive(Debug, Deserialize)]
        struct A {
            #[serde(default, deserialize_with = "deserialize_optional_f64")]
            n: Option<f64>,
        }

        let c = r#"{
            "n": "3.2"
        }"#;
        let r: A = serde_json::from_str(c).unwrap();
        assert_eq!(r.n.unwrap(), 3.2);
    }

    #[test]
    fn test_deserialize_optional_f64_from_Number() {
        use crate::model::radar::deserialize_optional_f64;

        #[derive(Debug, Deserialize)]
        struct A {
            #[serde(default, deserialize_with = "deserialize_optional_f64")]
            n: Option<f64>,
        }

        let c = r#"{
            "n": 3.2
        }"#;
        let r: A = serde_json::from_str(c).unwrap();
        assert_eq!(r.n.unwrap(), 3.2);
    }

    #[test]
    fn test_deserialize_optional_f64_from_blob() {
        use crate::model::radar::deserialize_optional_f64;

        #[derive(Debug, Deserialize)]
        struct A {
            #[serde(default, deserialize_with = "deserialize_optional_f64")]
            n: Option<f64>,
        }

        let c = r#"{
            "n": "err"
        }"#;
        match serde_json::from_str::<A>(c) {
            Ok(_) => assert_eq!(true, false, "This should not happen"),
            Err(_) => assert_eq!(true, true, "deserialization to f64 should fail"),
        };
    }
}
