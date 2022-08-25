use crate::error::DebNixError;

/// Uses the redirect functionality of `tracker.debian.org` in order to find out
/// the build package that a package output, or a package output + version that
/// is surfaced from itself.
pub(crate) fn debian_redirect(pkgs: &str) -> Result<String, DebNixError> {
    let tracker_site = "https://tracker.debian.org/pkg/";
    let mut tracker_site = String::from(tracker_site);
    tracker_site.push_str(pkgs);
    let resp = reqwest::blocking::get(tracker_site).unwrap();

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
    let control_file_location = format!(
        "https://sources.debian.org/data/main/{}/{}/{}/debian/control",
        &pkg.chars().take(1).collect::<String>(),
        &pkg,
        &version
    );
    let resp = reqwest::blocking::get(control_file_location)?;
    Ok(resp.text()?)
}

pub(crate) fn parse_control_file(content: &str) -> Result<Vec<String>, DebNixError> {
    let par = debcontrol::parse_str(content).map_err(|e| DebNixError::DebControl(e.to_string()))?;
    let mut result = vec![];

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

fn parse_control_value(value: &str) -> Result<Vec<String>, ()> {
    use regex::Regex;
    // for version numbers
    let ve = Regex::new(r"\(([^\)]+)\)").unwrap();
    let ve_curly = Regex::new(r"\$\{([^\)]+)\}").unwrap();

    let mut result = vec![];
    let values = value.split(',').collect::<Vec<&str>>();
    for value in &values {
        let value = value.trim_matches('\n');
        let value = ve.replace_all(value, "");
        let value = ve_curly.replace_all(&value, "");
        if value != "" {
            result.push(String::from(value.as_ref()));
        }
    }
    Ok(result)
}

pub(crate) fn get_debian_deps(pkgs: &str) -> Result<Vec<String>, DebNixError> {
    let pkgs = debian_redirect(pkgs)?;
    let download_control_file = download_control_file(&pkgs)?;
    let parsed_control_file = parse_control_file(&download_control_file)?;
    Ok(parsed_control_file)
}
