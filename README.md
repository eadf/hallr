[![crates.io](https://img.shields.io/crates/v/hallr.svg)](https://crates.io/crates/hallr)
[![Documentation](https://docs.rs/hallr/badge.svg)](https://docs.rs/hallr)
[![Workflow](https://github.com/eadf/hallr.rs/workflows/Rust/badge.svg)](https://github.com/eadf/hallr.rs/workflows/Rust/badge.svg)
[![dependency status](https://deps.rs/crate/hallr/0.1.0/status.svg)](https://deps.rs/crate/hallr/0.10)
![license](https://img.shields.io/crates/l/hallr)

# Hallr
Experimental Blender addon written in Rust. Work in progress, expect wildly fluctuating API:s.

## Usage
You should be able to find CI generated zip files under the [Actions->workflows](https://github.com/eadf/hallr/actions) tab.
These ZIP files contain the Hallr addon that Blender can install. 
There are (financial) issues with building for MacOs Arm with the github workflows. 
If you are on an Arm Mac you can locally build your own .dylib like this:

```bash
cargo build --release
```

and replace the `.dylib` inside the zip file.

Generating the entire zip file from scratch works too (must be in the root dir):
```bash
python3 build_script.py
```

## Contributing

We welcome contributions from the community. 
Feel free to submit pull requests or report issues on our GitHub repository.

## License
AGPL-3.0-or-later
