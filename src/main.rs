use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    "libpango1.0-dev,",
    "libpod-simple-perl",
];

fn main() {
    let result = match_libs(DEBEX.to_vec().as_ref(), NIXEX.to_vec().as_ref());
    println!("{:?}", result);
    println!("Amount: {:?}", result.keys().len());
}

fn match_libs(input: &[&str], output: &[&str]) -> HashMap<String, String> {
    let mut res_map = HashMap::new();
    let mut input = input.to_vec();
    let mut outputs = output.to_vec();

    // manual matching of the inputs
    input.retain(|lib| match match_inlib(lib, &mut outputs) {
        (true, None) => true,
        (true, Some(_)) => true,
        (false, None) => false,
        (false, Some(outlib)) => {
            res_map.insert(String::from(*lib), outlib.clone());
            outputs.retain(|lib| {
                if String::from(<&str>::clone(lib)) == outlib {
                    false
                } else {
                    true
                }
            });
            false
        }
    });
    // redirect the remaining packages and match them afterwards
    input.retain(|lib| {
        let (redirect, _original) = debian_redirect(&lib);
        match match_inlib(&redirect, &mut outputs) {
            (true, None) => true,
            (true, Some(_)) => true,
            (false, None) => false,
            (false, Some(outlib)) => {
                res_map.insert(String::from(*lib), outlib.clone());
                outputs.retain(|lib| {
                    if String::from(<&str>::clone(&lib)) == outlib {
                        false
                    } else {
                        true
                    }
                });
                false
            }
        }
    });
    // redirect the remaining packages and match them afterwards match remaining packages against
    // the full output and don't take pkgs out of the outputs (multiple binaries in one pkg)
    input.retain(|lib| {
        let mut outputs = output.to_vec();
        let (redirect, _original) = debian_redirect(&lib);
        match match_inlib(&redirect, &mut outputs) {
            (true, None) => true,
            (true, Some(_)) => true,
            (false, None) => false,
            (false, Some(outlib)) => {
                res_map.insert(String::from(*lib), outlib.clone());
                false
            }
        }
    });

    println!("\nInput {:?}\n", &input);
    println!("Output {:?}\n", &outputs);

    res_map
}

fn match_inlib(inlib: &str, outlibs: &mut [&str]) -> (bool, Option<String>) {
    use regex::Regex;
    // for version numbers
    let ve = Regex::new(r"\d(.\d*)*").unwrap();

    // exact match
    for outlib in &mut *outlibs {
        if inlib == *outlib {
            println!("{:?}", inlib);
            return (false, Some(outlib.to_string()));
        }
    }
    // replace `-dev`
    for outlib in &mut *outlibs {
        if inlib.replace("-dev", "") == *outlib {
            println!("{:?}", inlib);
            return (false, Some(outlib.to_string()));
        }
    }
    // replace `-dev` && lowercase
    for outlib in &mut *outlibs {
        if inlib.replace("-dev", "").to_lowercase() == *outlib.to_lowercase() {
            println!("{:?}", inlib);
            return (false, Some(outlib.to_string()));
        }
    }
    // replace `-dev` && lowercase && replace - _
    for outlib in &mut *outlibs {
        if inlib.replace("-dev", "").replace('-', "_").to_lowercase() == *outlib.to_lowercase() {
            println!("{:?}", inlib);
            return (false, Some(outlib.to_string()));
        }
    }
    // replace `-dev` && lowercase && replace - _ && replace lib
    for outlib in &mut *outlibs {
        if ve.replace_all(
            &inlib
                .replace("-dev", "")
                .replace('-', "_")
                .replace("lib", "")
                .to_lowercase(),
            "",
        ) == *outlib.to_lowercase()
        {
            println!("{:?}", inlib);
            return (false, Some(outlib.to_string()));
        }
    }
    // replace `-dev` && lowercase && replace - _ && don't replace lib
    for outlib in &mut *outlibs {
        if ve.replace_all(
            &inlib.replace("-dev", "").replace('-', "_").to_lowercase(),
            "",
        ) == *outlib.to_lowercase()
        {
            println!("{:?}", inlib);
            return (false, Some(outlib.to_string()));
        }
    }
    // replace `-dev` && lowercase && replace - "" && don't replace lib
    for outlib in outlibs {
        if ve.replace_all(
            &inlib
                .replace("-dev", "")
                .replace('-', "")
                .replace("lib", "")
                .to_lowercase(),
            "",
        ) == *outlib.to_lowercase()
        {
            println!("{:?}", inlib);
            return (false, Some(outlib.to_string()));
        }
    }
    (true, None)
}

fn debian_redirect(lib: &str) -> (String, String) {
    let tracker_site = "https://tracker.debian.org/pkg/";
    let mut tracker_site = String::from(tracker_site);
    tracker_site.push_str(lib);
    let resp = reqwest::blocking::get(tracker_site).unwrap();

    let pkgs = resp.url().path();
    let pkg = pkgs.rsplit_once("/pkg/").unwrap().1;
    // println!("{:?}", pkg);
    (String::from(pkg), String::from(lib))
}
