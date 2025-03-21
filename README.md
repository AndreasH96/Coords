# Coords
A work in progress encoding format which, for each pair of four bytes, turns it into decimal form and then replaces it by a coordinate with a corresponding distance from a given origin. 
To encode a file, you need to give a origin, this will then be the key to decode the file back to its original form. 

The plan is to make this into a library and also a cli tool

# Usage of the CLI tool
To encode a file

    cargo run -- --input-path ./tests/files/test_text.txt --output-path ./text.coords --coords "[40.6976312, -74.1444842, 0.0]"          

or 
    
    cargo run -- --input-path ./tests/files/test_text.txt --output-path ./text.coords --coords-file ./example_data/origin_1.txt  


The output will be stored in a binary format with the file extension `.coords`. 