use std::io;

#[derive(Debug)]
pub enum UserInput {
    Help,
    Message(String),
    Quit,
}

#[derive(Debug)]
pub enum UserInputError {
    IoError,
}

impl From<io::Error> for UserInputError {
    fn from(_: io::Error) -> Self {
        UserInputError::IoError
    }
}

impl From<&str> for UserInput {
    fn from(value: &str) -> Self {
        let trimmed = value.trim();
        match trimmed.split_whitespace().next().unwrap_or("") {
            "/quit" => UserInput::Quit,
            "/help" => UserInput::Help,
            _ => UserInput::Message(trimmed.to_string()),
        }
    }
}

pub async fn get_user_input<R>(reader: &mut R) -> Result<UserInput, UserInputError>
where
    R: tokio::io::AsyncBufReadExt + Unpin,
{
    let mut input_line = String::new();

    match reader.read_line(&mut input_line).await {
        Ok(0) => Ok(UserInput::Quit),
        Ok(_) => Ok(UserInput::from(input_line.as_str())),
        Err(e) => Err(e.into()),
    }
}
