use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Unsupported platform: {os} {arch}")]
    UnsupportedPlatform { os: String, arch: String },

    #[error("Download failed: {0}")]
    DownloadError(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Zip extraction failed: {0}")]
    ZipError(#[from] zip::result::ZipError),

    #[error("Installation failed: {message}")]
    InstallError { message: String },

    #[error("Command execution failed: {command} - {message}")]
    CommandError { command: String, message: String },

    #[error("User cancelled operation")]
    UserCancelled,

    #[error("AWS Error: {0}")]
    AwsError(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
