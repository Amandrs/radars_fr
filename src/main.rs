/*
needs rust nighlty for cstring ...

https://doc.rust-lang.org/book/appendix-07-nightly-rust.html

$ rustup toolchain install nightly
$ rustup override set nightly

Not really clean and many improvments needed ...

*/

use itertools::Itertools;
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

use polars::prelude::*;
use std::{fs, io::Write, path::PathBuf};

mod error;
mod gpx;
//mod model; //future use
mod ov2;
mod radar;
use crate::ov2::POIRecord;
use crate::radar::Radar;

use clap::{Parser, Subcommand, ValueEnum};

const MAX_ENTRIES_PER_BLOCK: u64 = 20;

use crate::error::Error;
pub type Result<T> = core::result::Result<T, Error>;

// lng_min,lat_min,lng_max,lat_max
fn extract_min_max(df: &DataFrame) -> (f64, f64, f64, f64) {
    let mm = df
        .clone()
        .lazy()
        .select([
            col("lng").min().alias("lng_min"),
            col("lat").min().alias("lat_min"),
            col("lng").max().alias("lng_max"),
            col("lat").max().alias("lat_max"),
        ])
        .collect()
        .unwrap();

    mm.get(0)
        .unwrap()
        .iter()
        .map(|e| e.try_extract::<f64>().unwrap())
        .collect_tuple()
        .unwrap()
}

fn count_between_lat_lng_scaled(
    df: &DataFrame,
    sw: (f64, f64), // SW lower point (lng_min,lat_min)
    ne: (f64, f64), // NE upper point (lng_max,lat_max)
) -> u64 {
    let lf = df.clone().lazy();

    let count = lf
        .filter(
            col("lng_scaled")
                .gt_eq(lit(sw.0))
                .and(col("lng_scaled").lt_eq(lit(ne.0)))
                .and(col("lat_scaled").gt_eq(lit(sw.1)))
                .and(col("lat_scaled").lt_eq(lit(ne.1))),
        )
        .select([col("id")])
        .count()
        .collect()
        .unwrap();
    //println!("Count {}",count);
    let c: u64 = *count
        .get(0)
        .unwrap()
        .iter()
        .map(|e| e.try_extract::<u64>().unwrap())
        .collect::<Vec<u64>>()
        .first()
        .unwrap();
    c
}

fn extract_between_lat_lng_scaled(
    df: &DataFrame,
    sw: (f64, f64), // SW lower point (lng_min,lat_min)
    ne: (f64, f64), // NE upper point (lng_max,lat_max)
) -> DataFrame {
    let lf = df.clone().lazy();

    lf.filter(
        col("lng_scaled")
            .gt_eq(lit(sw.0))
            .and(col("lng_scaled").lt_eq(lit(ne.0)))
            .and(col("lat_scaled").gt_eq(lit(sw.1)))
            .and(col("lat_scaled").lt_eq(lit(ne.1))),
    )
    .select([col("id"), col("type"), col("lat"), col("lng")])
    .collect()
    .unwrap()
}

// put lat lng between [0-1]
fn scale(df: &DataFrame) -> DataFrame {
    df.clone()
        .lazy()
        .select([
            all(),
            ((col("lat") - col("lat").min()) / (col("lat").max() - col("lat").min()))
                .alias("lat_scaled"),
            ((col("lng") - col("lng").min()) / (col("lng").max() - col("lng").min()))
                .alias("lng_scaled"),
        ])
        .collect()
        .unwrap()
}

