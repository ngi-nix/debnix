use crate::error::DebNixError;

/// Uses the redirect functionality of `tracker.debian.org` in order to find out
/// the build package that a package output, or a package output + version that
/// is surfaced from itself.
pub(crate) fn debian_redirect(pkgs: &str) -> Result<String, DebNixError> {
    let tracker_site = "https://tracker.debian.org/pkg/";
    let mut tracker_site = String::from(tracker_site);
    tracker_site.push_str(pkgs.trim());
    let resp = reqwest::blocking::get(tracker_site)?;
    let pkgs = resp.url().path();
    let pkg = pkgs.rsplit_once("/pkg/").unwrap().1;
    Ok(String::from(pkg))
}

/// Get's the latest version of a package that is surfaced in debian,
/// relies on a redirect from `sources.debian`.
pub(crate) fn get_unstable_version(pkg: &str) -> Result<String, DebNixError> {
    let debian_sources = format!("https://sources.debian.org/src/{}/unstable/", pkg);
    let resp = reqwest::blocking::get(&debian_sources)?;
    let version_path = resp.url().path();
    let version: String = version_path.split('/').rev().take(2).collect();
    Ok(version)
}

/// Downloads the control file directly from `sources.debian`.
pub(crate) fn download_control_file(pkg: &str) -> Result<String, DebNixError> {
    let version = get_unstable_version(pkg).unwrap();

    let prefix = if pkg.starts_with("lib") {
        pkg.chars().take(4).collect::<String>()
    } else {
        pkg.chars().take(1).collect::<String>()
    };

    let control_file_location = format!(
        "https://sources.debian.org/data/main/{}/{}/{}/debian/control",
        prefix, &pkg, &version
    );
    match reqwest::blocking::get(control_file_location) {
        Ok(resp) => Ok(resp.text()?),
        Err(e) => {
            error!("\nThis location doesn't work \n{}", e);
            Err(DebNixError::Reqwest(e))
        }
    }
}

/// Parses the various dependencies that a control file exposes.
pub(crate) fn parse_control_file_dependencies(content: &str) -> Result<Vec<String>, DebNixError> {
    let par = debcontrol::parse_str(content).map_err(|e| {
        DebNixError::DebControl(format!(
            "Control File could not be parsed: {}, {}",
            content, e
        ))
    })?;
    let mut result = vec![];
    debug!("Full control file: \n {:?}", &par);

    for paragraph in par {
        for field in &paragraph.fields {
            match field.name {
                "Build-Depends" | "Depends" | "Recommends" | "Suggests" => {
                    result.extend(parse_control_value(&field.value).unwrap());
                }
                _ => {}
            }
        }
    }
    Ok(result)
}

/// Parses a control file, in order to surface the pkgs that are provided by the package.
pub(crate) fn parse_control_file_pkgs(content: &str) -> Result<Vec<String>, DebNixError> {
    let par = debcontrol::parse_str(content).map_err(|e| {
        DebNixError::DebControl(format!(
            "Control File could not be parsed: {}, {}",
            content, e
        ))
    })?;
    let mut result = vec![];
    debug!("{:?}", &par);

    for paragraph in par {
        for field in &paragraph.fields {
            if let "Package" = field.name {
                result.extend(parse_control_value(&field.value).unwrap());
            }
            debug!("This was not included as a pkg output:\n {:?}", &field);
        }
    }
    Ok(result)
}

/// Parses control values, cleans them and returns them.
fn parse_control_value(value: &str) -> Result<Vec<String>, ()> {
    use regex::Regex;
    // Remove version numbers
    let ve = Regex::new(r"\(([^\)]+)\)").unwrap();
    // Remove "<>"
    let ve_angle = Regex::new(r"<([^\)]+)>").unwrap();
    // Remove "${}"
    let ve_curly = Regex::new(r"\$\{([^\)]+)\}").unwrap();
    // Remove "[]"
    let ve_square = Regex::new(r"\[([^\)]+)\]").unwrap();

    let mut result = vec![];
    let values = value.split(',').collect::<Vec<&str>>();
    for value in &values {
        let value = value.trim_matches('\n');
        let value = ve.replace_all(value, "");
        let value = ve_angle.replace_all(&value, "");
        let value = ve_curly.replace_all(&value, "");
        let value = ve_square.replace_all(&value, "");
        let value = value.trim();
        let optional_values = value.split('|').collect::<Vec<&str>>();
        for optional_value in &optional_values {
            let optional_value = optional_value.trim();
            if !optional_value.is_empty() {
                result.push(String::from(optional_value));
            }
        }
        // if !value.is_empty() {
        //     result.push(String::from(value));
        // }
    }
    Ok(result)
}

pub(crate) fn get_debian_deps(pkgs: &str) -> Result<Vec<String>, DebNixError> {
    let pkgs = debian_redirect(pkgs)?;
    let download_control_file = download_control_file(&pkgs)?;
    let parsed_control_file = parse_control_file_dependencies(&download_control_file)?;
    debug!("Parsed Control File: {:?}", &parsed_control_file);
    Ok(parsed_control_file)
}

pub(crate) fn get_debian_pkg_outputs(pkgs: &str) -> Result<Vec<String>, DebNixError> {
    let pkgs = debian_redirect(pkgs)?;
    let download_control_file = download_control_file(&pkgs)?;
    let parsed_control_file = parse_control_file_pkgs(&download_control_file)?;
    debug!("Parsed Control File: {:?}", &parsed_control_file);
    Ok(parsed_control_file)
}
