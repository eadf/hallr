import re
from pathlib import Path
import subprocess

def run_cargo_outdated():
    # Run the command and let it print directly to console
    process = subprocess.run(
        ["cargo", "outdated", "-d", "1"],
        text=True
    )

def sync_versions():
    # Paths to the files
    cargo_toml_path = Path("Cargo.toml")
    init_py_path = Path("blender_addon/__init__.py")

    # Read and parse version from Cargo.toml
    cargo_content = cargo_toml_path.read_text()
    version_match = re.search(r'version\s*=\s*"([\d.]+)"', cargo_content)
    if not version_match:
        raise ValueError("Could not find version in Cargo.toml")

    version_str = version_match.group(1)
    version_tuple = tuple(map(int, version_str.split('.')))

    # Read and update __init__.py
    init_content = init_py_path.read_text()

    updated_content = re.sub(r'"version": \([\d, ]+\)',
        f'"version": ({", ".join(map(str, map(int, version_str.split("."))))})', init_content)

    if init_content != updated_content:
        print(f"Updated __init__.py version to {version_tuple}")
    else:
        print(f"Did not update __init__.py version to {version_tuple}")

    # Write back to file
    init_py_path.write_text(updated_content)

if __name__ == "__main__":
    sync_versions()
    run_cargo_outdated()
