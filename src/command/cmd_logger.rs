// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use super::{ConfigType, Model};
use crate::HallrError;
use std::{
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

/// This is a command that peeks at incoming data and writes it to file
#[allow(dead_code)]
pub(crate) fn process_command(config: &ConfigType, models: &[Model<'_>]) -> Result<(), HallrError> {
    let log_dir = match env::var("HALLR_DATA_LOGGER_PATH") {
        Ok(path) => PathBuf::from(path),
        Err(_) => return Ok(()), // Silently return if env var not set
    };

    // Check directory status (exists + writable)
    if !fs::metadata(&log_dir)?.is_dir() || fs::metadata(&log_dir)?.permissions().readonly() {
        eprintln!(
            "The {:?} did not exists or is not writable, skipping logging",
            log_dir.to_str()
        );
        return Ok(());
    }

    let file_name = {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("🕰️ Whoa man, the clock's trippin'!")
            .as_nanos();
        format!("{:x}", timestamp)
    };
    let base_file_name = log_dir.join(file_name);
    {
        let log_file_name = base_file_name.with_extension("txt");
        let mut file = OpenOptions::new()
            .create(true) // Create file if it doesn't exist
            .write(true) // Open for writing
            .truncate(true) // Clear existing content
            .open(&log_file_name)?;
        write!(file, "{:?}", config)?;
        println!("Rust: logging input as {}", log_file_name.to_str().unwrap());
    }

    for (n, model) in models.iter().enumerate() {
        let model_name = format!("model{}", n);
        let model_filename = base_file_name.with_extension(format!("{}.obj", n));
        let obj = hronn::obj::Obj::new_from_triangles(
            model_name.as_str(),
            model.vertices.to_vec(),
            model.indices.to_vec(),
        );
        obj.write_obj(&model_filename)?;
        println!(
            "Rust: logging input as {}",
            model_filename.to_str().unwrap()
        );
    }
    Ok(())
}
