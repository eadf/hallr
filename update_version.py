import re
import argparse

# Create a command-line argument parser
parser = argparse.ArgumentParser(description="Update version strings in Cargo.toml and blender_addon/__init__.py")
parser.add_argument("new_version", help="The new version string (e.g., X.Y.Z)")

# Parse the command-line arguments
args = parser.parse_args()
new_version = args.new_version

# Update Cargo.toml
with open('Cargo.toml', 'r') as file:
    content = file.read()
    updated_content = re.sub(r'\[package\][\s\S]*?version = "[\d\.]+"', f'[package]\nversion = "{new_version}"', content)
with open('Cargo.toml', 'w') as file:
    file.write(updated_content)

# Update __init__.py
with open('blender_addon/__init__.py', 'r') as file:
    content = file.read()
    updated_content = re.sub(r'"version": \([\d, ]+\)',
                             f'"version": ({", ".join(map(str, map(int, new_version.split("."))))})', content)
with open('blender_addon/__init__.py', 'w') as file:
    file.write(updated_content)

