extern crate nav_types;

use hex;
use itertools::Itertools;
use nav_types::{ENU, WGS84};
use rand::Rng;

fn generate_coordinate_for_distance(origin: WGS84<f32>, distance: i32) -> WGS84<f32> {
    fn generate_coordinate(origin: WGS84<f32>, distance: i32) -> WGS84<f32> {
        let mut rng = rand::thread_rng();
        let angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI); // Generate random angle

        // Calculate x and y directly from angle and distance
        let x = distance as f32 * angle.cos();
        let y = distance as f32 * angle.sin();

        let  vec = ENU::new(x, y, 0.0);
        origin + vec
    }
    let mut new_pos = generate_coordinate(origin, distance);
    let mut prev = origin.distance(&new_pos).floor();
    while origin.distance(&new_pos).floor() as i32 != distance{
        let diff_vec = origin - new_pos;
        let unit_vec = diff_vec/diff_vec.norm();
        new_pos +=unit_vec;

         if prev == origin.distance(&new_pos).floor(){
         new_pos = generate_coordinate(origin, distance);
         }

        prev = origin.distance(&new_pos).floor();

    }

    new_pos
}


fn encode(origin:WGS84<f32>,text:&str) -> Vec<WGS84<f32>>{
    let window_size = 4;
    let mut coordinates: Vec<WGS84<f32>> = vec![];
    println!("Encoding text {:?}", text);

    for i in (0..(text.len())).step_by(window_size) {
        let window_size = std::cmp::min(window_size, text.len() - i); // Adjust size for last window
        let window = text
            .chars()
            .skip(i)
            .take(window_size)
            .collect::<String>();

        coordinates.push(generate_coordinate_for_distance(
            origin,i32::from_str_radix(&window, 16).unwrap()
            ,
        ))
    }
    coordinates
}

fn decode(origin: WGS84<f32>,encoded:Vec<WGS84<f32>>) -> String{
    String::from_utf8(
        hex::decode(
            encoded
                    .iter()
                    .map(|x| format!("{:x}", origin.distance(x).floor() as i32))
                    .join(""),
            )
            .unwrap(),
        )
        .unwrap_or(String::from("Failed to parse, incorrect coordinates!"))
}



fn main() {
    let start: WGS84<f32> = WGS84::from_degrees_and_meters(40.6976312,-74.1444842, 0.0);

    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum";

    let text_hex = hex::encode(text);

    let encoded = encode(start,&text_hex);

    println!("\nRestoring to text");
    let decoded = decode(start,encoded);

    println!("Restored text:{:?}", decoded);

    println!("Succeded: {:?}",text==decoded);

}