// quadtree partitioning
// https://www.gameprogrammingpatterns.com/spatial-partition.html
// https://en.wikipedia.org/wiki/Quadtree
fn quadsplit(
    threshold: u64,
    res: &mut Vec<((f64, f64), (f64, f64), u64)>,
    df: &DataFrame,
    sw: (f64, f64),
    ne: (f64, f64),
) {
    debug!("QuadT between [({:?}) ({:?})]", sw, ne);
    let nb = count_between_lat_lng_scaled(df, sw, ne);
    if nb <= threshold {
        if nb == 0 {
            debug!("Empty box [{:?},{:?}]", &sw, &ne);
        } else {
            debug!(
                "Found bounding box [{:?},{:?}] with {} elements",
                &sw, &ne, nb
            );
            res.push((sw, ne, nb));
        }
    } else {
        let delta_x = (ne.0 - sw.0) / 2.0;
        let delta_y = (ne.1 - sw.1) / 2.0;
        quadsplit(threshold, res, df, sw, (sw.0 + delta_x, sw.1 + delta_y));
        quadsplit(
            threshold,
            res,
            df,
            (sw.0 + delta_x, sw.1),
            (ne.0, sw.1 + delta_y),
        );
        quadsplit(
            threshold,
            res,
            df,
            (sw.0, sw.1 + delta_y),
            (sw.0 + delta_x, ne.1),
        );
        quadsplit(threshold, res, df, (sw.0 + delta_x, sw.1 + delta_y), ne);
    }
}

// fn from_scaled_to_unscaled(sw:(f64,f64),ne:(f64,f64),min_max:(f64,f64,f64,f64)) -> ((f64,f64),(f64,f64)) {
//     /*
//         nw      ne
//         *-------*
//         |       |
//         |       |
//         *-------*
//         sw      se
//      */
//     #[allow(non_snake_case)]
//     // lower case in [0..1]
//     // upper case in [A_min .. A_max]
//     fn unscaled_A(a:f64,A_min:f64,A_max:f64) -> f64 {
//         a*(A_max-A_min)+A_min
//     }

//     let sw_unscaled = (unscaled_A(sw.0, min_max.0, min_max.2), unscaled_A(sw.1, min_max.1, min_max.3));
//     let ne_unscaled = (unscaled_A(ne.0, min_max.0, min_max.2), unscaled_A(ne.1, min_max.1, min_max.3));

//     (sw_unscaled,ne_unscaled)
// }

// region:    --- Clap

#[derive(Parser, Debug)]
#[command(name = "radars")]
#[command(version = "1.0")]
#[command(about = "extracts radars info for your GPS", long_about = None)]
struct Cli {
    #[command(subcommand)]
    action: Actions,
}

#[derive(Subcommand, Debug)]
enum Actions {
    Get {
        #[arg(default_value = "https://radars.securite-routiere.gouv.fr/radars/all?_format=json")]
        url: url::Url,
        #[arg(short = 'o', default_value = "radars_fr.json")]
        output_file: PathBuf,
    },
    To {
        #[arg(value_enum)]
        format: Formats,
        #[arg(short = 'i', default_value = "radars_fr.json")]
        input_file: PathBuf,
        #[arg(short = 'o')]
        output_file: PathBuf,
    },
}

#[allow(clippy::upper_case_acronyms)]
#[derive(ValueEnum, Debug, Clone)]
enum Formats {
    GPX,
    OV2,
}

// endregion: --- Clap

fn get_radars_from(url: url::Url, output_file: &PathBuf) -> Result<()> {
    let all_radars = reqwest::blocking::get(url)?.text()?;

    // validate format
    let radars: Vec<Radar> = serde_json::from_str(&all_radars).expect("Bad content format");
    // println!("{c:?}");
    info!("Retrieved {} radars information", radars.len());

    // Save
    let mut file = std::fs::File::create(output_file.clone())?;
    write!(file, "{}", all_radars)?;
    info!("Saved into {:?} ", output_file);

    Ok(())
}

