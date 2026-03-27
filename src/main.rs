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

    /// Decode a .coords file back to the original file
    #[arg(short, long, default_value_t = false)]
    decode: bool,

    /// If to print the encoded string to the CLI
    #[arg( long, default_value_t = false)]
    output_cli: bool,

    /// Generate a PNG map plot of the encoded coordinates
    #[arg(long, default_value_t = false)]
    plot: bool
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

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

    if args.decode {
        // Decode mode: read .coords file, decode, write output
        let loaded = saveandload::load_encoding(&args.input_path);
        match coords::decode_with_metadata(origin, loaded) {
            Ok((bytes, filename)) => {
                fs::write(&args.output_path, &bytes)?;
                println!("Decoded '{}' ({} bytes) to {}", filename, bytes.len(), args.output_path.display());
            }
            Err(e) => eprintln!("Decode failed: {}", e),
        }
    } else {
        // Encode mode: read any file, encode with metadata, save
        let input_contents = fs::read(&args.input_path).expect("Should have been able to read the file");
        let filename = args.input_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let encoded = coords::encode_with_metadata(origin, &input_contents, &filename);

        if args.output_cli {
            log_encoding(encoded.clone());
        }

        saveandload::save_encoding(&encoded, &args.output_path);

        if args.plot {
            saveandload::generate_plot(&encoded, &args.output_path);
        }

        // Verification round-trip
        let loaded = saveandload::load_encoding(&args.output_path);
        match coords::decode_with_metadata(origin, loaded) {
            Ok((bytes, _)) => println!("Succeeded: {:?}", input_contents == bytes),
            Err(e) => println!("Verification failed: {}", e),
        }
    }

    Ok(())
}
