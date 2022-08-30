//! This program will match debian repository names with nix input names.
//!
//! The implementation currently uses the following heuristics for matching:
//! - exact matching & increasingly fuzzy matching
//! - querying of the debian pkg names in a packer instance
//! - matched libraries will be taken out of the potential matches
//!
//!
/// The cli interface.
mod cli;
/// Application constants.
pub mod consts;
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

use error::DebNixError;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use std::{
    fs::{self, File},
    io::Write,
};

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use self::cli::CliArgs;
use self::deb::get_debian_pkg_outputs;
use crate::{deb::get_debian_deps, matcher::match_libs, nix::get_drv_inputs};
use clap::Parser;

/// outputs/toplevel-debnix.json
/// HashMap {deb-lib: nix-lib}
/// outputs/toplevel-nixdeb.json
///
/// outputs/i3/i3-debnix.json
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DebNixOutputs {
    pkgs_name: Option<String>,
    // pkgs_src: Option<String>,
    // deb_name: Option<String>,
    // deb_src: Option<String>,
    // deb_inputs: Vec<String>,
    // nix_inputs: Vec<String>,
    map: HashMap<String, String>,
}

fn main() -> Result<(), DebNixError> {
    pretty_env_logger::init();
    let opts = CliArgs::parse();

    // Generate completion scripts
    if let Some(shell) = opts.generate_completion() {
        setup::generate_completion(&shell.to_string());
        std::process::exit(0);
    }

    if let Some(pkgs) = opts.pkg() {
        let map = discover(pkgs.clone())?;
        if let Some(destination) = opts.write() {
            let out = DebNixOutputs {
                pkgs_name: Some(pkgs.to_string()),
                map,
            };
            let serialized = serde_json::to_string(&out)?;
            let mut file = File::create(destination)?;
            file.write_all(serialized.as_bytes())?;
        }
    };

    if let Some(location) = opts.read_popcon() {
        let result = read_popcon(location);
        // println!("{:?}", result);
    }

    if let Some(location) = opts.generate_map() {
        let result = create_output_map(location)?;
        // println!("{:?}", result);
    }

    if let Some(amount) = opts.discover() {
        let pop = read_popcon("./test/popcon.csv")?;
        // println!("{:?}", &pop);
        for (i, pkg) in pop.into_iter().enumerate() {
            if i == amount {
                break;
            }
            if let Some(destination) = opts.write() {
                let error_destination = format!("{}/error/{}", destination, pkg);
                let destination = format!("{}/{}-debnix.json", destination, pkg);
                // For now don't overwrite paths, but only create them once.
                if !Path::new(&destination).exists() && !Path::new(&error_destination).exists() {
                    match discover(pkg.to_string()) {
                        Ok(map) => {
                            let out = DebNixOutputs {
                                pkgs_name: Some(pkg.to_string()),
                                map,
                            };
                            let serialized = serde_json::to_string(&out)?;
                            let mut file = File::create(&destination)?;
                            file.write_all(serialized.as_bytes())?;
                            error!("Written to location: {}", &destination);
                        }
                        Err(e) => {
                            error!("Discover Error: {}", e);
                            let mut file = File::create(&error_destination)?;
                            file.write_all(e.to_string().as_bytes())?;
                            error!("Written to location: {}", &error_destination);
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
        }
    }
    Ok(())
}

/// Try to get the inputs of a derivation from multiple possible pkg names
fn drv_inputs_from_pkgs(pkg: String) -> Result<Vec<String>, DebNixError> {
    let mut inputs = vec![];
    let mut pkgs = vec![];
    if let Ok(deb_inputs) = get_debian_pkg_outputs(&pkg) {
        pkgs.extend(deb_inputs);
    };
    pkgs.push(pkg.clone());
    let mut unwrapped = pkg;
    unwrapped.push_str("-unwrapped");
    pkgs.push(unwrapped);

    for pkg in pkgs {
        let input_names = get_drv_inputs(&pkg);
        match input_names {
            Ok(names) => {
                inputs.extend(names);
            }
            Err(e) => match e {
                DebNixError::Nix(e) => {
                    debug!("{}", e);
                }
                e => return Err(e),
            },
        }
    }
    inputs.sort();
    inputs.dedup();
    Ok(inputs)
}

fn discover(pkgs: String) -> Result<HashMap<String, String>, DebNixError> {
    let input_names = drv_inputs_from_pkgs(pkgs.clone())?;
    info!("{:?}", input_names);
    info!("Nix Inputs Amount: {:?}", input_names.len());

    let deb_deps = get_debian_deps(&pkgs)?;
    info!("{:?}", &deb_deps);
    info!("Debian Dependency Amount: {:?}", &deb_deps.len());
    let result = match_libs(deb_deps, input_names)?;
    info!("Amount: {:?}", result.keys().len());
    Ok(result)
}

/// Reads the pkgs from a popcon (popularity contest) file
/// and then collects the pkgs inside of a Vec
fn read_popcon(location: &str) -> Result<Vec<String>, DebNixError> {
    let mut popcon = vec![];
    let contents = fs::read_to_string(location)?;
    let mut rdr = csv::Reader::from_reader(contents.as_bytes());
    for result in rdr.records().flatten() {
        if let Some(record) = result.get(0) {
            if !record.starts_with('#') {
                let name = record
                    .split(' ')
                    .into_iter()
                    .skip(1)
                    .take(2)
                    .collect::<String>();
                let name = name.trim();
                if !name.is_empty() {
                    popcon.push(name.into());
                }
            }
        }
    }
    Ok(popcon)
}

/// Reads the provided output json's and creates a single json file
/// for easy key value lookups.
fn create_output_map(location: &str) -> Result<(), DebNixError> {
    // - Read from input maps
    // - Insert into result
    // - Write output map
    use std::io::Read;
    let mut result: HashMap<String, String> = HashMap::new();
    let outputs = Path::new("./outputs");
    for output in outputs.read_dir()?.flatten() {
            if output.file_type()?.is_file() {
                let mut file = File::open(output.path())?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                let deserialized: DebNixOutputs = serde_json::from_str(&contents)?;
                for key in deserialized.map.keys() {
                    if let Some((_, values)) = deserialized.map.get_key_value(key) {
                        result.insert(key.to_string(), values.to_string());
                    }
            }
        }
    }
    println!("{:?}", result);
    // write the result map to the target location
    let serialized = serde_json::to_string(&result)?;
    let mut file = File::create("./outputs/maps/debnix.json")?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}
