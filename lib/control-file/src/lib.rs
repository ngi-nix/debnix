use debcontrol::Paragraph;
use log::debug;
use thiserror::Error;

#[derive(Error, Debug)]
/// The Control File Error Type
pub enum ControlFileError {
    #[error("DebControl Error")]
    ControlParse(String),
}

#[derive(Debug)]
/// A wrapper around a Control file, and the debcontrol library.
/// Exposes convenience methods for working with control files.
pub struct ControlFile<'a> {
    paragraphs: Vec<Paragraph<'a>>,
}

impl<'a, 'b> ControlFile<'b> {
    pub fn from_str(content: &'b str) -> Result<ControlFile<'a>, ControlFileError>
    where
        'b: 'a,
    {
        Ok(Self {
            paragraphs: debcontrol::parse_str(content).map_err(|e| {
                ControlFileError::ControlParse(format!(
                    "Control File could not be parsed: {}, {}",
                    content, e
                ))
            })?,
        })
    }
    pub fn get_pkgs(&self) -> Result<Vec<String>, ControlFileError> {
        let mut result = vec![];
        debug!("Full control file: \n {:?}", self.paragraphs);
        for paragraph in &self.paragraphs {
            for field in &paragraph.fields {
                if let "Package" = field.name {
                    result.extend(Self::parse_control_value(&field.value));
                }
                debug!("This was not included as a pkg output:\n {:?}", &field);
            }
        }
        Ok(result)
    }
    pub fn get_dependencies(&self) -> Result<Vec<String>, ControlFileError> {
        let mut result = vec![];
        debug!("Full control file: \n {:?}", self.paragraphs);
        for paragraph in &self.paragraphs {
            for field in &paragraph.fields {
                match field.name {
                    "Build-Depends" | "Depends" | "Recommends" | "Suggests" => {
                        result.extend(Self::parse_control_value(&field.value));
                    }
                    _ => {}
                }
            }
        }
        Ok(result)
    }
    /// Parses control values, cleans them and returns them.
    fn parse_control_value(value: &str) -> Vec<String> {
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
        }
        result
    }
}
