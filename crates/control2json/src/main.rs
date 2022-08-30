use std::{
    collections::HashMap,
    env, fs,
    io::{self, Read, Write},
};

use control_file::ControlFile;

use self::error::Control2JsonError;

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
    let args: Vec<String> = env::args().collect();
    let input = &args[1];
    let match_from = &args[2];

    let mut reader: Box<dyn io::Read> = if input == "-" {
        Box::new(io::stdin().lock())
    } else {
        Box::new(fs::File::open(input)?)
    };

    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    let pkgs = pkgs_from_control_file(std::str::from_utf8(&buffer)?)?;
    let mut stdout = io::stdout();
    let fmt = format!("{:?}", pkgs);
    stdout.write_all(fmt.as_bytes())?;

    if !match_from.is_empty() {
        println!("Matching");
        let map = get_map(match_from)?;
        let result = match_from_map(pkgs, map)?;
        let fmt = format!("{:?}", result);
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
    Ok(result)
}
