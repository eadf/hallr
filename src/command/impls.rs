// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

//! A module containing boiler-plate implementations of standard traits such as Default, From etc etc

use crate::{HallrError, command::Options};
use std::collections::HashMap;

impl Options for HashMap<String, String> {
    /// Will return an option parsed as a `T` or an Err
    fn get_mandatory_parsed_option<'a, T: std::str::FromStr>(
        &'a self,
        key: &'a str,
        default: Option<T>,
    ) -> Result<T, HallrError> {
        match self.get(key) {
            Some(v) => match v.parse() {
                Ok(val) => Ok(val),
                Err(_) => Err(HallrError::InvalidParameter(format!(
                    "Invalid value for parameter {{\"{}\"}}: {{\"{}\"}}",
                    key, v
                ))),
            },
            None => {
                if let Some(default_value) = default {
                    Ok(default_value)
                } else {
                    Err(HallrError::MissingParameter(
                        format!("The mandatory parameter \"{key}\" was missing").to_string(),
                    ))
                }
            }
        }
    }

    /// Will return an option parsed as a `T` or None.
    /// If the option is missing None is returned, if it there but if it can't be parsed an error
    /// will be returned.
    fn get_parsed_option<'a, T: std::str::FromStr>(
        &'a self,
        key: &'a str,
    ) -> Result<Option<T>, HallrError> {
        match self.get(key) {
            Some(v) => match v.parse() {
                Ok(val) => Ok(Some(val)),
                Err(_) => Err(HallrError::InvalidParameter(format!(
                    "Invalid value for parameter {{\"{}\"}}: {{\"{}\"}}",
                    key, v
                ))),
            },
            None => Ok(None),
        }
    }

    /// Returns the &str value of an option, or an Err is it does not exists
    fn get_mandatory_option(&self, key: &str) -> Result<&str, HallrError> {
        match self.get(key) {
            Some(v) => Ok(v),
            None => Err(HallrError::MissingParameter(
                format!("The parameter {{\"{key}\"}} was missing").to_string(),
            )),
        }
    }

    /// Checks if an option exists
    fn does_option_exist(&self, key: &str) -> Result<bool, HallrError> {
        match self.get(key) {
            Some(_) => Ok(true),
            _ => Ok(false),
        }
    }
}
