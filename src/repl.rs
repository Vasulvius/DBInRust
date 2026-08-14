#[derive(Debug, PartialEq)]
pub enum MetaCommand {
    Exit,
    Help,
    Unknown(String),
}

impl MetaCommand {
    fn create(line: &str) -> MetaCommand {
        match line.to_lowercase().as_str() {
            ".exit" | ".quit" => MetaCommand::Exit,
            ".help" => MetaCommand::Help,
            _ => MetaCommand::Unknown(String::from(line)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Input {
    Meta(MetaCommand),
    Sql(String),
    Empty,
}

pub fn classify(line: &str) -> Input {
    let line = line.trim();
    if line.is_empty() {
        Input::Empty
    } else if line.starts_with('.') {
        Input::Meta(MetaCommand::create(line))
    } else {
        Input::Sql(String::from(line))
    }
}