fn transforms_into_ov2(df: &DataFrame, output_file: PathBuf) -> Result<()> {
    // [0-1] for lat_scaled & lng_scaled
    let df = scale(df);
    //println!("{:?}",df);

    let mut split: Vec<((f64, f64), (f64, f64), u64)> = Vec::new();
    quadsplit(MAX_ENTRIES_PER_BLOCK, &mut split, &df, (0., 0.), (1., 1.));
    info!(
        "Found {} Boxes with max {} elements",
        &split.len(),
        MAX_ENTRIES_PER_BLOCK
    );

    // https://users.rust-lang.org/t/processing-polars-dataframe-row-wise/97183
    // https://www.rustexplorer.com/b#%2F*%0A%5Bdependencies%5D%0Apolars%20%3D%20%7B%20version%20%3D%20%220.30%22%2C%20features%3D%5B%22dtype-struct%22%5D%20%7D%0A*%2F%0Ause%20polars%3A%3Aprelude%3A%3A*%3B%0A%0Ause%20std%3A%3Aconvert%3A%3ATryFrom%3B%0A%0A%23%5Bderive(Debug)%5D%0Astruct%20Foo%20%7B%0A%20%20%20%20bar%3A%20String%2C%0A%20%20%20%20baz%3A%20u8%2C%0A%20%20%20%20bat%3A%20bool%2C%0A%7D%0A%0Aimpl%3C'a%3E%20TryFrom%3C%26'a%20%5BAnyValue%3C'a%3E%5D%3E%20for%20Foo%20%7B%0A%20%20%20%20type%20Error%20%3D%20()%3B%0A%0A%20%20%20%20fn%20try_from(value%3A%20%26'a%20%5BAnyValue%3C'a%3E%5D)%20-%3E%20Result%3CSelf%2C%20Self%3A%3AError%3E%20%7B%0A%20%20%20%20%20%20%20%20match%20value%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%5BAnyValue%3A%3AUtf8(bar)%2C%20baz%2C%20AnyValue%3A%3ABoolean(bat)%5D%20%3D%3E%20Ok(Foo%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20bar%3A%20(*bar).to_owned()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20baz%3A%20baz.try_extract().map_err(%7C_%7C%20())%3F%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20bat%3A%20*bat%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%7D)%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20_%20%3D%3E%20Err(())%2C%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20df%20%3D%20df!(%0A%20%20%20%20%20%20%20%20%22bar%22%20%3D%3E%20%5B%22one%22%2C%20%22two%22%2C%20%22three%22%5D%2C%0A%20%20%20%20%20%20%20%20%22baz%22%20%3D%3E%20%5B1%2C%202%2C%203%5D%2C%0A%20%20%20%20%20%20%20%20%22bat%22%20%3D%3E%20%5Btrue%2C%20false%2C%20false%5D%2C%0A%20%20%20%20)%0A%20%20%20%20.unwrap()%3B%0A%0A%20%20%20%20let%20v%3A%20Result%3CVec%3CFoo%3E%2C%20()%3E%20%3D%20df%0A%20%20%20%20%20%20%20%20.into_struct(%22don't%20know%20what%20this%20argument%20is%20for%22)%0A%20%20%20%20%20%20%20%20.into_iter()%0A%20%20%20%20%20%20%20%20.map(%7Cs%7C%20Foo%3A%3Atry_from(s))%0A%20%20%20%20%20%20%20%20.collect()%3B%0A%0A%20%20%20%20println!(%22%7Bv%3A%3F%7D%22)%3B%0A%7D

    let mut nb_poi = 0;
    let mut nb_skipper = 0;

    // all in memory
    let mut super_block: Vec<u8> = Vec::new();

    for (sw, ne, nb) in split.iter() {
        let sub_box = extract_between_lat_lng_scaled(&df, *sw, *ne);

        // extract min, max from sub_box to get closer sw ne geopoints
        let box_min_max = extract_min_max(&sub_box);
        let box_sw = (box_min_max.0, box_min_max.1);
        let box_ne = (box_min_max.2, box_min_max.3);

        //let (SW,NE) = from_scaled_to_unscaled(*sw, *ne, min_max);
        //info!("Box [{:?} {:?}] -> [{:?} {:?}] {} with {:?}",sw,ne,SW,NE,nb,sub_box);
        //debug!("Box [{:?} {:?}] -> [{:?} {:?}] -> [{:?} {:?}] {}", sw, ne, SW, NE, box_SW, box_NE, nb);
        debug!(
            "Box [{:?} {:?}] -> [{:?} {:?}] {}",
            sw, ne, box_sw, box_ne, nb
        );
        let records: std::result::Result<Vec<POIRecord>,()> = sub_box.into_struct("radar")
            .into_iter()
            .map( |s| {
                match s {
                    [AnyValue::String(name), AnyValue::String(_), AnyValue::Float64(lat), AnyValue::Float64(lng)] => {
                        //println!("Found {} {} {}",*name,lat,lng); 
                        return Ok(
                            ov2::POIRecordBuilder::default()
                                .label(String::from(*name))
                                .latitude(*lat as f32)
                                .longitude(*lng as f32)
                                .build()
                                .unwrap()
                        );
                    },
                    _ => { println!("Error in {:?}",s); Err(()) }
                }

            })
            .collect();

        // Create block with skipper
        let mut sub_block: Vec<u8> = Vec::new();
        for r in records.unwrap() {
            sub_block.append(&mut r.to_binary());
            nb_poi += 1;
        }
        //println!("sub_block len {} : {:?}",sub_block.len(),sub_block);
        let s = ov2::SkipperRecord::new(sub_block.len(), box_sw, box_ne);
        let mut block = Vec::with_capacity(21 + sub_block.len());
        block.append(&mut s.to_binary());
        block.append(&mut sub_block);
        nb_skipper += 1;

        //println!("Block ({}) {:?}",block.len(),block);
        super_block.append(&mut block);
    }

    match std::fs::write(output_file.clone(), super_block) {
        Ok(_) => info!("OV2 File written {:?}", output_file),
        _ => error!("Error in writting file {:?}", output_file),
    }

    info!("Wrote {} POI(s) with {} SkipperRecords", nb_poi, nb_skipper);
    Ok(())
}

