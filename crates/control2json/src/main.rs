use std::{
    env, fs,
    io::{self, stdout, BufRead, Write},
};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let input = &args[1];

    if input == "-" {
        let mut output = String::new();
        let mut reader = io::stdin().lock();
        for line in reader.lines() {
            println!("{}", line?);
        }
    } else {
        let mut reader = fs::File::open(input).unwrap();
    }

    // let mut input = String::new();
    // match io::stdin().read_line(&mut input) {
    // for line in reader.lines() {
    //     println!("{}", line.unwrap());
    // }

    // let mut buffer = Vec::new();
    // // reader.read_line(&mut buffer)?;
    // let mut output = String::new();
    // reader.read_to_string(&mut output)?;
    // println!("{}", output);
    //
    // let mut stdout = io::stdout();
    // stdout.write_all(&buffer)?;

    Ok(())
}
