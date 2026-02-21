use std::env::VarError;
use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Mpesa error stack
#[derive(Error, Debug)]
pub enum MpesaError {
    #[error("Service error: {0}")]
    Service(ResponseError),
    #[error("An error has occurred while performing the http request")]
    NetworkError(#[from] reqwest::Error),
    #[error("A recoverable error has occurred while performing an operation. Retrying is possible.")]
    TransientError,
    #[error("An error has occurred while serializing/ deserializing")]
    ParseError(#[from] serde_json::Error),
    #[error("An error has occurred while retrieving an environmental variable")]
    EnvironmentalVariableError(#[from] VarError),
    #[cfg(feature = "openssl")]
    #[error("An error has occurred while generating security credentials")]
    EncryptionError(#[from] openssl::error::ErrorStack),
    #[cfg(feature = "no_openssl")]
    #[error("An error has occurred while generating security credentials")]
    EncryptionErrors(#[from] EncryptionErrors),
    #[error("{0}")]
    Message(&'static str),
    #[error("An error has occurred while building the request: {0}")]
    BuilderError(BuilderError),
}

/// Encryption errors when the `no_openssl` feature is enabled
#[cfg(feature = "no_openssl")]
#[derive(Error, Debug)]
pub enum EncryptionErrors {
    #[error("An error has occurred while generating security credentials")]
    RsaEncryption(#[from] rsa::errors::Error),
    #[error("An error has occurred while generating security credentials")]
    PublicKey(#[from] rsa::pkcs8::Error),
    #[error("An error has occurred while parsing or validating a certificate")]
    Pem(#[from] x509_parser::nom::Err<x509_parser::error::PEMError>),
    #[error("An error has occurred while parsing or validating a certificate")]
    X509(#[from] x509_parser::nom::Err<x509_parser::error::X509Error>),
}

/// `Result` enum type alias
pub type MpesaResult<T> = Result<T, MpesaError>;

pub(crate) type BackoffMpesaResult<T> = Result<T, backoff::Error<MpesaError>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ResponseError {
    pub request_id: String,
    pub error_code: String,
    pub error_message: String,
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "requestID: {}, errorCode:{}, errorMessage:{}",
            self.request_id, self.error_code, self.error_message
        )
    }
}

#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("Field [{0}] is required")]
    UninitializedField(&'static str),
    #[error("Field [{0}] is invalid")]
    ValidationError(String),
}

impl From<String> for BuilderError {
    fn from(s: String) -> Self {
        Self::ValidationError(s)
    }
}

impl From<derive_builder::UninitializedFieldError> for MpesaError {
    fn from(e: derive_builder::UninitializedFieldError) -> Self {
        Self::BuilderError(BuilderError::UninitializedField(e.field_name()))
    }
}

impl From<url::ParseError> for MpesaError {
    fn from(e: url::ParseError) -> Self {
        Self::BuilderError(BuilderError::ValidationError(e.to_string()))
    }
}

impl MpesaError {
    pub fn to_retryable<E: Into<MpesaError>>(val: E) -> backoff::Error<Self> {
        let val = val.into();
        match &val {
            MpesaError::TransientError => backoff::Error::transient(val),
            MpesaError::Service(res) => {
                match res.error_code.as_str() {
                    // system busy|quota violation or spike arrest violation
                    "500.003.02" | "500.003.03" => backoff::Error::retry_after(val, std::time::Duration::from_secs(1)),
                    // transaction already in progress
                    "500.001.1001" if res.error_message.contains("Unable to lock subscriber") => {
                        backoff::Error::retry_after(val, std::time::Duration::from_secs(1))
                    }
                    _ => backoff::Error::permanent(val),
                }
            }
            _ => backoff::Error::permanent(val),
        }
    }
}

impl From<backoff::Error<MpesaError>> for MpesaError {
    fn from(e: backoff::Error<MpesaError>) -> Self {
        match e {
            backoff::Error::Permanent(err) => err,
            backoff::Error::Transient { err, .. } => err,
        }
    }
}

impl From<backoff::Error<reqwest::Error>> for MpesaError {
    fn from(e: backoff::Error<reqwest::Error>) -> Self {
        match e {
            backoff::Error::Transient { err, .. } => {
                if err.is_connect() || err.is_timeout() {
                    return MpesaError::TransientError;
                }
                MpesaError::from(err)
            }
            backoff::Error::Permanent(err) => {
                if err.is_connect() || err.is_timeout() {
                    return MpesaError::TransientError;
                }
                MpesaError::from(err)
            }
        }
    }
}

impl TryFrom<MpesaError> for backoff::Error<reqwest::Error> {
    type Error = MpesaError;
    fn try_from(e: MpesaError) -> Result<Self, Self::Error> {
        match e {
            MpesaError::NetworkError(err) if err.is_timeout() || err.is_connect() => Ok(backoff::Error::transient(err)),
            MpesaError::NetworkError(err) => Ok(backoff::Error::permanent(err)),
            _ => Err(e),
        }
    }
}
