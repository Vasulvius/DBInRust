use std::{
    io::{self, Write},
    println,
};

use minidb::{
    parser::parse,
    repl::{Input, MetaCommand, classify},
};

const HELP: &str = "META COMMANDS
.exit, .quit        quit the minidb cli
.help               display help menu";

fn main() {
    loop {
        let mut input = String::new();

        print!("minidb> ");
        match io::stdout().flush() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("An unexpected error has occurred: {e}");
                break;
            }
        }
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => match classify(&input) {
                Input::Empty => (),
                Input::Sql(query) => match parse(&query) {
                    Ok(sql) => println!("{:#?}", sql),
                    Err(e) => eprintln!("{:#?}", e),
                },
                Input::Meta(MetaCommand::Exit) => break,
                Input::Meta(MetaCommand::Help) => println!("{HELP}"),
                Input::Meta(MetaCommand::Unknown(unknown_input)) => eprintln!(
                    "Error: unknown command or invalid arguments: `{unknown_input}`. Enter `.help` for help"
                ),
            },
            Err(e) => {
                eprintln!("Failed to read line: {e}");
                break;
            }
        }
    }
}
