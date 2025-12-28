use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("WalkDir error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("Repository not found at {0}")]
    RepositoryNotFound(PathBuf),

    #[error("Repository already exists at {0}")]
    RepositoryExists(PathBuf),

    #[error("Object not found: {0}")]
    ObjectNotFound(String),

    #[error("Invalid object type: {0}")]
    InvalidObjectType(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid hash: {0}")]
    InvalidHash(String),

    #[error("Invalid reference: {0}")]
    InvalidReference(String),

    #[error("Branch not found: {0}")]
    BranchNotFound(String),

    #[error("Branch already exists: {0}")]
    BranchExists(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Index error: {0}")]
    IndexError(String),

    #[error("Commit error: {0}")]
    CommitError(String),

    #[error("Remote not found: {0}")]
    RemoteNotFound(String),

    #[error("Remote already exists: {0}")]
    RemoteExists(String),

    #[error("Remote error: {0}")]
    RemoteError(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

