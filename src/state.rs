use crate::cli::CliArgs;
use crate::deb::{get_debian_pkg_outputs, ControlFileApi};
use crate::error::DebNixError;
use crate::matcher::match_libs;
use crate::nix::{get_drv_inputs, NIX_ATTRIBUTES_REVERSED};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{fs::File, io::Write, path::Path};

/// outputs/toplevel-debnix.json
/// HashMap {deb-lib: nix-lib}
/// outputs/toplevel-nixdeb.json
///
/// outputs/i3/i3-debnix.json
#[derive(Debug, Serialize, Deserialize)]
pub struct DebNixOutputs {
    pkgs_name: Option<String>,
    // pkgs_src: Option<String>,
    // deb_name: Option<String>,
    nix_pkg: Option<String>,
    control_file_hash: Option<String>,
    deb_inputs: Vec<String>,
    nix_inputs: Vec<String>,
    map: HashMap<String, String>,
}

#[derive(Debug)]
pub(crate) struct State {
    map: Option<HashMap<String, String>>,
    discover: Option<usize>,
    discover_start: Option<usize>,
    timeout: Option<usize>,
    output: Option<String>,
}

impl State {
    pub(crate) fn from_opts(opts: CliArgs) -> Result<Self, DebNixError> {
        let map = if let Some(location) = opts.map() {
            Some(open_map(location)?)
        } else {
            None
        };
        Ok(Self {
            map,
            discover: opts.discover(),
            discover_start: opts.discover_start(),
            timeout: opts.timeout(),
            output: opts.output(),
        })
    }

    pub fn discover_package(&self, pkg_name: String) -> Result<(), DebNixError> {
        // let outputs = discover(pkgs.clone(), state.map.clone())?;
        let outputs = self.discover_pkg(pkg_name)?;
        if let Some(destination) = &self.output {
            let serialized = serde_json::to_string(&outputs)?;
            let mut file = File::create(destination)
                .map_err(|e| DebNixError::IoPath(format!("{e}: {destination}")))?;
            file.write_all(serialized.as_bytes())?;
        }
        Ok(())
    }

    /// This is the main discovery function, for a single package.
    pub fn discover_pkg(&self, pkg: String) -> Result<DebNixOutputs, DebNixError> {
        // Prepare possible names for nix pkgs definitions.
        let mut nix_inputs = vec![];
        let mut nix_pkg = None;

        if let Some(attr_path) = NIX_ATTRIBUTES_REVERSED.get(&pkg) {
            nix_pkg = Some(attr_path.attrpath.clone()).flatten();
        }

        nix_inputs.push(pkg.clone());
        let mut unwrapped = pkg.clone();
        unwrapped.push_str("-unwrapped");
        nix_inputs.push(unwrapped);

        if let Some(map) = self.map() {
            // Lookup in the provided map for an associated pkg name
            if let Some(pkg) = map.get(&pkg) {
                nix_inputs.push(pkg.to_string())
            }
        }
        // Get the debian pkg outputs
        if let Ok(deb_inputs) = get_debian_pkg_outputs(&pkg) {
            nix_inputs.extend(deb_inputs);
        };
        let input_names = drv_inputs_from_pkgs(nix_inputs)?;
        info!("{:?}", input_names);
        info!("Nix Inputs Amount: {:?}", input_names.len());

        // Get the control file api for the specific package
        info!("Getting Control file for {:?}", &pkg);
        let control_file_api = ControlFileApi::from_redirect(&pkg)?;
        let control_file_hash =
            String::from(control_file_api.checksum().ok_or_else(|| {
                DebNixError::DebControl("Couldn't get Control file Hash.".into())
            })?);
        let mut deb_deps = control_file_api.get_debian_deps()?;
        deb_deps.sort();
        deb_deps.dedup();
        info!("{:?}", &deb_deps);
        info!("Debian Dependency Amount: {:?}", &deb_deps.len());
        let result = match_libs(deb_deps.clone(), input_names.clone())?;
        info!("Amount: {:?}", result.keys().len());
        Ok(DebNixOutputs {
            pkgs_name: Some(pkg),
            nix_pkg,
            control_file_hash: Some(control_file_hash),
            deb_inputs: deb_deps,
            nix_inputs: input_names,
            map: result,
        })
    }

    pub(crate) fn output(&self) -> Option<&String> {
        self.output.as_ref()
    }

    pub(crate) fn map(&self) -> Option<&HashMap<String, String>> {
        self.map.as_ref()
    }

    pub(crate) fn discover(&self) -> Option<usize> {
        self.discover
    }

    pub(crate) fn timeout(&self) -> Option<usize> {
        self.timeout
    }

    pub(crate) fn discover_start(&self) -> Option<usize> {
        self.discover_start
    }
}

/// Try to get the inputs of a derivation from multiple possible pkg names
/// TODO: pass in a vec of possible pkgs from outside.
fn drv_inputs_from_pkgs(pkgs: Vec<String>) -> Result<Vec<String>, DebNixError> {
    let mut inputs = vec![];

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

/// Reads the provided output json's and creates a single json file
/// for easy key value lookups.
pub fn create_output_map(_location: &str) -> Result<(), DebNixError> {
    use std::io::Read;
    let mut result: HashMap<String, String> = HashMap::new();
    let outputs = Path::new("./outputs");
    for output in outputs.read_dir()?.flatten() {
        if output.file_type()?.is_file() {
            let mut file = File::open(output.path())?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            if let Ok(deserialized) = serde_json::from_str::<DebNixOutputs>(&contents) {
                if let Some(deb_name) = deserialized.pkgs_name {
                    if let Some(nix_name) = deserialized.nix_pkg {
                        result.insert(deb_name.to_string(), nix_name.to_string());
                    }
                }
                for key in deserialized.map.keys() {
                    if let Some((_, values)) = deserialized.map.get_key_value(key) {
                        result.insert(key.to_string(), values.to_string());
                    }
                }
            } else {
                error!("Reading: {:?}", output.path());
            }
        }
    }
    // write the result map to the target location
    let serialized = serde_json::to_string(&result)?;
    let target_destination = "./outputs/maps/debnix.json";
    let mut file = File::create(target_destination)
        .map_err(|e| DebNixError::IoPath(format!("{e}: {target_destination}")))?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

/// Open a json map from a specified location and read it.
fn open_map(location: &str) -> Result<HashMap<String, String>, DebNixError> {
    use std::io::Read;

    let mut file = File::open(location)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(serde_json::from_str::<HashMap<String, String>>(&contents)?)
}
