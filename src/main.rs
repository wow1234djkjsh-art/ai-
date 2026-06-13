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
    /// Extra arguments passed to the script as `args` list
    #[arg(trailing_var_arg = true)]
    script_args: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    if cli.test {
        interpreter::run_tests();
        return;
    }
    if let Some(path) = cli.run {
        let src = std::fs::read_to_string(&path).expect("cannot read script");
        let mut env = interpreter::new_env();
        let argv: Vec<interpreter::Value> = cli.script_args.iter()
            .map(|s| interpreter::Value::String(s.clone()))
            .collect();
        env.define("args".into(), interpreter::Value::List(argv));
        let result = interpreter::exec_in(&mut env, &src);
        if let interpreter::Value::Error(msg) = &result {
            eprintln!("Runtime Error: {}", msg);
            std::process::exit(1);
        }
    } else {
        repl();
    }
}

fn repl() {
    use interpreter::{exec_in, new_env, Value};
    use std::io::{self, Write};

    let mut env = new_env();
    println!("C-DSL REPL  (exit to quit)");
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let first = line.trim().to_string();
                if first.is_empty() { continue; }
                if first == "exit" || first == "quit" { break; }

                // Accumulate block statements until 'end'
                let src = if needs_block(&first) {
                    let mut buf = first;
                    loop {
                        print!(".. ");
                        io::stdout().flush().unwrap();
                        let mut cont = String::new();
                        match io::stdin().read_line(&mut cont) {
                            Ok(0) | Err(_) => break,
                            Ok(_) => {
                                let trimmed = cont.trim_end_matches('\n').trim_end_matches('\r');
                                buf.push('\n');
                                buf.push_str(trimmed);
                                if trimmed.trim() == "end" { break; }
                            }
                        }
                    }
                    buf
                } else {
                    first
                };

                let result = exec_in(&mut env, &src);
                if !matches!(result, Value::Nil) {
                    println!("{}", result);
                }
            }
        }
    }
}

fn needs_block(src: &str) -> bool {
    let first_word = src.split_whitespace().next().unwrap_or("");
    matches!(first_word, "while" | "try")
}
