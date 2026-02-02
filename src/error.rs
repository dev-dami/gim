use thiserror::Error;

pub type Result<T> = std::result::Result<T, GimError>;

#[derive(Error, Debug)]
pub enum GimError {
    #[error("failed to collect {module} metrics: {source}")]
    Collector {
        module: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to load config from {path}: {source}")]
    ConfigLoad {
        path: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to parse config: {source}")]
    ConfigParse {
        #[from]
        source: serde_yaml::Error,
    },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("output formatting error: {0}")]
    Output(String),

    #[error("TUI error: {0}")]
    Tui(String),

    #[error("unknown module: {0}")]
    UnknownModule(String),

    #[error("unknown output format: {0}")]
    UnknownFormat(String),
}

impl GimError {
    pub fn exit_code(&self) -> i32 {
        match self {
            GimError::Collector { .. } => 2,
            GimError::ConfigLoad { .. } | GimError::ConfigParse { .. } => 3,
            GimError::Io(_) => 4,
            GimError::Output(_) => 5,
            GimError::Tui(_) => 6,
            GimError::UnknownModule(_) | GimError::UnknownFormat(_) => 1,
        }
    }
}
