# RADARS FRENCH

Retrieve and export French radars to OV2/GPX formats

License : [Gnu GPL V3](LICENSE)

# Pre-requists

have a rust nightly build installed

```
> rustup toolchain install nightly
```

# Install

```
// retrieve code
> git clone ...
// go the 
> cd radars_fr
// make rust nightly the default compilation environment for this project
radars_fr> rustup override set nightly
radars_fr> cargo build --release
```

# Help

```
radars_fr> ./target/release/radars 
extracts radars info for your GPS

Usage: radars <COMMAND>

Commands:
  get   
  to    
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```


# Usage

## Retrieve all french radars

```
radars_fr> ./target/release/radars get   
 INFO HTTP GET https://...securite-routiere... -> "radars_fr.json"
 INFO Retrieved 3204 radars information
 INFO Saved into "radars_fr.json" 
```

## Export to OV2 or GPX format


```
radars_fr> ./target/release/radars to 
Usage: radars to [OPTIONS] -o <OUTPUT_FILE> <FORMAT>

Arguments:
  <FORMAT>  [possible values: gpx, ov2]

Options:
  -i <INPUT_FILE>       [default: radars_fr.json]
  -o <OUTPUT_FILE>
  -h, --help            Print help
```

### OV2

OV2 file is a specific TOMTOM POI gps file format.
This produces an optimised version of the OV2 file.

Thanks to [gordthompson](https://gordthompson.github.io/ov2optimizer/) for the specifications and idea.

```
radars_fr> ./target/release/radars to -o radars_fr.ov2 ov2
 INFO Tranforms "radars_fr.json" into "radars_fr.ov2" (format: OV2)
 INFO Starting ...
 INFO Found 384 Boxes with max 20 elements
 INFO OV2 File written "radars_fr.ov2"
 INFO Wrote 3204 POI(s) with 384 SkipperRecords
```



### GPX

to use with Openstreetmap (and others ...)

```
radars_fr> ./target/release/radars to -o radars_fr.gpx gpx
 INFO Tranforms "radars_fr.json" into "radars_fr.gpx" (format: GPX)
 INFO Starting ...
 INFO GPX File written "radars_fr.gpx"
```


ENJOY !
