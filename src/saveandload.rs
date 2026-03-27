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

pub fn generate_plot(data: &[WGS84<f64>], output_path: &std::path::PathBuf) {
    use plotters::prelude::*;
    use plotters::style::RGBColor;

    let png_path = output_path.with_extension("png");
    let point_color = RGBColor(220, 50, 50);

    // Load the world map image
    let map_bytes = include_bytes!("../assets/world_map.jpg");
    let map_img = image::load_from_memory(map_bytes).expect("Failed to load world map");
    let map_rgb = map_img.to_rgb8();
    let (map_w, map_h) = map_rgb.dimensions();

    // Create output image buffer starting with the world map
    let mut buf = vec![0u8; (map_w * map_h * 3) as usize];
    buf.copy_from_slice(&map_rgb.into_raw());

    {
        let root = BitMapBackend::with_buffer(&mut buf, (map_w, map_h)).into_drawing_area();

        let mut chart = ChartBuilder::on(&root)
            .build_cartesian_2d(-180.0f64..180.0f64, -90.0f64..90.0f64)
            .unwrap();

        chart.configure_mesh().disable_mesh().draw().unwrap();

        chart
            .draw_series(data.iter().map(|c| {
                Circle::new(
                    (c.longitude_degrees(), c.latitude_degrees()),
                    4,
                    point_color.filled(),
                )
            }))
            .unwrap();

        root.present().unwrap();
    }

    let out_img = image::RgbImage::from_raw(map_w, map_h, buf).unwrap();
    out_img.save(&png_path).expect("Failed to save plot");

    println!("Plot saved to: {}", png_path.display());

    std::process::Command::new("open")
        .arg(&png_path)
        .spawn()
        .ok();
}
