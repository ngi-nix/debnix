use clap::Parser;

#[derive(Parser, Clone)]
pub(crate) struct CliArgs {
    pkg: Option<String>,
    #[clap(long, value_parser)]
    discover: Option<usize>,
    #[clap(long, value_parser)]
    discover_start: Option<usize>,
    /// Set a timeout in minutes after which the program will gracefully exit
    #[clap(long, value_parser)]
    timeout: Option<usize>,
    #[clap(long, value_parser)]
    /// The location of the generated output files.
    output: Option<String>,
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
    pub(crate) fn generate_completion(&self) -> Option<&String> {
        self.generate_completion.as_ref()
    }

    pub(crate) fn generate_map(&self) -> Option<&String> {
        self.generate_map.as_ref()
    }

    pub(crate) fn map(&self) -> Option<&String> {
        self.map.as_ref()
    }

    pub(crate) fn timeout(&self) -> Option<usize> {
        self.timeout
    }

    pub(crate) fn output(&self) -> Option<String> {
        self.output.clone()
    }
}
