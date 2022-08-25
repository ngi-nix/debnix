use crate::error::DebNixError;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, process::Command};

#[derive(Debug, Clone, Serialize, Deserialize)]
// A wrapped around the derivation output from nix
pub(crate) struct SimpleDerivation {
    env: DerivationEnv,
}

impl SimpleDerivation {
    pub(crate) fn env(&self) -> &DerivationEnv {
        &self.env
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DerivationEnv {
    pname: Option<String>,
    #[serde(rename = "buildInputs")]
    build_inputs: Option<String>,
    #[serde(rename = "nativeBuildInputs")]
    native_build_inputs: Option<String>,
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
}

///! Wraps the nix command in order to surface information about derivations that
///! make up a certain package.
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

///! Collect's all the `pnames` of the `buildInputs` and `nativeBuildInputs`
///! of a derivation into a Vec.
pub(crate) fn get_drv_inputs(pkgs: &str) -> Result<Vec<String>, DebNixError> {
    let derivation = find_package_info(pkgs)?;
    let mut inputs = vec![];
    let mut input_names = vec![];
    inputs.extend(
        derivation
            .env()
            .build_inputs()
            .unwrap()
            .split(' ')
            .collect::<Vec<&str>>(),
    );
    inputs.extend(
        derivation
            .env()
            .native_build_inputs()
            .unwrap()
            .split(' ')
            .collect::<Vec<&str>>(),
    );
    println!("{:?}", inputs);
    for drv in &inputs {
        println!("Checking {:?}", &drv);
        let maybe_drv = find_package_info(drv);
        if let Ok(maybe_name) = maybe_drv {
            if let Some(name) = maybe_name.env().pname() {
                input_names.push(name.clone());
            }
        } else {
            println!("Error {:?}", &maybe_drv);
        }
    }
    Ok(input_names)
}
