#!/usr/bin/env python3
"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

import subprocess
import time
import os
import platform
import shutil
import glob
import argparse
import sys


def clear_directory(dir_path, delete_root=False):
    """Remove all files from a directory while keeping the directory itself."""
    if not os.path.exists(dir_path):
        return
    for filename in os.listdir(dir_path):
        file_path = os.path.join(dir_path, filename)
        try:
            if os.path.isdir(file_path):
                clear_directory(file_path)
            elif os.path.isfile(file_path):
                os.chmod(file_path, 0o666)
                os.remove(file_path)
        except Exception as e:
            print(f"Failed to delete {file_path}: {e}")
    if delete_root:
        os.chmod(dir_path, 0o666)
        os.rmdir(dir_path)


def parse_arguments():
    """Parse command-line arguments."""
    parser = argparse.ArgumentParser(description="Package the hallr Blender add-on.")
    parser.add_argument("--dev_mode", action="store_true", help="Enable development mode.")
    parser.add_argument("--release", action="store_true", help="Build in release mode.")
    parser.add_argument("--debug", action="store_true", help="Build in debug mode.")
    parser.add_argument("--generate_tests", action="store_true", help="enable the ’generate_test_case_from_input’ feature")

    args = parser.parse_args()
    if args.debug and args.release:
        parser.error("--debug and --release cannot be used together.")
    if args.release:
        print("Warning: --release is now the default behavior. Use --debug for debug mode. "
              "The --release flag will be deprecated in a future version.", file=sys.stderr)

    # Set defaults if neither is specified
    if not args.debug and not args.release:
        args.release = True
        args.debug = False
    else:
        args.release = not args.debug

    return args


def validate_rust_project():
    """Check for Cargo.toml and verify project name."""
    if not os.path.isfile("Cargo.toml"):
        print("Error: Cargo.toml not found.")
        exit(1)
    with open("Cargo.toml", "r") as f:
        content = f.read()
        if "name = \"hallr\"" not in content:
            print("Error: The Cargo.toml file does not specify the project name as 'hallr'. Are you in the correct cwd?")
            exit(1)


def run_cargo_build(dev_mode, debug, generate_tests):
    """Execute the cargo build command with appropriate flags.

    Args:
        dev_mode: Whether to build in development mode
        debug: Whether to build with debug symbols (implies dev_mode)
        generate_tests: Whether to generate test cases (only works in dev_mode)
    """
    feature_args = []
    if generate_tests:
        if not dev_mode:
            print("Warning: Test generation is only available in dev_mode. Ignoring generate_tests.")
        else:
            feature_args = ["--features", "generate_test_case_from_input"]

    # Build base command
    if not dev_mode:
        # Production build with maximum optimizations
        command = ["cargo", "rustc", "--release", "--crate-type=cdylib"]
        command.extend(feature_args)  # Will be empty in production
        command.extend(["--", "-C", "opt-level=3", "-C", "lto=fat"])
    elif debug:
        # Debug build (implies dev_mode)
        command = ["cargo", "build"]
        command.extend(feature_args)
    else:
        # Dev release build
        command = ["cargo", "build", "--release"]
        command.extend(feature_args)

    print(f"running : {command}")
    result = subprocess.run(command)
    if result.returncode != 0:
        print("Cargo build failed.")
        exit(1)


def copy_python_files(source_dir, dest_dir):
    """Copy Python files from the source to the destination directory."""
    clear_directory(os.path.join(dest_dir, "__pycache__"), delete_root=True)
    py_files = glob.glob(f"{source_dir}/*.py")
    os.makedirs(dest_dir, exist_ok=True)
    for source_file in py_files:
        dest_file = os.path.join(dest_dir, os.path.basename(source_file))
        if os.path.isfile(dest_file):
            os.chmod(dest_file, 0o666)
        shutil.copy(source_file, dest_file)
        print(f"Copied Python file: {source_file} -> {dest_file}")


def process_python_files(addon_exported_path, dev_mode):
    """Perform replacements in exported Python files."""
    addon_exported_path = os.path.join(os.getcwd(), addon_exported_path)
    target_release_path = os.path.join(addon_exported_path, "lib")

    for file in glob.glob(f"{addon_exported_path}/*.py"):
        if args.dev_mode:
            os.chmod(file, 0o666)  # Make writable before overwrite
            with open(file, 'r') as f:
                content = f.read()

            content = content.replace('HALLR__BLENDER_ADDON_PATH', addon_exported_path)
            content = content.replace('HALLR__TARGET_RELEASE', target_release_path)
            content = content.replace('DEV_MODE = False', 'DEV_MODE = True')
            content = content.replace('from . import', 'import')

            with open(file, 'w') as f:
                f.write(content)
        os.chmod(file, 0o444)  # Read-only for everyone
        print(f"Processed Python file: {file}")


def copy_library_files(dev_mode, debug, dest_lib_directory):
    """Copy built library files to the destination directory."""
    timestamp = str(int(time.time()))
    build_type = "debug" if debug else "release"
    target_dir = os.path.join("target", build_type)

    is_windows = platform.system() == "Windows"
    library_extension = ".dll" if is_windows else ".so" if platform.system() == "Linux" else ".dylib"

    # Use different prefix patterns based on platform
    lib_prefix = "" if is_windows else "lib"

    # Find library files with the correct pattern for the platform
    lib_files = [f for f in os.listdir(target_dir)
                if f.startswith(f"{lib_prefix}hallr") and f.endswith(library_extension)]

    if not lib_files:
        print(f"No library files found in {target_dir}.")
        exit(1)

    clear_directory(dest_lib_directory)
    # Check if the destination directory exists, and create it if not
    os.makedirs(dest_lib_directory, exist_ok=True)

    for lib_file in lib_files:
        new_name = os.path.join(dest_lib_directory, lib_file)
        if dev_mode:
            # For consistent naming in dev mode, potentially add lib prefix on Windows
            new_name = os.path.join(dest_lib_directory,
                                   f"{lib_prefix}hallr_{timestamp}{library_extension}")
        lib_file = os.path.join(target_dir, lib_file)
        shutil.copy(lib_file, new_name)
        os.chmod(new_name, 0o444)  # Read-only for everyone
        print(f"Copied {lib_file} to {new_name}")


def package_addon(dev_mode):
    """Package the add-on into a zip file."""
    base_dir = os.getcwd()
    export_dir = "blender_addon_exported"
    if dev_mode:
        print(f"Updated the files under blender_addon_exported.")
        print(f"Use blender to open and run the script: {export_dir}/__init__.py")
    else:
        shutil.move(export_dir, "hallr")
        shutil.make_archive("hallr", "zip", root_dir=base_dir, base_dir="hallr")
        shutil.move("hallr", export_dir)
        print("Created a new hallr.zip file.")
        print(f"Use blender to open the add-on: {os.path.join(base_dir, 'hallr.zip')}")


if __name__ == "__main__":
    args = parse_arguments()
    validate_rust_project()
    run_cargo_build(args.dev_mode, args.debug, args.generate_tests)
    staging_area = "blender_addon_exported"
    dest_lib_directory = os.path.join(staging_area, "lib")
    copy_library_files(args.dev_mode,args.debug, dest_lib_directory)
    copy_python_files("blender_addon", staging_area)
    process_python_files(staging_area, args.dev_mode)
    package_addon(args.dev_mode)
