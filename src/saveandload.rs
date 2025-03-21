extern crate nav_types;
use std::{error::Error, io::BufRead};
use std::fs::File;
use core::f32;
use nav_types::WGS84;
use polars::prelude::*;
use std::io::{BufReader, BufWriter, Read, Write};

#[allow(dead_code)]
pub fn save_dataframe_to_csv(df: &DataFrame, filename: &str) -> PolarsResult<()> {
    let file = File::create(filename)?;
    let mut writer = CsvWriter::new(BufWriter::new(file));
    writer.finish(&mut df.clone())
}

pub fn save_encoding(data: &Vec<WGS84<f32>>, filename: &std::path::PathBuf) {
    let file = File::create(filename).expect("Unable to create file");
    let mut writer = BufWriter::new(file);

    for coord in data {
        let vector = vec![
            coord.latitude_degrees(),
            coord.longitude_degrees(),
            coord.altitude(),
        ];
        writer
            .write_all(bytemuck::cast_slice(&vector))
            .expect("Unable to write binary data");
    }

    writer.flush().expect("Failed to flush writer");
}

pub fn load_encoding(filename: &std::path::PathBuf) -> Vec<WGS84<f32>> {
    let file = File::open(filename).expect("Unable to open file");
    let mut reader = BufReader::new(file);

    let mut vectors = vec![];
    let mut buffer = [0f32; 3];

    while reader
        .read_exact(bytemuck::cast_slice_mut(&mut buffer))
        .is_ok()
    {
        vectors.push(WGS84::from_degrees_and_meters(
            buffer[0], buffer[1], buffer[2],
        ));
    }

    vectors
}


pub fn load_origin_coordinates(filename: &std::path::PathBuf) -> Result<WGS84<f32>, Box<dyn Error>> {
    let file = File::open(filename).expect("Unable to open file");
    let mut reader = BufReader::new(file);

    // Read the first (and only) line
    let mut line = String::new();
    reader.read_line(&mut line)?;

    let values: Vec<f32> = line
        .trim()
        .split(',')
        .map(|s| s.trim().parse::<f32>().expect("Invalid float"))
        .collect();

    let origin = WGS84::from_degrees_and_meters(values[0], values[1], values[2]);
    Ok(origin)
}
