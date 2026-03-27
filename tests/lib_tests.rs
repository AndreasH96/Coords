#[cfg(test)]
pub(crate) mod lib_tests {
    use std::fs;

    use coords::{compute_coordinate, decode, encode};
    use nav_types::WGS84;

    #[test]
    fn test_encoding_decoding() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let original_text = "Hello, World!";
        let text_hex = hex::encode(original_text);
        let encoded = encode(origin, &text_hex);
        let decoded = decode(origin, encoded);
        assert_eq!(original_text, decoded, "Decoded text does not match original");
    }

    #[test]
    fn test_encoding_decoding_long_text() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);

        let original_text =
            fs::read_to_string("./tests/files/test_text.txt").expect("Should have been able to read the file");

        let text_hex = hex::encode(&original_text);
        let encoded = coords::encode(origin, &text_hex);
        let decoded = coords::decode(origin, encoded);
        assert_eq!(original_text, decoded, "Decoded text does not match original");
    }

    #[test]
    fn test_round_trip_encoding() {
        let origin = WGS84::from_degrees_and_meters(52.5200, 13.4050, 35.0);
        let text = "RustLang";
        let text_hex = hex::encode(text);

        let encoded = encode(origin, &text_hex);
        let decoded = decode(origin, encoded);

        assert_eq!(text, decoded, "Round-trip encoding/decoding failed");
    }

    #[test]
    fn test_distance_based_encoding() {
        let origin = WGS84::from_degrees_and_meters(51.5074, -0.1278, 0.0);
        let test_value = 0x1234;
        let coord = compute_coordinate(&origin, test_value);
        let computed_distance = origin.distance(&coord).round() as i32;

        assert_eq!(
            test_value, computed_distance,
            "Computed distance does not match input"
        );
    }

    #[test]
    fn test_chain_has_origin_at_end() {
        let origin = WGS84::from_degrees_and_meters(48.8566, 2.3522, 10.0);
        let text = "test";
        let text_hex = hex::encode(text);
        let encoded = encode(origin, &text_hex);

        // Last point should be the origin
        let last = encoded.last().unwrap();
        assert!(
            (last.latitude_degrees() - origin.latitude_degrees()).abs() < 1e-6
                && (last.longitude_degrees() - origin.longitude_degrees()).abs() < 1e-6,
            "Last point should be the origin"
        );
    }
}
