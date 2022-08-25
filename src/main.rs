//! This program will match debian repository names with nix input names.
//!
//! The implementation currently uses the following heuristics for matching:
//! - exact matching & increasingly fuzzy matching
//! - querying of the debian pkg names in a packer instance
//! - matched libraries will be taken out of the potential matches
//!
//!
pub mod deb;
pub mod error;
pub mod matcher;
/// This module wraps the `nix` command.
/// And provides convenience functions.
pub mod nix;
use error::DebNixError;

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, process::Command};

use crate::{
    deb::{debian_redirect, download_control_file, parse_control_file, get_debian_deps},
    matcher::match_libs,
    nix::{find_package_info, get_drv_inputs},
};

#[derive(Parser)]
struct CliArgs {
    pkg: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct DebInputs {
    pkgs_name: Option<String>,
    pkgs_src: Option<String>,
    deb_name: Option<String>,
    deb_src: Option<String>,
    deb_inputs: Vec<String>,
    nix_inputs: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct DebNixOutputs {
    pkgs_name: Option<String>,
    pkgs_src: Option<String>,
    deb_name: Option<String>,
    deb_src: Option<String>,
    deb_inputs: Vec<String>,
    nix_inputs: Vec<String>,
    map: HashMap<String, String>,
}

static NIXEX: [&str; 31] = [
    "pkg-config",
    "makeWrapper",
    "meson",
    "ninja",
    "installShellFiles",
    "perl",
    "asciidoc",
    "xmlto",
    "docbook_xml_dtd_45",
    "docbook_xsl",
    "findXMLCatalogs",
    "libxcb",
    "xcbutilkeysyms",
    "xcbutil",
    "xcbutilwm",
    "xcbutilxrm",
    "libxkbcommon",
    "libstartup_notification",
    "libX11",
    "pcre",
    "libev",
    "yajl",
    "xcb-util-cursor",
    "perl",
    "pango",
    "perlPackages.AnyEventI3",
    "perlPackages.X11XCB perlPackages.IPCRun",
    "perlPackages.ExtUtilsPkgConfig",
    "perlPackages.InlineC",
    "xorgserver",
    "xvfb-run",
];

static DEBEX: [&str; 25] = [
    "debhelper",
    "meson",
    "libx11-dev",
    "libxcb-util0-dev",
    "libxcb-keysyms1-dev",
    "libxcb-xinerama0-dev",
    "libxcb-randr0-dev",
    "libxcb-icccm4-dev",
    "libxcb-cursor-dev",
    "libxcb-xrm-dev",
    "libxcb-xkb-dev",
    "libxcb-shape0-dev",
    "libxkbcommon-dev",
    "libxkbcommon-x11-dev",
    "asciidoc",
    "xmlto",
    "docbook-xml",
    "pkg-config",
    "libev-dev",
    "libyajl-dev",
    "libpcre2-dev",
    "libstartup-notification0-dev",
    "libcairo2-dev",
    "libpango1.0-dev",
    "libpod-simple-perl",
];

fn main() -> Result<(), DebNixError> {
    let opts = CliArgs::parse();

    if let Some(pkgs) = opts.pkg {
        let input_names = get_drv_inputs(&pkgs)?;
        println!("{:?}", input_names);
        println!("Nix Inputs Amount: {:?}", input_names.len());

        let deb_deps = get_debian_deps(&pkgs)?;
        println!("{:?}", &deb_deps);
        println!("Debian Dependency Amount: {:?}", &deb_deps.len());
        let result = match_libs(deb_deps, input_names);
        println!("{:?}", result);
        println!("Amount: {:?}", result.keys().len());
    } else {
        // let result = match_libs(DEBEX.as_ref().to_vec(), NIXEX.as_ref().to_vec());
        // println!("{:?}", result);
        // println!("Amount: {:?}", result.keys().len());
    };
    Ok(())
}
