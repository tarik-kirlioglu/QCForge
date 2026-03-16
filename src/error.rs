use thiserror::Error;

#[derive(Error, Debug)]
pub enum QcForgeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse {tool} output from {path}: {detail}")]
    ParseError {
        tool: String,
        path: String,
        detail: String,
    },

    #[error("ZIP archive error: {0}")]
    ZipError(#[from] zip::result::ZipError),

    #[error("No fastqc_data.txt found in archive: {0}")]
    FastqcDataNotFound(String),

    #[error("No QC files found in directory: {0}")]
    NoFilesFound(String),

    #[error("Invalid numeric value in {field}: {value}")]
    NumericParse { field: String, value: String },

    #[error("Terminal error: {0}")]
    Terminal(String),
}

pub type Result<T> = std::result::Result<T, QcForgeError>;
