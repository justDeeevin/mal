mod tokens;

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use tokens::Tokens;

fn read(input: String) -> Tokens {
    Tokens::read_str(&input)
}

fn eval(input: Tokens) -> Tokens {
    input
}

fn print(mut input: Tokens) -> Option<String> {
    input.pr_str()
}

fn rep(input: String) -> Option<String> {
    print(eval(read(input)))
}

fn main() -> anyhow::Result<()> {
    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new()?;
    loop {
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                println!("{}", rep(line).unwrap_or("unbalanced".to_string()));
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                return Err(err.into());
            }
        }
    }

    Ok(())
}
