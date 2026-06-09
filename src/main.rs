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
        interpreter::execute(&src);
    } else {
        eprintln!("No script supplied. Use --run <file>");
    }
}
