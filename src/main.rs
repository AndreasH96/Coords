pub mod saveandload;
extern crate nav_types;

use clap::Parser;
use coords::log_encoding;
use regex::Regex;
use nav_types::WGS84;
use std::fs;

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
    coords: Option<String>,

    /// If to encode, otherwise decode
    #[arg(short, long, default_value_t = true)]
    encode: bool,

    /// If to print the encoded string to the CLI
    #[arg( long, default_value_t = false)]
    output_cli: bool,

    /// Generate an HTML map plot of the encoded coordinates and open it in the browser
    #[arg(long, default_value_t = false)]
    plot: bool
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let input_contents =
        fs::read_to_string(args.input_path).expect("Should have been able to read the file");



    // Load origin
    let origin: WGS84<f64>;
    if args.coords_path.is_some(){
        origin= saveandload::load_origin_coordinates(&args.coords_path.unwrap()).unwrap();
    }
    else if args.coords.is_some() {
        let re: Regex = Regex::new(r"[\[\]\(\)\{\}]+").unwrap();
        let coords_string_raw = args.coords.unwrap();
        let coords_string = Regex::replace_all(&re,&coords_string_raw, "");
        let coords_vec: Vec<f64> = coords_string
        .split(',')
        .map(|s| s.trim().parse::<f64>().expect("Invalid float in --coords"))
        .collect();
        origin = WGS84::from_degrees_and_meters(coords_vec[0], coords_vec[1], coords_vec[2])
    }
    else {
        panic!("No origin given!")
    }
    

    let text_hex = hex::encode(&input_contents);
    let encoded = coords::encode(origin, &text_hex);

    if args.output_cli {
        log_encoding(encoded.clone());
    }

    saveandload::save_encoding(&encoded, &args.output_path);

    if args.plot {
        saveandload::generate_plot(&encoded, &args.output_path);
    }

    let loaded = saveandload::load_encoding(&args.output_path);
    let decoded = coords::decode(origin, loaded);

    println!("Succeded: {:?}", input_contents == decoded);

    

    Ok(())
}
