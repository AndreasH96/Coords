extern crate nav_types;

use hex;
use nav_types::WGS84;
use rand::Rng;

const EARTH_RADIUS: f64 = 6_371_000.0;
const CHUNK_HEX_WIDTH: usize = 6; // 6 hex chars = 3 bytes per chunk
const COORD_SCALE: f64 = 1e8;
const CHUNK_MOD: u32 = 0x1000000; // 2^24, the max value for a 6-hex-digit chunk

/// Derive a deterministic 64-bit seed from an origin coordinate.
fn derive_seed(origin: &WGS84<f64>) -> u64 {
    let lat_bits = (origin.latitude_degrees() * COORD_SCALE).round() as i64;
    let lon_bits = (origin.longitude_degrees() * COORD_SCALE).round() as i64;
    let mut h = (lat_bits as u64).wrapping_mul(0x9E3779B97F4A7C15);
    h = h.wrapping_add(lon_bits as u64);
    // splitmix64 mixing
    h = h.wrapping_add(0x9E3779B97F4A7C15);
    h = (h ^ (h >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94D049BB133111EB);
    h ^ (h >> 31)
}

/// Splitmix64 PRNG step — returns next pseudo-random value.
fn next_prng(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// Obfuscate a chunk value by adding a PRNG-derived offset (mod CHUNK_MOD).
fn obfuscate_chunk(value: u32, prng_output: u64) -> u32 {
    let offset = (prng_output % CHUNK_MOD as u64) as u32;
    (value + offset) % CHUNK_MOD
}

/// Reverse obfuscation by subtracting the same offset (mod CHUNK_MOD).
fn deobfuscate_chunk(obfuscated: u32, prng_output: u64) -> u32 {
    let offset = (prng_output % CHUNK_MOD as u64) as u32;
    (obfuscated + CHUNK_MOD - offset) % CHUNK_MOD
}

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

    let mut prng_state = derive_seed(&origin);

    for chunk in &chunks {
        match u32::from_str_radix(chunk, 16) {
            Ok(value) => {
                let obfuscated = obfuscate_chunk(value, next_prng(&mut prng_state));
                let next = compute_coordinate(&current, obfuscated as i32);
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

/// Core decode logic. When `trim_last` is true, the last chunk is trimmed to
/// its minimal even hex width (for variable-length text). When false, all
/// chunks use full CHUNK_HEX_WIDTH (for padded/compressed data).
fn decode_coords(encoded: &[WGS84<f64>], origin: &WGS84<f64>, trim_last: bool) -> Result<Vec<u8>, String> {
    if encoded.len() < 3 {
        return Err("Not enough points to decode".to_string());
    }

    let data_points = &encoded[..encoded.len() - 1];
    let mut strings: Vec<String> = vec![];
    let mut prng_state = derive_seed(origin);

    for i in 0..data_points.len() - 1 {
        let raw_dist = data_points[i].distance(&data_points[i + 1]).round() as u32;
        let val = deobfuscate_chunk(raw_dist, next_prng(&mut prng_state));
        let is_last = i == data_points.len() - 2;

        if is_last && trim_last {
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

    hex::decode(&hex_string).map_err(|e| format!("Failed to decode hex string: {}", e))
}

/// Decode a chain of coordinates back to raw bytes (with last-chunk trimming).
/// Last point is the origin (ignored for decoding).
/// Data is in consecutive distances: dist(P0,P1), dist(P1,P2), ...
pub fn decode(origin: WGS84<f64>, encoded: Vec<WGS84<f64>>) -> Result<Vec<u8>, String> {
    decode_coords(&encoded, &origin, true)
}

/// Encode raw bytes with metadata (filename, size) embedded in the coordinate chain.
/// Data is compressed with gzip before encoding.
pub fn encode_with_metadata(origin: WGS84<f64>, data: &[u8], filename: &str) -> Vec<WGS84<f64>> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let header = format!(r#"{{"name":"{}","size":{}}}"#, filename, data.len());
    let header_bytes = header.as_bytes();
    let header_len = (header_bytes.len() as u32).to_be_bytes();

    let mut payload = Vec::with_capacity(4 + header_bytes.len() + data.len());
    payload.extend_from_slice(&header_len);
    payload.extend_from_slice(header_bytes);
    payload.extend_from_slice(data);

    // Compress the payload
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&payload).expect("Failed to compress data");
    let mut compressed = encoder.finish().expect("Failed to finish compression");

    // Pad to a multiple of 3 bytes (CHUNK_HEX_WIDTH/2) so all hex chunks are
    // full-width and the last-chunk trimming in decode doesn't corrupt data.
    // The gzip decompressor ignores trailing bytes after the stream.
    let bytes_per_chunk = CHUNK_HEX_WIDTH / 2;
    while compressed.len() % bytes_per_chunk != 0 {
        compressed.push(0);
    }

    let text_hex = hex::encode(&compressed);
    encode(origin, &text_hex)
}

/// Decode a coordinate chain that contains metadata, returning (file_bytes, original_filename).
/// Data is decompressed with gzip after decoding.
pub fn decode_with_metadata(
    origin: WGS84<f64>,
    encoded: Vec<WGS84<f64>>,
) -> Result<(Vec<u8>, String), String> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let compressed = decode_coords(&encoded, &origin, false)?;

    // Decompress
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut bytes = Vec::new();
    decoder
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Failed to decompress data: {}", e))?;

    if bytes.len() < 4 {
        return Err("Data too short to contain metadata header".to_string());
    }

    let header_len = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;

    if bytes.len() < 4 + header_len {
        return Err("Data too short for declared header length".to_string());
    }

    let header_str = String::from_utf8(bytes[4..4 + header_len].to_vec())
        .map_err(|_| "Invalid UTF-8 in metadata header".to_string())?;

    // Parse filename from header JSON
    let name = header_str
        .find(r#""name":""#)
        .and_then(|start| {
            let val_start = start + 8;
            header_str[val_start..]
                .find('"')
                .map(|end| header_str[val_start..val_start + end].to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    let file_bytes = bytes[4 + header_len..].to_vec();
    Ok((file_bytes, name))
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
