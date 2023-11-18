[![crates.io](https://img.shields.io/crates/v/hallr.svg)](https://crates.io/crates/hallr)
[![Documentation](https://docs.rs/hallr/badge.svg)](https://docs.rs/hallr)
[![Workflow](https://github.com/eadf/hallr/workflows/Rust/badge.svg)](https://github.com/eadf/hallr.rs/workflows/Rust/badge.svg)
![license](https://img.shields.io/crates/l/hallr)
[![](https://img.shields.io/static/v1?label=Sponsor&message=%E2%9D%A4&logo=GitHub&color=%23fe8e86)](https://github.com/sponsors/eadf)

Experimental Blender addon written in Rust. Work in progress, expect wildly fluctuating API:s.

## Usage
Read the [wiki](https://github.com/eadf/hallr/wiki) for the operation instructions. 

You should be able to find CI generated zip files under the [Actions->workflows](https://github.com/eadf/hallr/actions) tab.
These ZIP files contain the Hallr addon that Blender can install. 
There are ([financial](https://github.blog/2023-10-02-introducing-the-new-apple-silicon-powered-m1-macos-larger-runner-for-github-actions/#new-macos-runner-pricing)) issues with building for macOS Arm with the GitHub workflows. 
If you are on an Arm Mac you can locally rebuild your zip file like this (must be in the project root dir):
```bash
python3 build_script.py
```

## Contributing

We welcome contributions from the community. 
Feel free to submit pull requests or report issues on our GitHub repository.

## License
AGPL-3.0-or-later