fn transforms_into_gpx(df: &DataFrame, output_file: PathBuf) -> Result<()> {
    let res = gpx::save_waypoints(df)?;

    match std::fs::write(output_file.clone(), res) {
        Ok(_) => info!("GPX File written {:?}", output_file),
        _ => error!("Error in writting file {:?}", output_file),
    }

    Ok(())
}

fn transforms_into(format: Formats, input_file: PathBuf, output_file: PathBuf) -> Result<()> {
    info!("Starting ...");

    let file = fs::File::open(input_file).map_err(|e| Error::StdIO(e.to_string()))?;

    let df = JsonReader::new(file).finish()?;

    match format {
        Formats::OV2 => transforms_into_ov2(&df, output_file),
        Formats::GPX => transforms_into_gpx(&df, output_file),
    }
}

//#[tokio::main]
#[allow(non_snake_case)]
//async fn main() -> Result<()> {
fn main() -> Result<()> {
    // tracing
    // Change log level with RUST_LOG env variable
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::fmt()
        .without_time() // TODO: remove
        .with_target(false) // TODO: remove
        .with_env_filter(filter_layer)
        .init();

    // region:    --- Clap
    let cli = Cli::parse();

    match cli.action {
        Actions::Get { url, output_file } => {
            info!("HTTP GET {} -> {:?}", url.as_str(), output_file.as_os_str());
            match get_radars_from(url, &output_file) {
                Ok(_) => std::process::exit(0),
                Err(e) => {
                    error!("{:?}", e);
                    std::process::exit(-1);
                }
            }
        }
        Actions::To {
            format,
            input_file,
            output_file,
        } => {
            info!(
                "Tranforms {:?} into {:?} (format: {:?})",
                input_file.as_os_str(),
                output_file.as_os_str(),
                format
            );
            transforms_into(format, input_file, output_file)?;
        }
    }

    // endregion: --- Clap

    Ok(())
}

// #[cfg(test)]
// #[allow(non_snake_case)]
// mod tests {
//     use crate::from_scaled_to_unscaled;

//     #[test]
//     fn test_from_scaled_full_box() {
//         let min_max = ( -20.,-10.,30.,20. );
//         let sw = (0.,0.);
//         let ne = (1.,1.);
//         let (SW,NE) = from_scaled_to_unscaled(sw, ne, min_max);
//         assert_eq!(SW,(-20.,-10.));
//         assert_eq!(NE,(30.,20.));
//     }

//     #[test]
//     fn test_from_scaled_smaller_box() {
//         let min_max = ( -20.,-10.,30.,20. );
//         let sw = (0.5,0.5);
//         let ne = (0.5+1./50.,0.5+1./30.);
//         let (SW,NE) = from_scaled_to_unscaled(sw, ne, min_max);
//         assert_eq!(SW,(5.,5.));
//         assert_eq!(NE,(6.,6.));
//     }
// }
