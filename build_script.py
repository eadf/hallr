#!/usr/bin/env python3
"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

# This script prepares and packages the blender addon files under the
# `blender_addon_exported` folder.

if __name__ == "__main__":
    import subprocess
    import time
    import os
    import platform
    import shutil
    import re
    import glob
    import argparse

    parser = argparse.ArgumentParser(description="A script that packages the hallr blender addon files")

    parser.add_argument(
        "--dev_mode",
        action="store_true",
        help="Enable development mode, in this mode you can open the ’__init__.py’ file in blender and run it there, directly. " +
             "You can also re-run it without having to recreate the zip file every time."
    )
    parser.add_argument(
        "--release",
        action="store_true",
        help="Build in release mode when --dev_mode is active."
    )
    args = parser.parse_args()

    # Check if the current directory is a Rust project
    if not os.path.isfile("Cargo.toml"):
        print("Error: This directory does not contain a Cargo.toml file.")
        exit(1)

    # Validate the Cargo.toml file to ensure it's the correct Rust project
    with open("Cargo.toml", "r") as f:
        content = f.read()
        if "name = \"hallr\"" not in content:
            print("Error: The Cargo.toml file does not specify the project name as 'hallr'. Are you in the correct cwd?")
            exit(1)

    # Run the cargo build command
    if args.dev_mode and not args.release:
        build_command = ["cargo", "build"]
    else:
        build_command = ["cargo", "build", "--release"]
    result = subprocess.run(build_command)
    # result = subprocess.run(["cargo", "build", "--release", "--features", "display_sdf_chunks"])

    # Check the return status
    if result.returncode != 0:
        print(f"Cargo command failed with return code {result.returncode}.")
        exit(1)

    # Get the current timestamp
    timestamp = str(int(time.time()))

    # Determine the library extension based on the platform
    system = platform.system()
    library_extension = ".dylib"  # Default to macOS

    if system == "Linux":
        library_extension = ".so"
    elif system == "Windows":
        library_extension = ".dll"

    source_directory = 'blender_addon'
    destination_directory = 'blender_addon_exported'
    dest_lib_directory = os.path.join(destination_directory, "lib")

    # Check if the destination directory exists, and create it if not
    os.makedirs(destination_directory, exist_ok=True)
    # Ensure the directory exists or create it if it doesn't
    os.makedirs(dest_lib_directory, exist_ok=True)

    # Rename and move the library file
    if args.dev_mode and not args.release:
        target_dir = "target/debug"
    else:
        target_dir = "target/release"

    lib_files = [f for f in os.listdir(target_dir) if f.startswith("libhallr") and f.endswith(library_extension)]
    if len(lib_files) == 0:
        print(f"Could not find the libfile in ´{target_dir}´.")
        exit(1)

    if args.dev_mode:
        old_lib_files = [f for f in os.listdir(dest_lib_directory) if
                         f.startswith("libhallr_") and f.endswith(library_extension)]
        for lib_file in old_lib_files:
            old_file = os.path.join(dest_lib_directory, lib_file)
            os.remove(old_file)

    for lib_file in lib_files:
        if args.dev_mode:
            new_name = os.path.join(dest_lib_directory, f"libhallr_{timestamp}{library_extension}")
        else:
            new_name = os.path.join(dest_lib_directory,lib_file)
        if os.path.exists(new_name):
            os.chmod(new_name, 0o666)  # Make writable before overwrite
        shutil.copy(f"{target_dir}/{lib_file}", new_name)

    file_extension = '.py'

    # Get a list of all files with the specified extension in the source directory
    source_files = glob.glob(f"{source_directory}/*{file_extension}")

    # Copy each selected file to the destination directory
    for source_file in source_files:
        new_name = os.path.join(destination_directory, os.path.basename(source_file))
        if os.path.exists(new_name):
            os.chmod(new_name, 0o666)  # Make writable before overwrite
        # Use shutil.copy to copy the file
        shutil.copy(source_file, new_name)

    base_directory = os.getcwd()  # Get the current working directory

    # Paths to be replaced
    addon_exported_path = os.path.join(base_directory, 'blender_addon_exported')
    target_release_path = os.path.join(base_directory, addon_exported_path, 'lib')

    # List files in the directory (non-recursively)
    file_list = [f for f in os.listdir(addon_exported_path) if
                 os.path.isfile(os.path.join(addon_exported_path, f)) and f.endswith(".py")]

    # Do find and replace on the .py files
    for file in file_list:
        file_path = os.path.join(addon_exported_path, file)
        with open(file_path, 'r') as f:
            content = f.read()
        content = re.sub(r'HALLR__BLENDER_ADDON_PATH', addon_exported_path, content)
        content = re.sub(r'HALLR__TARGET_RELEASE', target_release_path, content)
        if args.dev_mode:
            content = re.sub(r'DEV_MODE = False', 'DEV_MODE = True', content)
            content = re.sub(r'from . import', 'import', content)

        with open(file_path, 'w') as f:
            f.write(content)

    # Set all files in the exported directory to read-only
    for root, _, files in os.walk(addon_exported_path):
        for file in files:
            file_path = os.path.join(root, file)
            os.chmod(file_path, 0o444)  # Read-only for everyone

    if not args.dev_mode:
        subprocess.run("mv blender_addon_exported hallr", shell=True)
        subprocess.run("zip -r hallr.zip hallr", shell=True)#,cwd=addon_exported_path)
        subprocess.run("mv hallr blender_addon_exported", shell=True)
        print("Created a new hallr.zip file in the root, install it as an addon in blender.")
    else:
        print("Updated the files under blender_addon_exported, use blender to run __init__.py")
