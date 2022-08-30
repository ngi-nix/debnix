use crate::CliArgs;
use clap::CommandFactory;
use clap_complete::Shell;

pub(crate) fn generate_completion(shell: &str) {
    let shell: Shell = match shell.to_lowercase().parse() {
        Ok(shell) => shell,
        _ => {
            eprintln!("Unsupported shell: {}", shell);
            std::process::exit(1);
        }
    };
    let mut out = std::io::stdout();
    clap_complete::generate(shell, &mut CliArgs::command(), "debnix", &mut out);
}
