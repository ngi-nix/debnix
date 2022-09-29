//! This program will match debian repository names with nix input names.
//!
//! The implementation currently uses the following heuristics for matching:
//! - exact matching & increasingly fuzzy matching
//! - querying of the debian pkg names in a tracker instance
//! - matched libraries will be taken out of the potential matches
//!
//!
/// The cli interface.
mod cli;
/// Query debian control files, and redirect pkg names.
pub mod deb;
/// Error handling.
pub mod error;
/// Matching package names.
pub mod matcher;
/// This module wraps the `nix` command.
/// And provides convenience functions.
pub mod nix;
/// Setup helpers.
pub mod setup;
pub mod state;

use chrono::Utc;
use error::DebNixError;
// use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use self::state::{create_output_map, State};
use self::{cli::CliArgs, deb::read_popcon};
use clap::Parser;

fn main() -> Result<(), DebNixError> {
    pretty_env_logger::init();
    let opts = CliArgs::parse();
    let state = State::from_opts(opts.clone())?;

    // Generate completion scripts
    if let Some(shell) = opts.generate_completion() {
        setup::generate_completion(&shell.to_string());
        std::process::exit(0);
    }

    // Query a single debian pkg name.
    if let Some(pkgs) = opts.pkg() {
        state.discover_package(pkgs.clone())?;
    };

    if let Some(location) = opts.generate_map() {
        create_output_map(location)?;
    }

    if let Some(amount) = state.discover() {
        let start_time = Utc::now().time();
        let pop = read_popcon("./test/popcon.csv")?;
        for (i, pkg) in pop
            .into_iter()
            .skip(state.discover_start().unwrap_or(0))
            .enumerate()
        {
            if i == amount {
                break;
            }
            if let Some(destination) = state.output() {
                create_dir_all(format!("{destination}/error"))?;
                let error_destination = format!("{}/error/{}", destination, pkg);
                let destination = format!("{}/{}-debnix.json", destination, pkg);
                // For now don't overwrite paths, but only create them once.
                if !Path::new(&destination).exists() && !Path::new(&error_destination).exists() {
                    match state.discover_package(pkg.clone()) {
                        Ok(outputs) => {
                            let serialized = serde_json::to_string(&outputs)?;
                            let mut file = File::create(&destination)?;
                            file.write_all(serialized.as_bytes())?;
                            error!("Written to location: {}", &destination);
                        }
                        Err(e) => {
                            error!("Discover Error: {}", e);
                            if let Ok(mut file) = File::create(&error_destination) {
                                file.write_all(e.to_string().as_bytes())?;
                                error!("Written to location: {}", &error_destination);
                            } else {
                                error!("Could not write to error location: {}", &error_destination);
                            }
                        }
                    }
                } else {
                    if Path::new(&destination).exists() {
                        error!("Path already exists: {}", &destination);
                    }
                    if Path::new(&error_destination).exists() {
                        error!("Path already exists: {}", &error_destination);
                    }
                }
            }
            if let Some(duration) = state.timeout() {
                let since_startup = Utc::now()
                    .time()
                    .signed_duration_since(start_time)
                    .num_minutes();

                if since_startup as usize > duration {
                    println!("Timeout Exceeded, shutting down.");
                    std::process::exit(0);
                }
            }
        }
    }
    Ok(())
}
