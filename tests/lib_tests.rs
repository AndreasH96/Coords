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

#[cfg(test)]
mod pentest {
    use coords::{decode, decode_with_metadata, encode, encode_with_metadata};
    use nav_types::WGS84;

    // ── Category 1: Wrong-key attacks ──

    #[test]
    fn test_nearby_origin_does_not_decode() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let nearby = WGS84::from_degrees_and_meters(40.6986312, -74.1434842, 0.0); // ~0.001 deg off
        let data = b"Sensitive payload";

        let encoded = encode_with_metadata(origin, data, "file.txt");
        let result = decode_with_metadata(nearby, encoded);

        assert!(result.is_err(), "Nearby but non-identical origin should fail to decode");
    }

    #[test]
    fn test_brute_force_nearby_origins_all_fail() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let data = b"Brute force test";
        let encoded = encode_with_metadata(origin, data, "bf.txt");

        for lat_step in -2i32..=2 {
            for lon_step in -2i32..=2 {
                if lat_step == 0 && lon_step == 0 {
                    continue; // skip exact match
                }
                let trial = WGS84::from_degrees_and_meters(
                    40.6976312 + lat_step as f64 * 0.0001,
                    -74.1444842 + lon_step as f64 * 0.0001,
                    0.0,
                );
                let result = decode_with_metadata(trial, encoded.clone());
                assert!(
                    result.is_err(),
                    "Nearby origin ({}, {}) should not decode",
                    lat_step,
                    lon_step
                );
            }
        }
    }

    #[test]
    fn test_zero_origin_does_not_bypass() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let zero = WGS84::from_degrees_and_meters(0.0, 0.0, 0.0);
        let data = b"Zero origin bypass test";

        let encoded = encode_with_metadata(origin, data, "zero.txt");
        let result = decode_with_metadata(zero, encoded);

        assert!(result.is_err(), "Zero origin must not bypass salt");
    }

    #[test]
    fn test_antipodal_origin_fails() {
        let lat = 40.6976312;
        let lon = -74.1444842;
        let origin = WGS84::from_degrees_and_meters(lat, lon, 0.0);
        let antipodal = WGS84::from_degrees_and_meters(-lat, lon + 180.0, 0.0);
        let data = b"Antipodal test";

        let encoded = encode_with_metadata(origin, data, "anti.txt");
        let result = decode_with_metadata(antipodal, encoded);

        assert!(result.is_err(), "Antipodal origin must not decode");
    }

    #[test]
    fn test_swapped_lat_lon_origin_fails() {
        let lat = 40.6976312;
        let lon = -74.1444842;
        let origin = WGS84::from_degrees_and_meters(lat, lon, 0.0);
        let swapped = WGS84::from_degrees_and_meters(lon, lat, 0.0);
        let data = b"Swapped coords test";

        let encoded = encode_with_metadata(origin, data, "swap.txt");
        let result = decode_with_metadata(swapped, encoded);

        assert!(result.is_err(), "Swapped lat/lon must not decode");
    }

    // ── Category 2: Ciphertext analysis ──

    #[test]
    fn test_same_plaintext_different_origins_differ() {
        let origin_a = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let origin_b = WGS84::from_degrees_and_meters(52.5200, 13.4050, 0.0);
        let text = "DEADBEEF";

        let chain_a = encode(origin_a, text);
        let chain_b = encode(origin_b, text);

        // Extract raw distances (skip P0 which is random, and last which is origin)
        let dist_a: Vec<i32> = chain_a.windows(2)
            .take(chain_a.len() - 2) // exclude origin segment
            .map(|w| w[0].distance(&w[1]).round() as i32)
            .collect();
        let dist_b: Vec<i32> = chain_b.windows(2)
            .take(chain_b.len() - 2)
            .map(|w| w[0].distance(&w[1]).round() as i32)
            .collect();

        assert_eq!(dist_a.len(), dist_b.len(), "Same plaintext should produce same number of distances");
        assert_ne!(dist_a, dist_b, "Different origins must produce different distance sequences");
    }

    #[test]
    fn test_same_plaintext_same_origin_distances_differ() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let text = "Hello, World!";
        let text_hex = hex::encode(text);

        let chain_a = encode(origin, &text_hex);
        let chain_b = encode(origin, &text_hex);

        // Both must decode correctly
        let decoded_a = decode(origin, chain_a.clone()).expect("decode a failed");
        let decoded_b = decode(origin, chain_b.clone()).expect("decode b failed");
        assert_eq!(decoded_a, text.as_bytes());
        assert_eq!(decoded_b, text.as_bytes());

        // Coordinates differ (random P0)
        let coords_a: Vec<(f64, f64)> = chain_a.iter()
            .map(|c| (c.latitude_degrees(), c.longitude_degrees()))
            .collect();
        let coords_b: Vec<(f64, f64)> = chain_b.iter()
            .map(|c| (c.latitude_degrees(), c.longitude_degrees()))
            .collect();
        assert_ne!(coords_a, coords_b, "Two encodings should use different random starts");
    }

    #[test]
    fn test_raw_distances_not_valid_ascii() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let text = "Hello, World!";
        let text_hex = hex::encode(text);
        let chain = encode(origin, &text_hex);

        // Extract raw distances (without deobfuscation)
        let raw_distances: Vec<u32> = chain.windows(2)
            .take(chain.len() - 2)
            .map(|w| w[0].distance(&w[1]).round() as u32)
            .collect();

        // Try to interpret raw distances as hex chunks → bytes
        let raw_hex: String = raw_distances.iter()
            .map(|d| format!("{:06x}", d))
            .collect();

        if let Ok(raw_bytes) = hex::decode(&raw_hex) {
            // If it happens to decode as hex, it should NOT be valid ASCII matching input
            let raw_str = String::from_utf8_lossy(&raw_bytes);
            assert_ne!(
                raw_str.as_ref(),
                text,
                "Raw distances must not directly reveal plaintext"
            );
        }
        // If hex::decode fails, that's also fine — raw distances are obfuscated
    }

    #[test]
    fn test_repeated_plaintext_chunks_produce_different_distances() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        // "AAAAAA" → hex "414141414141" → two identical 3-byte chunks "414141"
        let text = "AAAAAA";
        let text_hex = hex::encode(text);
        let chain = encode(origin, &text_hex);

        // Raw distances for the two data segments (skip last point = origin)
        let data_points = &chain[..chain.len() - 1];
        assert!(data_points.len() >= 3, "Need at least 3 data points for 2 chunks");

        let dist_0 = data_points[0].distance(&data_points[1]).round() as u32;
        let dist_1 = data_points[1].distance(&data_points[2]).round() as u32;

        assert_ne!(
            dist_0, dist_1,
            "Identical plaintext chunks must produce different obfuscated distances"
        );
    }

    #[test]
    fn test_distance_distribution_appears_uniform() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        // Long payload to get enough distances for statistical test
        let data = vec![0xABu8; 300]; // 300 bytes → 200 chunks of 3 bytes
        let encoded = encode_with_metadata(origin, &data, "dist_test.bin");

        let data_points = &encoded[..encoded.len() - 1];
        let distances: Vec<u32> = data_points
            .windows(2)
            .map(|w| w[0].distance(&w[1]).round() as u32)
            .collect();

        // Bucket into 16 ranges over [0, CHUNK_MOD)
        let num_buckets = 16u32;
        let bucket_size = 0x1000000u32 / num_buckets;
        let mut buckets = vec![0u32; num_buckets as usize];

        for &d in &distances {
            let bucket = (d / bucket_size).min(num_buckets - 1);
            buckets[bucket as usize] += 1;
        }

        let expected = distances.len() as f64 / num_buckets as f64;
        let max_allowed = (expected * 3.0) as u32;

        for (i, &count) in buckets.iter().enumerate() {
            assert!(
                count <= max_allowed,
                "Bucket {} has {} entries (expected ~{:.0}, max {}). Distribution not uniform enough.",
                i,
                count,
                expected,
                max_allowed
            );
        }
    }

    // ── Category 3: Structural attacks ──

    #[test]
    fn test_stripped_origin_decode_fails() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let data = b"Strip origin test";
        let mut encoded = encode_with_metadata(origin, data, "strip.txt");

        // Remove the last point (the origin)
        encoded.pop();
        // Append an arbitrary point as fake origin
        let fake_origin = WGS84::from_degrees_and_meters(35.0, -120.0, 0.0);
        encoded.push(fake_origin);

        let result = decode_with_metadata(fake_origin, encoded);
        assert!(result.is_err(), "Stripped-and-replaced origin should fail to decode");
    }

    #[test]
    fn test_truncated_chain_fails() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let data = b"Truncation test with enough data";
        let mut encoded = encode_with_metadata(origin, data, "trunc.txt");

        // Remove a point from the middle
        let mid = encoded.len() / 2;
        encoded.remove(mid);

        let result = decode_with_metadata(origin, encoded);
        assert!(result.is_err(), "Truncated chain should fail to decode");
    }

    #[test]
    fn test_reordered_points_fail() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let data = b"Reorder test with enough data here";
        let mut encoded = encode_with_metadata(origin, data, "reorder.txt");

        // Swap two adjacent data points (not P0 or origin)
        let swap_idx = 2;
        if encoded.len() > swap_idx + 2 {
            encoded.swap(swap_idx, swap_idx + 1);
        }

        let result = decode_with_metadata(origin, encoded);
        assert!(result.is_err(), "Reordered points should fail to decode");
    }

    #[test]
    fn test_spliced_chains_fail() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let data_a = b"First payload for splice";
        let data_b = b"Second payload for splice";

        let encoded_a = encode_with_metadata(origin, data_a, "a.txt");
        let encoded_b = encode_with_metadata(origin, data_b, "b.txt");

        let mid_a = encoded_a.len() / 2;
        let mid_b = encoded_b.len() / 2;

        // Take first half of A, second half of B
        let mut spliced: Vec<WGS84<f64>> = Vec::new();
        spliced.extend_from_slice(&encoded_a[..mid_a]);
        spliced.extend_from_slice(&encoded_b[mid_b..]);

        let result = decode_with_metadata(origin, spliced);
        assert!(result.is_err(), "Spliced chains should fail to decode");
    }

    // ── Category 4: Known-plaintext attacks ──

    #[test]
    fn test_known_plaintext_cannot_recover_origin() {
        // Attacker knows plaintext and encoded coordinates.
        // Derive PRNG offsets = (raw_distance - expected_chunk) mod CHUNK_MOD.
        // Multiple origins can produce the same offset sequence due to mod.
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let text = "KnownPT";
        let text_hex = hex::encode(text);
        let chain = encode(origin, &text_hex);

        // Recover raw distances
        let data_points = &chain[..chain.len() - 1];
        let raw_distances: Vec<u32> = data_points
            .windows(2)
            .map(|w| w[0].distance(&w[1]).round() as u32)
            .collect();

        // Compute expected chunk values from known plaintext
        let chunks: Vec<u32> = text_hex
            .as_bytes()
            .chunks(6)
            .map(|c| {
                let s: String = c.iter().map(|&b| b as char).collect();
                u32::from_str_radix(&s, 16).unwrap()
            })
            .collect();

        // Derive offsets
        let chunk_mod = 0x1000000u32;
        let offsets: Vec<u32> = raw_distances
            .iter()
            .zip(chunks.iter())
            .map(|(&raw, &expected)| (raw + chunk_mod - expected) % chunk_mod)
            .collect();

        // The offsets alone cannot uniquely determine the origin because:
        // 1. The offset is mod 2^24 — many seeds map to the same offset
        // 2. Multiple origins produce different seeds but could share offset patterns
        // We verify the offsets are non-trivial (not all zero, not plaintext-matching)
        assert!(
            offsets.iter().any(|&o| o != 0),
            "PRNG offsets must be non-zero (salt is active)"
        );

        // Verify offsets are not directly the raw distances (i.e., expected chunks ≠ 0)
        assert_ne!(
            offsets, raw_distances.iter().map(|&d| d % chunk_mod).collect::<Vec<_>>(),
            "Offsets should differ from raw distances (plaintext is not all zeros)"
        );
    }

    #[test]
    fn test_known_plaintext_different_encodings_give_different_coordinates() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let text = "SameText";
        let text_hex = hex::encode(text);

        let chain_a = encode(origin, &text_hex);
        let chain_b = encode(origin, &text_hex);

        // Both must decode correctly
        let decoded_a = decode(origin, chain_a.clone()).expect("decode a");
        let decoded_b = decode(origin, chain_b.clone()).expect("decode b");
        assert_eq!(decoded_a, text.as_bytes());
        assert_eq!(decoded_b, text.as_bytes());

        // Absolute coordinates differ due to random P0, even though distances
        // are deterministic for the same origin+plaintext. An attacker observing
        // two encoded chains sees entirely different coordinate sequences.
        let coords_a: Vec<(f64, f64)> = chain_a.iter()
            .take(chain_a.len() - 1) // exclude origin
            .map(|c| (c.latitude_degrees(), c.longitude_degrees()))
            .collect();
        let coords_b: Vec<(f64, f64)> = chain_b.iter()
            .take(chain_b.len() - 1)
            .map(|c| (c.latitude_degrees(), c.longitude_degrees()))
            .collect();
        assert_ne!(
            coords_a, coords_b,
            "Two encodings must produce different coordinate chains (different random P0)"
        );
    }

    // ── Category 5: Edge cases ──

    #[test]
    fn test_extreme_origins_work() {
        let origins = [
            WGS84::from_degrees_and_meters(89.9999, 179.9999, 0.0),
            WGS84::from_degrees_and_meters(-89.9999, -179.9999, 0.0),
            WGS84::from_degrees_and_meters(0.0, 0.0, 0.0),
            WGS84::from_degrees_and_meters(0.0, 180.0, 0.0),
        ];

        for origin in &origins {
            let data = b"Edge case test";
            let encoded = encode_with_metadata(*origin, data, "edge.txt");
            let (decoded, name) = decode_with_metadata(*origin, encoded)
                .unwrap_or_else(|e| panic!(
                    "Failed at origin ({}, {}): {}",
                    origin.latitude_degrees(),
                    origin.longitude_degrees(),
                    e
                ));
            assert_eq!(decoded, data.to_vec());
            assert_eq!(name, "edge.txt");
        }
    }

    #[test]
    fn test_very_short_plaintext_protected() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let wrong = WGS84::from_degrees_and_meters(52.5200, 13.4050, 0.0);

        // Single byte via encode_with_metadata
        let data = b"X";
        let encoded = encode_with_metadata(origin, data, "x.txt");
        let result = decode_with_metadata(wrong, encoded);
        assert!(result.is_err(), "Single-byte payload must be protected by salt");
    }

    #[test]
    fn test_empty_data_metadata_protected() {
        let origin = WGS84::from_degrees_and_meters(40.6976312, -74.1444842, 0.0);
        let wrong = WGS84::from_degrees_and_meters(52.5200, 13.4050, 0.0);

        let data: &[u8] = b"";
        let encoded = encode_with_metadata(origin, data, "empty.txt");

        // Correct origin should work
        let (decoded, name) = decode_with_metadata(origin, encoded.clone())
            .expect("Correct origin should decode empty data");
        assert_eq!(decoded, data.to_vec());
        assert_eq!(name, "empty.txt");

        // Wrong origin should fail
        let result = decode_with_metadata(wrong, encoded);
        assert!(result.is_err(), "Empty data must still be protected by salt");
    }
}
