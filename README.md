# Carol - A programmable third party for any protocol

## Running WASM locally

- Examples in `example-crates`. Each of these has a "guest" which can be compiled to WASM (and also be imported as a normal rust library).
- To run them you run `./build_example.sh <guest_name>` and this will build the guest into `/target`
- and then run the "run" crate which just invokes the guest through WASM: `cargo run --bin bitmex_run`


## Running HTTP 
