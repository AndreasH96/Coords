extern crate nav_types;

use dashmap::DashMap;
use hex;
use lazy_static::lazy_static;
use nav_types::WGS84;
use rand::Rng;
use rayon::prelude::*;

const EARTH_RADIUS: f64 = 6_371_000.0;
const CHUNK_HEX_WIDTH: usize = 6; // 6 hex chars = 3 bytes per chunk

lazy_static! {
    static ref CACHE: DashMap<i32, WGS84<f64>> = DashMap::new();
}

/// Compute a destination point on a sphere given origin, bearing, and distance.
fn haversine_destination(origin: &WGS84<f64>, bearing_rad: f64, distance_m: f64) -> WGS84<f64> {
    let lat1 = origin.latitude_degrees().to_radians();
    let lon1 = origin.longitude_degrees().to_radians();
    let d = distance_m / EARTH_RADIUS;

    let lat2 = (lat1.sin() * d.cos() + lat1.cos() * d.sin() * bearing_rad.cos()).asin();
    let lon2 = lon1
        + (bearing_rad.sin() * d.sin() * lat1.cos()).atan2(d.cos() - lat1.sin() * lat2.sin());

    // Wrap longitude to [-180, 180]
    let mut lon_deg = lon2.to_degrees();
    lon_deg = ((lon_deg + 180.0) % 360.0 + 360.0) % 360.0 - 180.0;

    WGS84::from_degrees_and_meters(lat2.to_degrees(), lon_deg, 0.0)
}

pub fn compute_coordinate(origin: WGS84<f64>, target_distance: i32) -> WGS84<f64> {
    if target_distance == 0 {
        return origin;
    }

    let mut rng = rand::thread_rng();

    loop {
        let bearing = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
        let mut guess_distance = target_distance as f64;
        let mut prev_error = i32::MAX;

        for _ in 0..100 {
            let point = haversine_destination(&origin, bearing, guess_distance);
            let actual = origin.distance(&point).floor() as i32;
            let error = target_distance - actual;

            if error == 0 {
                return point;
            }

            // Oscillating — jump a random distance to escape
            if error == -prev_error {
                guess_distance += rng.gen_range(2.0..5.0) * error as f64;
            } else {
                guess_distance += error as f64;
            }

            prev_error = error;
        }
        // Retry with a different bearing
    }
}

pub fn generate_coordinate_for_distance(origin: WGS84<f64>, distance: i32) -> WGS84<f64> {
    if let Some(entry) = CACHE.get(&distance) {
        return *entry.value();
    }

    let coordinate = compute_coordinate(origin, distance);
    CACHE.insert(distance, coordinate);
    coordinate
}

pub fn encode(origin: WGS84<f64>, text: &str) -> Vec<WGS84<f64>> {
    let chunks: Vec<String> = text
        .chars()
        .collect::<Vec<_>>()
        .chunks(CHUNK_HEX_WIDTH)
        .map(|c| c.iter().collect::<String>())
        .collect();

    chunks
        .par_iter()
        .filter_map(|chunk| match i32::from_str_radix(chunk, 16) {
            Ok(value) => Some(generate_coordinate_for_distance(origin, value)),
            Err(_e) => None,
        })
        .collect()
}

pub fn decode(origin: WGS84<f64>, encoded: Vec<WGS84<f64>>) -> String {
    let mut strings: Vec<String> = vec![];

    for (i, cord) in encoded.iter().enumerate() {
        let val = origin.distance(cord).floor() as i32;
        // Last chunk may have fewer hex digits
        if i == encoded.len() - 1 {
            // Determine actual hex width of last chunk by checking leading zeros
            let hex = format!("{:0width$x}", val, width = CHUNK_HEX_WIDTH);
            let trimmed = hex.trim_start_matches('0');
            // Must be even length for valid hex byte pairs
            let len = if trimmed.is_empty() {
                2
            } else if trimmed.len() % 2 != 0 {
                trimmed.len() + 1
            } else {
                trimmed.len()
            };
            strings.push(format!("{:0width$x}", val, width = len));
        } else {
            strings.push(format!("{:0width$x}", val, width = CHUNK_HEX_WIDTH));
        }
    }
    let hex_string: String = strings.join("");

    match hex::decode(&hex_string) {
        Ok(decoded_bytes) => match String::from_utf8(decoded_bytes.clone()) {
            Ok(text) => text,
            Err(_e) => String::from("Failed to parse, invalid UTF-8!"),
        },
        Err(_e) => String::from("Failed to decode hex string!"),
    }
}

pub fn log_encoding(encoded: Vec<WGS84<f64>>) {
    for cord in encoded.iter() {
        println!(
            "({}, {}, {})",
            cord.latitude_degrees(),
            cord.longitude_degrees(),
            cord.altitude()
        );
    }
}