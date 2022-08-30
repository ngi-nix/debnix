use std::{
    collections::HashMap,
    fs,
    io::{self, Read, Write},
};

use clap::Parser;
use control_file::ControlFile;

use self::error::Control2JsonError;

pub mod cli {
    use clap::Parser;

    #[derive(Parser)]
    pub(crate) struct CliArgs {
        /// The input file, if supplied `-`,
        /// then it will be read from stdin.
        input: String,
        #[clap(long, value_parser)]
        /// The path to a json map.
        map: Option<String>,
    }

    impl CliArgs {
        pub(crate) fn map(&self) -> Option<&String> {
            self.map.as_ref()
        }

        pub(crate) fn input(&self) -> &str {
            self.input.as_ref()
        }
    }
}

mod error {
    use thiserror::Error;
    /// The debnix error type
    #[derive(Error, Debug)]
    pub enum Control2JsonError {
        /// Io Error
        #[error("IoError: {0}")]
        Io(#[from] std::io::Error),
        #[error("Utf8 Conversion Error")]
        Utf8(#[from] std::str::Utf8Error),
        #[error("Control File Error {0}")]
        ControlFile(#[from] control_file::ControlFileError),
    }
}

fn main() -> Result<(), Control2JsonError> {
    let opts = cli::CliArgs::parse();

    let mut reader: Box<dyn io::Read> = if let "-" = opts.input() {
        Box::new(io::stdin().lock())
    } else {
        Box::new(fs::File::open(opts.input())?)
    };

    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    let pkgs = pkgs_from_control_file(std::str::from_utf8(&buffer)?)?;
    let mut stdout = io::stdout();

    if let Some(location) = opts.map() {
        let map = get_map(location)?;
        let result = match_from_map(pkgs, map)?;
        let fmt = format!("{:?}", result);
        stdout.write_all(fmt.as_bytes())?;
    } else {
        let fmt = format!("{:?}", pkgs);
        stdout.write_all(fmt.as_bytes())?;
    }

    Ok(())
}

fn pkgs_from_control_file(control_file: &str) -> Result<Vec<String>, Control2JsonError> {
    let control_file = ControlFile::from_str(control_file)?;
    Ok(control_file.get_dependencies()?)
}

fn get_map(map: &str) -> Result<HashMap<String, String>, Control2JsonError> {
    let mut file = fs::File::open(&map)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    let map: HashMap<String, String> = serde_json::from_str(&buffer).unwrap();
    Ok(map)
}

fn match_from_map(
    control: Vec<String>,
    map: HashMap<String, String>,
) -> Result<Vec<String>, Control2JsonError> {
    let mut result = vec![];

    for pkg in control {
        if let Some(matched) = map.get(&pkg) {
            result.push(matched.into());
        }
    }
    result.sort();
    result.dedup();
    Ok(result)
}
