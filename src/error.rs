use crossterm::style::{Colorize, Styler};
use std::{error, fmt, io};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    internal: io::Error,
    help: Option<&'static str>,
}

impl Error {
    pub fn new<E>(kind: io::ErrorKind, error: E) -> Self
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Error {
            internal: io::Error::new(kind, error),
            help: None,
        }
    }

    pub fn with_help(mut self, help: &'static str) -> Self {
        self.help = Some(help);

        self
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self {
            internal: error,
            help: None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", "error".dark_red().bold(), self.internal)?;

        if let Some(help) = self.help {
            write!(f, "\n {}: {}", "help".dark_yellow().bold(), help,)?;
        }

        Ok(())
    }
}

impl error::Error for Error {}
