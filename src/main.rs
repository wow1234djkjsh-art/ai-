use clap::Parser;
mod builtins;
mod cache;
mod interpreter;
mod lexer;
mod parser;

#[derive(Parser)]
#[command(name = "c-dsl")]
struct Cli {
    #[arg(short, long)]
    run: Option<String>,
    #[arg(short, long)]
    test: bool,
}

fn main() {
    let cli = Cli::parse();
    if cli.test {
        interpreter::run_tests();
        return;
    }
    if let Some(path) = cli.run {
        let src = std::fs::read_to_string(&path).expect("cannot read script");
        let result = interpreter::execute(&src);
        if let interpreter::Value::Error(msg) = &result {
            eprintln!("Runtime Error: {}", msg);
            std::process::exit(1);
        }
    } else {
        repl();
    }
}

fn repl() {
    use interpreter::{exec_in, Environment, Value};
    use std::io::{self, Write};

    let mut env = Environment::new();
    println!("C-DSL REPL  (exit to quit)");
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let src = line.trim();
                if src.is_empty() {
                    continue;
                }
                if src == "exit" || src == "quit" {
                    break;
                }
                let result = exec_in(&mut env, src);
                if !matches!(result, Value::Nil) {
                    println!("{}", result);
                }
            }
        }
    }
}
