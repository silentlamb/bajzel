use bajzel_lib::{
    evaluator::evaluate_program,
    generator::Gen,
    lexer::{lex_tokens, Tokens},
    parser::parse_tokens,
};
use clap::{arg, Command};
use std::io::Write;

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

    // println!("Input: ");
    // println!("{}\n", input);

    // println!("Tokens: ");
    // for token in tokens.iter() {
    //     println!("- {:?}", token);
    // }

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
        bajzel_lib::error::BajzelError::NotConstructedProperly => {
            panic!("program was not constructed properly!");
        }
    })?;

    // println!("Evaluated program:\n");
    // for (name, group) in &env.groups {
    //     println!(">>> Group: {}", name);
    //     println!("    Fields:");
    //     for field in &group.fields {
    //         println!("        - {:?}: {:?}", field.alias, field.def);
    //     }
    //     println!();
    // }

    // {
    //     let gen = env.gen.as_ref().expect("Generator should be defined");
    //     println!(">>> Generator: {}", gen.name);
    //     println!("    OUT  = {} to {}", gen.out_min, gen.out_max);
    //     println!("    TERM = {:?}", gen.term);
    // }

    let gen = Gen::default();
    let output = gen
        .generate(&env)
        .map_err(|_| "Generate error".to_owned())?;
    let _ = std::io::stdout().write_all(&output);
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("[-] {}", e);
    }
}
