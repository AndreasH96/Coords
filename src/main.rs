pub mod saveandload;
extern crate nav_types;

use clap::Parser;
use core::f32;
use dashmap::DashMap;
use hex;
use lazy_static::lazy_static;
use nav_types::WGS84;
use std::fs;
use std::time::Instant;

lazy_static! {
    static ref CACHE: DashMap<i32, WGS84<f32>> = DashMap::new();
}

/// Encode the content of a file into the Coords format, or decode back to original file content
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the file to read
    #[arg(short, long)]
    input_path: std::path::PathBuf,
    /// The destination path
    #[arg(short, long)]
    output_path: std::path::PathBuf,

    /// (Optional) Path to coordinate keys stored in a file
    #[arg(short('C'), long, default_value = None)]
    coords_path: Option<std::path::PathBuf>,

    /// (Optional) Coordinate keys (latitude,longitude, altitude)
    #[arg(short, long, default_value = None)]
    coords: Vec<f32>,

    /// If to encode, otherwise decode
    #[arg(short, long, default_value_t = true)]
    encode: bool,
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let input_contents =
        fs::read_to_string(args.input_path).expect("Should have been able to read the file");

    let start_time = Instant::now();
    let origin: WGS84<f32> = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);

    let text_hex = hex::encode(&input_contents);
    let encoded = coords::encode(origin, &text_hex);

    saveandload::save_encoding(&encoded, &args.output_path);

    let loaded = saveandload::load_encoding(&args.output_path);
    println!("\nRestoring to text");
    let decoded = coords::decode(origin, encoded);

    println!("Succeded: {:?}", input_contents == decoded);
    let duration = start_time.elapsed();

    println!("Time elapsed in expensive_function() is: {:?}", duration);

    Ok(())
}
