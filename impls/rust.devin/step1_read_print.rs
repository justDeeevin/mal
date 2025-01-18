mod reader;

use anyhow::anyhow;
use reader::Reader;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

fn read(input: String) -> String {
    input
}

fn eval(input: String) -> String {
    input
}

fn print(input: String) -> String {
    input
}

fn rep(input: String) -> String {
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
                if line.starts_with("tokenize") {
                    let string = &line[(line.find(' ').ok_or(anyhow!("no input"))?)..];
                    println!("{:?}", Reader::read_str(string).collect::<Vec<_>>());
                }
                println!("{}", rep(line));
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
