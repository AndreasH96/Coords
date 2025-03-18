extern crate nav_types;

use core::f32;
use dashmap::DashMap;
use hex;
use lazy_static::lazy_static;
use nav_types::{ENU, WGS84};
use rand::Rng;
use rayon::prelude::*;
use std::cmp::max;

lazy_static! {
    static ref CACHE: DashMap<i32, WGS84<f32>> = DashMap::new();
}

pub fn round_coord(coord: WGS84<f32>, decimals: i32) -> WGS84<f32> {
    let factor = 10_f32.powi(decimals);
    WGS84::from_degrees_and_meters(
        (coord.latitude_degrees() * factor).round() / factor,
        (coord.longitude_degrees() * factor).round() / factor,
        (coord.altitude() * factor).round() / factor,
    )
}

pub fn compute_coordinate(origin: WGS84<f32>, distance: i32) -> WGS84<f32> {
    fn generate_coordinate(origin: WGS84<f32>, distance: i32) -> WGS84<f32> {
        let mut rng = rand::thread_rng();
        let angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI); // Generate random angle

        let x = distance as f32 * angle.cos();
        let y = distance as f32 * angle.sin();

        let vec = ENU::new(x, y, 0.0);
        round_coord(origin + vec, 6)
    }

    let n_spawns = 10;
    let mut positions = vec![];
    let mut best_index = 0;
    let mut best_distance = i32::MAX;

    for i in 0..n_spawns {
        let p = generate_coordinate(origin, distance);
        let dist = origin.distance(&p).floor() as i32;
        if (distance - dist).abs() < best_distance {
            best_distance = (distance - dist).abs();
            best_index = i;
        }
        positions.push(p);
    }
    while best_distance != 0 {
        for i in 0..n_spawns {
            let diff_vec = origin - positions[i];
            let unit_vec = diff_vec / diff_vec.norm() * max(best_distance, 1) as f32;
            let curr_dist = origin.distance(&(positions[i])).floor() as i32;

            if curr_dist == origin.distance(&(positions[i] + unit_vec)).floor() as i32 {
                let mut rng = rand::thread_rng();
                positions[i] += unit_vec * rng.gen_range(2.0..3.0);
            } else {
                positions[i] += unit_vec;
            }
            positions[i] = round_coord(positions[i], 6);
            let curr_dist = origin.distance(&(positions[i])).floor() as i32;
            if (distance - curr_dist).abs() < best_distance {
                best_distance = (distance - curr_dist).abs();
                best_index = i;
            }
        }
    }
    positions[best_index]
}

pub fn generate_coordinate_for_distance(origin: WGS84<f32>, distance: i32) -> WGS84<f32> {
    if let Some(entry) = CACHE.get(&distance) {
        return *entry.value();
    }

    let coordinate = compute_coordinate(origin, distance);
    CACHE.insert(distance, coordinate);
    coordinate
}

pub fn encode(origin: WGS84<f32>, text: &str) -> Vec<WGS84<f32>> {
    let window_size = 4;
    let chunks: Vec<String> = text
        .chars()
        .collect::<Vec<_>>()
        .chunks(window_size)
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

pub fn decode(origin: WGS84<f32>, encoded: Vec<WGS84<f32>>) -> String {
    let mut strings: Vec<String> = vec![];

    for (i, cord) in encoded.iter().enumerate() {
        let val = origin.distance(cord).floor() as i32;
        // Last byte in file might be corrupted if this is not done
        if val < 100 && i == encoded.len() - 1 {
            strings.push(format!("{:02x}", val))
        } else {
            strings.push(format!("{:04x}", val))
        }
    }
    let hex_string: String = strings.join("");
    
    // Attempt hex decoding
    match hex::decode(&hex_string) {
        Ok(decoded_bytes) => match String::from_utf8(decoded_bytes.clone()) {
            Ok(text) => text,
            Err(_e) => String::from("Failed to parse, invalid UTF-8!"),
        },
        Err(_e) => String::from("Failed to decode hex string!"),
    }
}
