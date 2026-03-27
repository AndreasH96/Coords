extern crate nav_types;

use hex;
use nav_types::WGS84;
use rand::Rng;

const EARTH_RADIUS: f64 = 6_371_000.0;
const CHUNK_HEX_WIDTH: usize = 6; // 6 hex chars = 3 bytes per chunk
const COORD_SCALE: f64 = 1e8;

/// Simulate save/load quantization: round lat/lon to COORD_SCALE precision.
fn quantize(coord: &WGS84<f64>) -> WGS84<f64> {
    let lat = (coord.latitude_degrees() * COORD_SCALE).round() / COORD_SCALE;
    let lon = (coord.longitude_degrees() * COORD_SCALE).round() / COORD_SCALE;
    WGS84::from_degrees_and_meters(lat, lon, 0.0)
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

/// Compute a coordinate at exactly target_distance (in meters) from origin,
/// ensuring the result survives save/load quantization.
pub fn compute_coordinate(origin: &WGS84<f64>, target_distance: i32) -> WGS84<f64> {
    if target_distance == 0 {
        return *origin;
    }

    let mut rng = rand::thread_rng();

    loop {
        let bearing = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
        let mut guess_distance = target_distance as f64;
        let mut prev_error = i32::MAX;

        for _ in 0..100 {
            let point = haversine_destination(origin, bearing, guess_distance);
            let actual = origin.distance(&point).round() as i32;
            let error = target_distance - actual;

            if error == 0 {
                // Verify it survives quantization
                let quantized = quantize(&point);
                let quantized_distance = origin.distance(&quantized).round() as i32;
                if quantized_distance == target_distance {
                    return quantized;
                }
                // Not safe — nudge with small random steps until safe
                let mut nudged = point;
                for _ in 0..50 {
                    let nudge_bearing = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
                    let nudge_dist = rng.gen_range(0.1..0.4);
                    nudged = haversine_destination(&nudged, nudge_bearing, nudge_dist);
                    let q = quantize(&nudged);
                    if origin.distance(&q).round() as i32 == target_distance {
                        return q;
                    }
                }
                // Nudging failed, retry with new bearing
                break;
            }

            if error == -prev_error {
                guess_distance += rng.gen_range(2.0..5.0) * error as f64;
            } else {
                guess_distance += error as f64;
            }

            prev_error = error;
        }
    }
}

/// Generate a random coordinate on the globe.
fn random_coordinate() -> WGS84<f64> {
    let mut rng = rand::thread_rng();
    let lat = rng.gen_range(-80.0..80.0);
    let lon = rng.gen_range(-180.0..180.0);
    quantize(&WGS84::from_degrees_and_meters(lat, lon, 0.0))
}

/// Encode text as a chain of coordinates.
/// Output: [P0_random, P1, P2, ..., Pn, origin]
/// where distance(Pi, Pi+1) encodes chunk[i], and origin is appended at the end.
pub fn encode(origin: WGS84<f64>, text: &str) -> Vec<WGS84<f64>> {
    let chunks: Vec<String> = text
        .chars()
        .collect::<Vec<_>>()
        .chunks(CHUNK_HEX_WIDTH)
        .map(|c| c.iter().collect::<String>())
        .collect();

    let mut result: Vec<WGS84<f64>> = Vec::with_capacity(chunks.len() + 2);
    let mut current = random_coordinate();
    result.push(current);

    for chunk in &chunks {
        match i32::from_str_radix(chunk, 16) {
            Ok(value) => {
                let next = compute_coordinate(&current, value);
                result.push(next);
                current = next;
            }
            Err(_) => {}
        }
    }

    // Append the origin as the last point
    result.push(origin);

    result
}

/// Decode a chain of coordinates back to text.
/// Last point is the origin (ignored for decoding).
/// Data is in consecutive distances: dist(P0,P1), dist(P1,P2), ...
pub fn decode(_origin: WGS84<f64>, encoded: Vec<WGS84<f64>>) -> String {
    if encoded.len() < 3 {
        return String::from("Not enough points to decode");
    }

    // Last point is the stored origin, data points are everything before it
    let data_points = &encoded[..encoded.len() - 1];
    let mut strings: Vec<String> = vec![];

    for i in 0..data_points.len() - 1 {
        let val = data_points[i].distance(&data_points[i + 1]).round() as i32;
        let is_last = i == data_points.len() - 2;

        if is_last {
            let hex = format!("{:0width$x}", val, width = CHUNK_HEX_WIDTH);
            let trimmed = hex.trim_start_matches('0');
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
