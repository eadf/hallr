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
python3 build_script.py --dev_mode