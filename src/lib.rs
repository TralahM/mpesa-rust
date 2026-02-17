#![doc = include_str!("../README.md")]
//!
//! ## Optional Features
//!
//! The following are a list of [Cargo features][cargo-features] that can be
//! enabled or disabled:
//!
//! - **account_balance** *(enabled by default)*: Enables Account Balance service support.
//! - **b2b** *(enabled by default)*: Enables B2B service support.
//! - **b2c** *(enabled by default)*: Enables B2C service support.
//! - **bill_manager** *(enabled by default)*: Enables Bill Manager services support.
//! - **c2b_register** *(enabled by default)*: Enables C2B URL Registration service support.
//! - **c2b_simulate** *(enabled by default)*: Enables C2B request simulation service support.
//! - **express** *(enabled by default)*: Enables Mpesa Express (STK push) service support.
//! - **dynamic_qr** *(enabled by default)*: Enables Dynamic QR generation service support.
//! - **transaction_reversal** *(enabled by default)*: Enables Transaction Reversal service support.
//! - **transaction_status** *(enabled by default)*: Enables Transaction Status service support.
//! - **no_openssl** *(enabled by default)*: Disables the dependency on `openssl` as the crate for handling mpesa
//!   certificates and base64 encoding, instead using the `x509-parser`, `rsa`, `base64` and `rand` crates.
//!   Automatically enabled when any of the features *account_balance*, *b2b*, *b2c*, *bill_manager*, *express*,
//!   *transaction_reversal*, or *transaction_status* are enabled.
//! - **openssl**: Enables the use of `openssl` as the dependency for handling certificates and base64 encoding instead
//!   of the default.

mod auth;
mod client;
mod constants;
pub mod environment;
mod errors;
pub mod services;
pub mod validator;

pub use client::Mpesa;
pub use constants::{CommandId, IdentifierTypes, ResponseType, SendRemindersTypes, TransactionType};
#[cfg(feature = "bill_manager")]
#[cfg_attr(docsrs, doc(cfg(feature = "bill_manager")))]
pub use constants::{Invoice, InvoiceItem};
pub use environment::ApiEnvironment;
pub use environment::Environment::{self, Production, Sandbox};
#[cfg(feature = "no_openssl")]
#[cfg_attr(docsrs, doc(cfg(feature = "no_openssl")))]
pub use errors::EncryptionErrors;
pub use errors::{BuilderError, MpesaError, MpesaResult, ResponseError};
