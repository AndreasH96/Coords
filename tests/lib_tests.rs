#[cfg(test)]
pub(crate) mod lib_tests {
    use std::fs;

    use coords::{compute_coordinate, decode, decode_with_metadata, encode, encode_with_metadata};
    use nav_types::WGS84;

    #[test]
    fn test_encoding_decoding() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let original_text = "Hello, World!";
        let text_hex = hex::encode(original_text);
        let encoded = encode(origin, &text_hex);
        let decoded = decode(origin, encoded).expect("decode failed");
        assert_eq!(original_text.as_bytes(), &decoded[..], "Decoded bytes do not match original");
    }

    #[test]
    fn test_encoding_decoding_long_text() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);

        let original_text =
            fs::read_to_string("./tests/files/test_text.txt").expect("Should have been able to read the file");

        let text_hex = hex::encode(&original_text);
        let encoded = encode(origin, &text_hex);
        let decoded = decode(origin, encoded).expect("decode failed");
        assert_eq!(original_text.as_bytes(), &decoded[..], "Decoded bytes do not match original");
    }

    #[test]
    fn test_round_trip_encoding() {
        let origin = WGS84::from_degrees_and_meters(52.5200, 13.4050, 35.0);
        let text = "RustLang";
        let text_hex = hex::encode(text);

        let encoded = encode(origin, &text_hex);
        let decoded = decode(origin, encoded).expect("decode failed");

        assert_eq!(text.as_bytes(), &decoded[..], "Round-trip encoding/decoding failed");
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

        let last = encoded.last().unwrap();
        assert!(
            (last.latitude_degrees() - origin.latitude_degrees()).abs() < 1e-6
                && (last.longitude_degrees() - origin.longitude_degrees()).abs() < 1e-6,
            "Last point should be the origin"
        );
    }

    #[test]
    fn test_metadata_round_trip() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let data = b"Hello, metadata!";
        let filename = "test.txt";

        let encoded = encode_with_metadata(origin, data, filename);
        let (decoded_bytes, decoded_name) = decode_with_metadata(origin, encoded).expect("decode failed");

        assert_eq!(data.to_vec(), decoded_bytes);
        assert_eq!(filename, decoded_name);
    }

    #[test]
    fn test_binary_round_trip() {
        let origin = WGS84::from_degrees_and_meters(52.5200, 13.4050, 0.0);
        // Binary data with null bytes and non-UTF8 sequences
        let data: Vec<u8> = vec![0x00, 0xFF, 0x01, 0xFE, 0x80, 0x7F, 0x00, 0x00];

        let encoded = encode_with_metadata(origin, &data, "binary.bin");
        let (decoded_bytes, decoded_name) = decode_with_metadata(origin, encoded).expect("decode failed");

        assert_eq!(data, decoded_bytes);
        assert_eq!("binary.bin", decoded_name);
    }

    #[test]
    fn test_png_round_trip() {
        let origin = WGS84::from_degrees_and_meters(35.6762, 139.6503, 0.0);
        let data = fs::read("./tests/files/test_image.png").expect("Failed to read PNG");

        let encoded = encode_with_metadata(origin, &data, "test_image.png");
        let (decoded_bytes, decoded_name) = decode_with_metadata(origin, encoded).expect("decode failed");

        assert_eq!(data, decoded_bytes, "PNG round-trip failed");
        assert_eq!("test_image.png", decoded_name);
    }

    #[test]
    fn test_json_round_trip() {
        let origin = WGS84::from_degrees_and_meters(-33.8688, 151.2093, 0.0);
        let data = fs::read("./tests/files/test_data.json").expect("Failed to read JSON");

        let encoded = encode_with_metadata(origin, &data, "test_data.json");
        let (decoded_bytes, decoded_name) = decode_with_metadata(origin, encoded).expect("decode failed");

        assert_eq!(data, decoded_bytes, "JSON round-trip failed");
        assert_eq!("test_data.json", decoded_name);
    }

    #[test]
    fn test_csv_round_trip() {
        let origin = WGS84::from_degrees_and_meters(55.7558, 37.6173, 0.0);
        let data = fs::read("./tests/files/test_data.csv").expect("Failed to read CSV");

        let encoded = encode_with_metadata(origin, &data, "test_data.csv");
        let (decoded_bytes, decoded_name) = decode_with_metadata(origin, encoded).expect("decode failed");

        assert_eq!(data, decoded_bytes, "CSV round-trip failed");
        assert_eq!("test_data.csv", decoded_name);
    }

    #[test]
    fn test_all_byte_values_round_trip() {
        let origin = WGS84::from_degrees_and_meters(1.3521, 103.8198, 0.0);
        let data = fs::read("./tests/files/test_binary.bin").expect("Failed to read binary");

        let encoded = encode_with_metadata(origin, &data, "test_binary.bin");
        let (decoded_bytes, decoded_name) = decode_with_metadata(origin, encoded).expect("decode failed");

        assert_eq!(data, decoded_bytes, "Binary (all byte values) round-trip failed");
        assert_eq!("test_binary.bin", decoded_name);
    }

    #[test]
    fn test_jpeg_round_trip() {
        let origin = WGS84::from_degrees_and_meters(41.9028, 12.4964, 0.0);
        let data = fs::read("./tests/files/test_image_large.jpg").expect("Failed to read JPEG");

        let encoded = encode_with_metadata(origin, &data, "test_image_large.jpg");
        let (decoded_bytes, decoded_name) = decode_with_metadata(origin, encoded).expect("decode failed");

        assert_eq!(data, decoded_bytes, "JPEG round-trip failed");
        assert_eq!("test_image_large.jpg", decoded_name);
    }

    #[test]
    fn test_mp4_round_trip() {
        let origin = WGS84::from_degrees_and_meters(37.7749, -122.4194, 0.0);
        let data = fs::read("./tests/files/test_video.mp4").expect("Failed to read MP4");

        let encoded = encode_with_metadata(origin, &data, "test_video.mp4");
        let (decoded_bytes, decoded_name) = decode_with_metadata(origin, encoded).expect("decode failed");

        assert_eq!(data, decoded_bytes, "MP4 round-trip failed");
        assert_eq!("test_video.mp4", decoded_name);
    }

    #[test]
    fn test_wrong_origin_decode_fails() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let wrong_origin = WGS84::from_degrees_and_meters(52.5200, 13.4050, 0.0);
        let data = b"Secret data that should not decode with wrong origin";

        let encoded = encode_with_metadata(origin, data, "secret.txt");
        let result = decode_with_metadata(wrong_origin, encoded);

        assert!(result.is_err(), "Decoding with wrong origin should fail");
    }
}
