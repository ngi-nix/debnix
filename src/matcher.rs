use std::collections::HashMap;

use crate::deb::debian_redirect;

/// Matches the input libraries with the output libraries
pub(crate) fn match_libs(input: Vec<String>, output: Vec<String>) -> HashMap<String, String> {
    let mut res_map = HashMap::new();
    let mut input = input.to_vec();
    let mut outputs = output.to_vec();

    // manual matching of the inputs
    input.retain(|lib| match match_inlib(lib, &mut outputs) {
        (true, None) => true,
        (true, Some(_)) => true,
        (false, None) => false,
        (false, Some(outlib)) => {
            res_map.insert(lib.clone(), outlib.clone());
            outputs.retain(|lib| *lib != outlib);
            false
        }
    });
    // redirect the remaining packages and match them afterwards
    input.retain(|lib| {
        let redirect = debian_redirect(lib).unwrap();
        match match_inlib(&redirect, &mut outputs) {
            (true, None) => true,
            (true, Some(_)) => true,
            (false, None) => false,
            (false, Some(outlib)) => {
                res_map.insert(lib.to_string(), outlib.clone());
                outputs.retain(|lib| lib != &outlib);
                false
            }
        }
    });
    // redirect the remaining packages and match them afterwards match remaining packages against
    // the full output and don't take pkgs out of the outputs (multiple binaries in one pkg)
    input.retain(|lib| {
        let mut outputs = output.to_vec();
        let redirect  = debian_redirect(lib).unwrap();
        match match_inlib(&redirect, &mut outputs) {
            (true, None) => true,
            (true, Some(_)) => true,
            (false, None) => false,
            (false, Some(outlib)) => {
                res_map.insert(String::from(lib), outlib);
                false
            }
        }
    });

    println!("\nInput {:?}\n", &input);
    println!("Output {:?}\n", &outputs);

    res_map
}

fn match_inlib(inlib: &str, outlibs: &mut Vec<String>) -> (bool, Option<String>) {
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
