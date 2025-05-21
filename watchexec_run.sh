#!/bin/bash

# Validate we're in the correct directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found."
    exit 1
fi

if ! grep -q 'name = "hallr"' "Cargo.toml"; then
    echo "Error: The Cargo.toml file does not specify the project name as 'hallr'. Are you in the correct cwd?"
    exit 1
fi

# Only proceed with destructive operations if validation passes
rm -rf blender_addon_exported
rm -f hallr.zip

args=""
if [ -n "${HALLR_BUILD_TEST_FROM_INPUT+x}" ]; then
    echo "As a precaution, blender also needs to be run with the HALLR_BUILD_TEST_FROM_INPUT env set."
    echo "eg. 'HALLR_BUILD_TEST_FROM_INPUT=1 blender --debug'"
    args+=" --generate_tests"
fi
if [ -n "${HALLR_DISPLAY_SDF_CHUNKS+x}" ]; then
    args+=" --display_sdf_chunks"
fi

python3 build_script.py --dev_mode $args
