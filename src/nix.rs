use crate::error::DebNixError;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, process::Command};

lazy_static::lazy_static! {
    /// Attribute names, that are exposed for the x86_64-linux
    /// platform on the nixpkgs side.
    pub static ref NIX_ATTRIBUTES_NEW: HashMap<String, NixAttributes> = {
        let output = Command::new("nix-env")
            .arg("--query")
            .arg("--json")
            .arg("--attr-path")
            .arg("--available")
            // .arg("--out-path")
            .arg("--drv-path")
            .output()
            .expect("Nix-env is broken, maybe there is a problem with the channel.");
    let serialized = std::str::from_utf8(&output.stdout)
            .expect("NIX_ATTRIBUTES are not generated correctly.");
    let deserialized: HashMap<String, NixAttributes> = serde_json::from_str(serialized)
            .expect("Serializing from NIX_ATTRIBUTES broken.");
        // deserialized.iter_mut().map(|(mut k, _v)| k = &k.split_once('.').unwrap().1.to_owned()).collect::<Vec<_>>();
        deserialized.iter().map(|(k, v)| (k.split_once('.').unwrap().1.to_owned(), v.clone())).collect::<HashMap<String, NixAttributes>>()
    };
    pub static ref NIX_ATTRIBUTES: Vec<String> = {
        NIX_ATTRIBUTES_NEW.values().map(|a| a.pname.clone()).collect()
    };
    pub static ref NIX_ATTRIBUTES_REVERSED: HashMap<String, NixAttributes> = {
        let mut result = HashMap::new();
        for (k, v) in NIX_ATTRIBUTES_NEW.iter() {
            result.insert(v.pname.clone(), NixAttributes {
                attrpath: Some(k.to_owned()),
                name: v.name.clone(),
                pname: v.pname.clone(),
                version: v.version.clone(),
            });
        }
            result
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixAttributes {
    name: String,
    pname: String,
    version: String,
    pub attrpath: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Wraps the derivation output from `nix`,
/// only few attributes are actually captured.
pub(crate) struct SimpleDerivation {
    env: DerivationEnv,
}

impl SimpleDerivation {
    pub(crate) fn env(&self) -> &DerivationEnv {
        &self.env
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Surfaces `DerivationEnv` attributes
pub(crate) struct DerivationEnv {
    pname: Option<String>,
    #[serde(rename = "buildInputs")]
    build_inputs: Option<String>,
    #[serde(rename = "nativeBuildInputs")]
    native_build_inputs: Option<String>,
    #[serde(rename = "propagatedBuildInputs")]
    propagated_build_inputs: Option<String>,
    #[serde(rename = "propagatedNativeBuildInputs")]
    propagated_native_build_inputs: Option<String>,
}

impl DerivationEnv {
    pub(crate) fn pname(&self) -> Option<&String> {
        self.pname.as_ref()
    }

    pub(crate) fn build_inputs(&self) -> Option<&String> {
        self.build_inputs.as_ref()
    }

    pub(crate) fn native_build_inputs(&self) -> Option<&String> {
        self.native_build_inputs.as_ref()
    }

    pub(crate) fn propagated_build_inputs(&self) -> Option<&String> {
        self.propagated_build_inputs.as_ref()
    }

    pub(crate) fn propagated_native_build_inputs(&self) -> Option<&String> {
        self.propagated_native_build_inputs.as_ref()
    }
}

/// Wraps the nix command in order to surface information about derivations that
/// make up a certain package.
pub(crate) fn find_package_info(pkgs: &str) -> Result<SimpleDerivation, DebNixError> {
    let output = if pkgs.starts_with('/') {
        Command::new("nix")
            .arg("show-derivation")
            .arg(pkgs)
            .output()?
    } else {
        Command::new("nix")
            .arg("show-derivation")
            .arg(format!("nixpkgs#legacyPackages.x86_64-linux.{}", pkgs))
            .output()?
    };

    if !output.status.success() {
        return Err(DebNixError::Nix(
            std::str::from_utf8(&output.stderr)?.to_string(),
        ));
    }

    let serialized = std::str::from_utf8(&output.stdout)?;
    let deserialized: HashMap<String, SimpleDerivation> = serde_json::from_str(serialized)?;
    let deserialized: SimpleDerivation = deserialized
        .into_values()
        .collect::<Vec<SimpleDerivation>>()
        .first()
        .unwrap()
        .clone();
    Ok(deserialized)
}

/// Collects all the `pnames` of the `buildInputs` and `nativeBuildInputs`
/// of a derivation into a Vec.
pub(crate) fn get_drv_inputs(pkgs: &str) -> Result<Vec<String>, DebNixError> {
    let derivation = find_package_info(pkgs)?;
    debug!("Nix derivation:\n {:?}", derivation);
    let mut inputs = vec![];
    let mut input_names = vec![];
    if let Some(drv) = derivation.env().build_inputs() {
        if !drv.is_empty() {
            inputs.extend(drv.split(' ').collect::<Vec<&str>>())
        }
    }
    if let Some(drv) = derivation.env().native_build_inputs() {
        if !drv.is_empty() {
            inputs.extend(drv.split(' ').collect::<Vec<&str>>())
        }
    }
    if let Some(drv) = derivation.env().propagated_build_inputs() {
        if !drv.is_empty() {
            inputs.extend(drv.split(' ').collect::<Vec<&str>>())
        }
    }
    if let Some(drv) = derivation.env().propagated_native_build_inputs() {
        if !drv.is_empty() {
            inputs.extend(drv.split(' ').collect::<Vec<&str>>())
        }
    }
    debug!("Nix inputs:\n {:?}", inputs);
    for drv in &inputs {
        debug!("Checking {:?}", &drv);
        let maybe_drv = find_package_info(drv);
        if let Ok(maybe_name) = maybe_drv {
            if let Some(name) = maybe_name.env().pname() {
                input_names.push(name.clone());
            }
        } else {
            error!("Error {:?}", &maybe_drv);
        }
    }
    Ok(input_names)
}
