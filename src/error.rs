#[cfg(feature="blowfish")]
use blowfish::cipher::InvalidLength;
use thiserror::Error;
#[cfg(feature="steam")]
use steamworks::{SteamError, SteamAPIInitError};

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {

    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Parse error: {0}")]
    ParseError(#[from] url::ParseError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    /// this error is raised when using the global login but you needed to use the steam login
    #[error("restartup, but not Steam")]
    Restartup,

    #[error("Missing Login Form")]
    MissingLoginForm,
    #[error("LoginFailure")]
    LoginFailure,
    #[error("LoginFailure: {0}")]
    LoginFailureMessage(String),

    #[error("Missing Username")]
    MissingUsername,
    #[error("Missing Password")]
    MissingPassword,

    #[cfg(feature="steam")]
    #[error(transparent)]
    Steam(#[from] SteamError),
    #[cfg(feature="steam")]
    #[error(transparent)]
    SteamApiInit(#[from] SteamAPIInitError),

    #[error(transparent)]
    #[cfg(feature="blowfish")]
    InvalidLength(#[from]InvalidLength)
}