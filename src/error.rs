use std::fmt;

/// Result type for argument parsing
pub type Result<T> = std::result::Result<T, Error>;

/// An error that occurred during argument parsing
#[derive(Debug)]
pub enum Error {
    /// A required argument was not provided
    MissingRequired { name: String },

    /// An unknown flag was provided
    UnknownFlag { flag: String },

    /// An argument expected a value but none was provided
    MissingValue { name: String },

    /// Failed to parse a value
    InvalidValue {
        name: String,
        value: String,
        expected: &'static str,
    },

    /// Duplicate value for a non-array argument
    DuplicateValue { name: String },

    /// A positional argument was missing
    MissingPositional { name: String, position: usize },

    /// Too many positional arguments
    TooManyPositional { max: usize, got: usize },

    /// Required config file is missing
    MissingConfig { path: String },

    /// Help was requested
    Help(String),

    /// Version was requested
    Version(String),

    /// TOML parsing error
    Toml(stoml::Error),

    /// IO error
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingRequired { name } => {
                write!(f, "required argument '{}' was not provided", name)
            }
            Error::UnknownFlag { flag } => {
                write!(f, "unknown flag '{}'", flag)
            }
            Error::MissingValue { name } => {
                write!(f, "argument '{}' requires a value", name)
            }
            Error::InvalidValue {
                name,
                value,
                expected,
            } => {
                write!(
                    f,
                    "invalid value '{}' for '{}': expected {}",
                    value, name, expected
                )
            }
            Error::DuplicateValue { name } => {
                write!(f, "argument '{}' cannot be specified multiple times", name)
            }
            Error::MissingPositional { name, position } => {
                write!(
                    f,
                    "missing required positional argument '{}' at position {}",
                    name, position
                )
            }
            Error::TooManyPositional { max, got } => {
                write!(
                    f,
                    "too many positional arguments: expected at most {}, got {}",
                    max, got
                )
            }
            Error::MissingConfig { path } => {
                write!(f, "required config file '{}' not found", path)
            }
            Error::Help(msg) => write!(f, "{}", msg),
            Error::Version(msg) => write!(f, "{}", msg),
            Error::Toml(e) => write!(f, "TOML error: {}", e),
            Error::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Toml(e) => Some(e),
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<stoml::Error> for Error {
    fn from(e: stoml::Error) -> Self {
        Error::Toml(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl Error {
    /// Returns true if this is a help request
    pub fn is_help(&self) -> bool {
        matches!(self, Error::Help(_))
    }

    /// Returns true if this is a version request
    pub fn is_version(&self) -> bool {
        matches!(self, Error::Version(_))
    }

    /// Returns true if this is a help or version request
    pub fn is_info_request(&self) -> bool {
        self.is_help() || self.is_version()
    }

    /// Exit the program with the appropriate status code
    ///
    /// Prints help/version to stdout with exit code 0,
    /// prints errors to stderr with exit code 1.
    pub fn exit(&self) -> ! {
        if self.is_info_request() {
            println!("{}", self);
            std::process::exit(0);
        } else {
            eprintln!("error: {}", self);
            std::process::exit(1);
        }
    }
}
