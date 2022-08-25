//! This program will match debian repository names with nix input names.
//!
//! The implementation currently uses the following heuristics for matching:
//! - exact matching & increasingly fuzzy matching
//! - querying of the debian pkg names in a packer instance
//! - matched libraries will be taken out of the potential matches
//!
//!
pub mod consts;
pub mod deb;
pub mod error;
pub mod matcher;
/// This module wraps the `nix` command.
/// And provides convenience functions.
pub mod nix;

use clap::Parser;
use error::DebNixError;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use self::{consts::POP, deb::get_debian_pkg_outputs};
use crate::{deb::get_debian_deps, matcher::match_libs, nix::get_drv_inputs};

#[derive(Parser)]
struct CliArgs {
    pkg: Option<String>,
    #[clap(long, value_parser)]
    discover: Option<usize>,
    #[clap(long, value_parser)]
    discover_start: Option<usize>,
    #[clap(long, value_parser)]
    write: Option<String>,
    #[clap(long, value_parser)]
    read_popcon: Option<String>,
}

/// outputs/toplevel-debnix.json
/// HashMap {deb-lib: nix-lib}
/// outputs/toplevel-nixdeb.json
///
/// outputs/i3/i3-debnix.json
#[derive(Serialize, Deserialize)]
pub(crate) struct DebNixOutputs {
    pkgs_name: Option<String>,
    // pkgs_src: Option<String>,
    // deb_name: Option<String>,
    // deb_src: Option<String>,
    // deb_inputs: Vec<String>,
    // nix_inputs: Vec<String>,
    map: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Popcon {
    name: Option<String>,
}

fn main() -> Result<(), DebNixError> {
    pretty_env_logger::init();
    let opts = CliArgs::parse();

    if let Some(pkgs) = opts.pkg {
        let map = discover(pkgs.clone())?;
        if let Some(destination) = opts.write.clone() {
            let out = DebNixOutputs {
                pkgs_name: Some(pkgs),
                map,
            };
            let serialized = serde_json::to_string(&out)?;
            let mut file = File::create(destination)?;
            file.write_all(serialized.as_bytes())?;
        }
    };

    if let Some(location) = opts.read_popcon {
        let contents =
            fs::read_to_string(location).expect("Should have been able to read the file");
        let mut rdr = csv::Reader::from_reader(contents.as_bytes());
        for result in rdr.deserialize() {
            // Notice that we need to provide a type hint for automatic
            // deserialization.
            let record: Popcon = result.unwrap();
            println!("{:?}", record);
        }
    }

    if let Some(amount) = opts.discover {
        for (i, pkg) in POP.into_iter().enumerate() {
            if i == amount {
                break;
            }
            if let Some(destination) = opts.write.clone() {
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
                            // let destination = format!("{}/{}-debnix.json", destination, pkg);
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
    // let input_names = get_drv_inputs(&pkgs)?;
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
