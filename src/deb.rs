use std::fs;

use control_file::ControlFile;
use serde::{Deserialize, Serialize};

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

/// Get's the location of a packages latest version of debians api
/// relies on a redirect from `sources.debian`.
pub(crate) fn get_latest_version_api(pkg: &str) -> Result<String, DebNixError> {
    let debian_sources = format!("https://sources.debian.org/api/src/{}/latest/", pkg);
    let resp = reqwest::blocking::get(debian_sources)?;
    let version_path = resp.url().path();
    Ok(version_path.to_string())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Wrapper of a subset of debians tracker api:
/// <https://sources.debian.org/doc/>
/// This exposes functionality for querying and downloading of
/// the debian control file of a package.
pub(crate) struct ControlFileApi {
    // The pkg that is being queried.
    package: Option<String>,
    // The sha256 checksum of the control file.
    checksum: Option<String>,
    // The type of the control file.
    file: Option<String>,
    // The location of the control file.
    raw_url: Option<String>,
}

impl ControlFileApi {
    fn new(pkg: &str) -> Result<Self, DebNixError> {
        let version = get_latest_version_api(pkg).unwrap();
        let control_file_api_location =
            format!("https://sources.debian.org{}debian/control", &version);

        match reqwest::blocking::get(control_file_api_location) {
            Ok(resp) => Ok(serde_json::from_str::<ControlFileApi>(&resp.text()?)?),
            Err(e) => {
                error!("\nThis location doesn't work \n{}", e);
                Err(DebNixError::Reqwest(e))
            }
        }
    }

    fn raw_url(&self) -> Option<&String> {
        self.raw_url.as_ref()
    }

    /// Get's the actual url of the control file
    fn url(&self) -> Option<String> {
        self.raw_url().map_or_else(
            || None,
            |url| Some(format!("{}{}", "https://sources.debian.org", url)),
        )
    }
    /// Downloads the control file directly from `sources.debian`.
    pub(crate) fn download_control_file(&self) -> Result<String, DebNixError> {
        if let Some(control_file_url) = self.url() {
            match reqwest::blocking::get(control_file_url) {
                Ok(resp) => {
                    return Ok(resp.text()?);
                }
                Err(e) => {
                    error!("\nThis location doesn't work \n{}", e);
                    return Err(DebNixError::Reqwest(e));
                }
            }
        }
        Err(DebNixError::DebControl(format!(
            "No raw URL for package: {:?}",
            self.package()
        )))
    }

    fn package(&self) -> Option<&String> {
        self.package.as_ref()
    }

    pub(crate) fn checksum(&self) -> Option<&String> {
        self.checksum.as_ref()
    }

    /// The debian api let's us redirect from tail packages
    /// to the real package definitions.
    pub(crate) fn from_redirect(pkgs: &str) -> Result<ControlFileApi, DebNixError> {
        let pkgs = debian_redirect(pkgs)?;
        ControlFileApi::new(&pkgs)
    }

    pub(crate) fn get_debian_deps(&self) -> Result<Vec<String>, DebNixError> {
        let download_control_file = &self.download_control_file()?;
        let parsed_control_file =
            ControlFile::from_str(download_control_file)?.get_dependencies()?;
        debug!("Parsed Control File: {:?}", &parsed_control_file);
        Ok(parsed_control_file)
    }
}

pub(crate) fn get_debian_pkg_outputs(pkgs: &str) -> Result<Vec<String>, DebNixError> {
    let pkgs = debian_redirect(pkgs)?;
    let download_control_file = ControlFileApi::new(&pkgs)?.download_control_file()?;
    let parsed_control_file = ControlFile::from_str(&download_control_file)?.get_pkgs()?;
    debug!("Parsed Control File: {:?}", &parsed_control_file);
    Ok(parsed_control_file)
}

/// Reads the packages from a popcon (popularity contest) file
/// and then collects them inside of a Vec.
pub(crate) fn read_popcon(location: &str) -> Result<Vec<String>, DebNixError> {
    let mut popcon = vec![];
    let contents = fs::read_to_string(location)?;
    let mut rdr = csv::Reader::from_reader(contents.as_bytes());
    for result in rdr.records().flatten() {
        // TODO: convert this into some error
        if let Some(untrimmed_record) = result.get(0) {
            let record = untrimmed_record.trim_end();
            if !record.starts_with('#') && !record.ends_with("(Not in sid)") {
                let name = record
                    .split(' ')
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
