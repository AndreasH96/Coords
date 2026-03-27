extern crate nav_types;
use std::{error::Error, io::BufRead};
use std::fs::File;
use nav_types::WGS84;
use polars::prelude::*;
use std::io::{BufReader, BufWriter, Read, Write};

#[allow(dead_code)]
pub fn save_dataframe_to_csv(df: &DataFrame, filename: &str) -> PolarsResult<()> {
    let file = File::create(filename)?;
    let mut writer = CsvWriter::new(BufWriter::new(file));
    writer.finish(&mut df.clone())
}

const COORD_SCALE: f64 = 1e8;

pub fn save_encoding(data: &Vec<WGS84<f64>>, filename: &std::path::PathBuf) {
    let file = File::create(filename).expect("Unable to create file");
    let mut writer = BufWriter::new(file);

    for coord in data {
        let lat = (coord.latitude_degrees() * COORD_SCALE).round() as i64;
        let lon = (coord.longitude_degrees() * COORD_SCALE).round() as i64;
        writer
            .write_all(bytemuck::cast_slice(&[lat, lon]))
            .expect("Unable to write binary data");
    }

    writer.flush().expect("Failed to flush writer");
}

pub fn load_encoding(filename: &std::path::PathBuf) -> Vec<WGS84<f64>> {
    let file = File::open(filename).expect("Unable to open file");
    let mut reader = BufReader::new(file);

    let mut vectors = vec![];
    let mut buffer = [0i64; 2];

    while reader
        .read_exact(bytemuck::cast_slice_mut(&mut buffer))
        .is_ok()
    {
        let lat = buffer[0] as f64 / COORD_SCALE;
        let lon = buffer[1] as f64 / COORD_SCALE;
        vectors.push(WGS84::from_degrees_and_meters(lat, lon, 0.0));
    }

    vectors
}

pub fn load_origin_coordinates(filename: &std::path::PathBuf) -> Result<WGS84<f64>, Box<dyn Error>> {
    let file = File::open(filename).expect("Unable to open file");
    let mut reader = BufReader::new(file);

    let mut line = String::new();
    reader.read_line(&mut line)?;

    let values: Vec<f64> = line
        .trim()
        .split(',')
        .map(|s| s.trim().parse::<f64>().expect("Invalid float"))
        .collect();

    let origin = WGS84::from_degrees_and_meters(values[0], values[1], values[2]);
    Ok(origin)
}
