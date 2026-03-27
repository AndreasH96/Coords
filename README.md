# Coords

A steganographic encoding tool that hides file contents in GPS coordinates. Any file — text, images, binaries — is encoded as a chain of coordinate points where distances between consecutive points represent the data. The coordinates can be plotted on a world map but only decoded with the original origin key.

## How It Works

1. The input file is read as raw bytes and a metadata header (filename, size) is prepended
2. The combined bytes are converted to hex and split into chunks
3. Each chunk's hex value becomes a distance (in meters)
4. A random starting coordinate is generated, and each subsequent point is placed at the corresponding distance from the previous one
5. The origin coordinate (your secret key) is appended as the last point
6. The result is a chain of GPS coordinates that looks like a random walk across the globe

To decode, the distances between consecutive points are measured and converted back to the original data. The embedded metadata preserves the original filename.

## Installation

```
cargo build --release
```

## Usage

### Encode a file

Pass the origin as coordinates directly:

```
cargo run -- -i ./input.txt -o ./output.coords -c "40.6976312, -74.1444842, 0.0"
```

Or load the origin from a file:

```
cargo run -- -i ./input.txt -o ./output.coords -C ./example_data/origin_1.txt
```

The origin file should contain comma-separated latitude, longitude, and altitude values:

```
40.6976312, -74.1444842, 0.0
```

Any file type works — text, images, executables, archives, etc.

### Decode a file

```
cargo run -- --decode -i ./output.coords -o ./decoded_output.txt -c "40.6976312, -74.1444842, 0.0"
```

The original filename and size are printed during decode.

### Plot the encoded coordinates

Add `--plot` to generate a PNG world map showing where the coordinates land:

```
cargo run -- -i ./input.txt -o ./output.coords -c "40.6976312, -74.1444842, 0.0" --plot
```

This saves a `output.png` next to the output file and opens it.

### Print coordinates to terminal

```
cargo run -- -i ./input.txt -o ./output.coords -c "40.6976312, -74.1444842, 0.0" --output-cli
```

## CLI Options

| Flag | Description |
|------|-------------|
| `-i, --input-path` | Path to the input file (or `.coords` file in decode mode) |
| `-o, --output-path` | Path for the output file |
| `-c, --coords` | Origin coordinates as `"lat, lon, alt"` |
| `-C, --coords-path` | Path to a file containing origin coordinates |
| `-d, --decode` | Decode a `.coords` file back to the original |
| `--plot` | Generate a PNG world map of the encoded coordinates |
| `--output-cli` | Print encoded coordinates to the terminal |

## File Format

Encoded files use the `.coords` extension and store coordinates in a compact binary format: each point is two `i64` values (latitude and longitude scaled by 10^8) for lossless round-trip encoding. A metadata header embedded in the data preserves the original filename and file size.
