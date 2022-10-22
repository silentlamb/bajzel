use bajzel_lib::lexer::lex_tokens;
use clap::{arg, Command};

fn run() -> Result<(), String> {
    let cmd = Command::new("bajzel").arg(arg!(<input> ".fuzl input file"));
    let m = cmd
        .try_get_matches()
        .map_err(|_| "Args required".to_string())?;

    let path = m.get_one::<String>("input").ok_or("wrong args")?;
    let input = std::fs::read_to_string(path)
        .map_err(|_e| "Could not read file".to_string())?;

    let tokens =
        lex_tokens(input.as_str()).map_err(|_| String::from("Lexer failed"))?;

    for token in tokens.iter() {
        println!("Token::{:?},", token);
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("[-] Error: {}", e);
    }
}
