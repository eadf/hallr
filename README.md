[![crates.io](https://img.shields.io/crates/v/hallr.svg)](https://crates.io/crates/hallr)
[![Documentation](https://docs.rs/hallr/badge.svg)](https://docs.rs/hallr)
[![Workflow](https://github.com/eadf/hallr.rs/workflows/Rust/badge.svg)](https://github.com/eadf/hallr.rs/workflows/Rust/badge.svg)
[![dependency status](https://deps.rs/crate/hallr/0.1.0/status.svg)](https://deps.rs/crate/hallr/0.10)
![license](https://img.shields.io/crates/l/hallr)

# Hallr
Experimental Blender addon written in Rust. Work in progress

## Usage
You should be able to find CI generated zip files under the [Actions->workflows](https://github.com/eadf/hallr/actions) tab that blender can install. 
There are issues with building for MacOs Arm with the github workflows, 
in that case you can locally run:
```bash
cargo build --release
```

and replace the `.dylib` inside the zip file

## License
AGPL-3.0-or-later
