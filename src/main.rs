use bajzel_lib::{
    evaluator::evaluate_program,
    lexer::{lex_tokens, Tokens},
    parser::parse_tokens,
};
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
    let tokens = Tokens::new(&tokens);
    let program = parse_tokens(tokens)?;

    // for statement in program.clone().into_iter() {
    //     println!("> {:#?}", statement);
    // }

    let env = evaluate_program(program).map_err(|x| match x {
        bajzel_lib::error::BajzelError::Conversion(x) => {
            format!("conversion error: {}", x)
        }
        bajzel_lib::error::BajzelError::ProgramNotFinished => {
            "program end not expected yet".to_owned()
        }
        bajzel_lib::error::BajzelError::Syntax(x) => {
            format!("syntax error: {}", x)
        }
        bajzel_lib::error::BajzelError::Expr(x) => {
            format!("expression error: {}", x)
        }
    })?;

    println!("Evaluated program:\n");
    for (name, group) in env.groups {
        println!(">>> Group: {}", name);
        println!("    Fields:");
        for field in group.fields {
            println!("        - {:?}: {:?}", field.alias, field.def);
        }
        println!();
    }

    let gen = env.gen.expect("Generator should be defined");
    println!(">>> Generator: {}", gen.name);
    println!("    OUT  = {} to {}", gen.out_min, gen.out_max);
    println!("    TERM = {:?}", gen.term);

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("[-] {}", e);
    }
}
