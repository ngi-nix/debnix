use std::{
    env, fs,
    io::{self, Read, Write},
};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let input = &args[1];

    if input == "-" {
        let mut buffer = Vec::new();
        let mut reader = io::stdin().lock();
        reader.read_to_end(&mut buffer)?;
        let mut stdout = io::stdout();
        stdout.write_all(&buffer)?;
    } else {
        let mut buffer = Vec::new();
        let mut reader = fs::File::open(input).unwrap();
        reader.read_to_end(&mut buffer)?;
        let mut stdout = io::stdout();
        stdout.write_all(&buffer)?;
    }

    Ok(())
}
