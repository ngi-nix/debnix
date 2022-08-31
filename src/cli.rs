use clap::Parser;

#[derive(Parser)]
pub(crate) struct CliArgs {
    pkg: Option<String>,
    #[clap(long, value_parser)]
    discover: Option<usize>,
    #[clap(long, value_parser)]
    discover_start: Option<usize>,
    #[clap(long, value_parser)]
    write: Option<String>,
    #[clap(long, value_parser)]
    read_popcon: Option<String>,
    /// Generates completion for the specified shell.
    #[clap(long, value_name = "SHELL", value_parser)]
    generate_completion: Option<String>,
    #[clap(long, value_parser)]
    /// The map that is generated out of multiple input files.
    generate_map: Option<String>,
    /// The input map, that can be used for Lookup.
    #[clap(long, value_parser)]
    map: Option<String>,
}

impl CliArgs {
    pub(crate) fn pkg(&self) -> Option<&String> {
        self.pkg.as_ref()
    }
    pub(crate) fn discover(&self) -> Option<usize> {
        self.discover
    }
    pub(crate) fn discover_start(&self) -> Option<usize> {
        self.discover_start
    }
    pub(crate) fn write(&self) -> Option<&String> {
        self.write.as_ref()
    }
    pub(crate) fn read_popcon(&self) -> Option<&String> {
        self.read_popcon.as_ref()
    }
    pub(crate) fn generate_completion(&self) -> Option<&String> {
        self.generate_completion.as_ref()
    }

    pub(crate) fn generate_map(&self) -> Option<&String> {
        self.generate_map.as_ref()
    }
}
