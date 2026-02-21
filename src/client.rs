use std::sync::{Arc, RwLock};
use std::time::Duration;

use backoff::ExponentialBackoff;
use reqwest::Client as HttpClient;
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::environment::ApiEnvironment;
use crate::errors::BackoffMpesaResult;
#[cfg(feature = "account_balance")]
use crate::services::AccountBalanceBuilder;
#[cfg(feature = "b2b")]
use crate::services::B2bBuilder;
#[cfg(feature = "b2c")]
use crate::services::B2cBuilder;
#[cfg(feature = "c2b_register")]
use crate::services::C2bRegisterBuilder;
#[cfg(feature = "c2b_simulate")]
use crate::services::C2bSimulateBuilder;
#[cfg(feature = "transaction_status")]
use crate::services::TransactionStatusBuilder;
#[cfg(feature = "bill_manager")]
use crate::services::{
    BulkInvoiceBuilder, CancelInvoiceBuilder, OnboardBuilder, OnboardModifyBuilder, ReconciliationBuilder,
    SingleInvoiceBuilder,
};
#[cfg(feature = "dynamic_qr")]
use crate::services::{DynamicQR, DynamicQRBuilder};
#[cfg(feature = "express")]
use crate::services::{MpesaExpress, MpesaExpressBuilder, MpesaExpressQuery, MpesaExpressQueryBuilder};
#[cfg(feature = "transaction_reversal")]
use crate::services::{TransactionReversal, TransactionReversalBuilder};
use crate::{MpesaError, MpesaResult, ResponseError, auth};

/// Source: [test credentials](https://developer.safaricom.co.ke/test_credentials)
const DEFAULT_INITIATOR_PASSWORD: &str = "Safaricom999!*!";
/// Get current package version from metadata
const CARGO_PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Pure Rust implementation of `openssl::base64::encode_block`
/// to avoid the need for OpenSSL in environments where it is not available e.g, `musl` targets
#[allow(dead_code)]
#[cfg(feature = "no_openssl")]
pub(crate) fn encode_block(src: &[u8]) -> String {
    use base64::prelude::*;
    // OpenSSL's encode_block uses the standard alphabet with padding
    // base64::engine::general_purpose::STANDARD handles standard padding
    // and encoding, consistent with EVP_EncodeBlock
    BASE64_STANDARD.encode(src)
}

/// Mpesa client that will facilitate communication with the Safaricom API
#[derive(Clone, Debug)]
pub struct Mpesa {
    consumer_key: String,
    consumer_secret: SecretString,
    initiator_password: Arc<RwLock<Option<SecretString>>>,
    pub(crate) base_url: String,
    certificate: String,
    auth_token: Arc<RwLock<SecretString>>,
    auth_expiry: Arc<RwLock<i64>>,
    pub(crate) http_client: HttpClient,
}

impl Mpesa {
    /// Constructs a new `Mpesa` client.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mpesa::{Environment, Mpesa};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     dotenvy::dotenv().ok();
    ///
    ///     let client = Mpesa::new(
    ///         dotenvy::var("CONSUMER_KEY").unwrap(),
    ///         dotenvy::var("CONSUMER_SECRET").unwrap(),
    ///         Environment::Sandbox,
    ///     );
    ///
    ///     assert!(client.is_connected().await);
    /// }
    /// ```
    /// # Panics
    /// This method can panic if a TLS backend cannot be initialized for the internal http_client
    pub fn new<S: Into<String>>(consumer_key: S, consumer_secret: S, environment: impl ApiEnvironment) -> Self {
        let http_client = HttpClient::builder()
            .connect_timeout(Duration::from_secs(10))
            .user_agent(format!("httpie/{CARGO_PACKAGE_VERSION}"))
            .build()
            .expect("Error building http client");

        let base_url = environment.base_url().to_owned();
        let certificate = environment.get_certificate().to_owned();

        Self {
            consumer_key: consumer_key.into(),
            consumer_secret: consumer_secret.into().into(),
            initiator_password: Arc::new(RwLock::new(None)),
            base_url,
            certificate,
            http_client,
            auth_token: Arc::new(RwLock::new("".into())),
            auth_expiry: Arc::new(RwLock::new(0)),
        }
    }

    /// Gets the initiator password
    /// If `None`, the default password is `"Safcom496!"`
    pub(crate) fn initiator_password(&self) -> String {
        self.initiator_password
            .read()
            .unwrap()
            .as_ref()
            .map(|password| password.expose_secret().into())
            .unwrap_or(DEFAULT_INITIATOR_PASSWORD.to_owned())
    }

    /// Get the consumer key
    pub(crate) fn consumer_key(&self) -> &str {
        &self.consumer_key
    }

    /// Get the consumer secret
    pub(crate) fn consumer_secret(&self) -> &str {
        self.consumer_secret.expose_secret()
    }

    /// Optional in development but required for production for the following apis:
    /// - `account_balance`
    /// - `b2b`
    /// - `b2c`
    /// - `transaction_reversal`
    /// - `transaction_status`
    ///
    /// You will need to call this method and set your production initiator password.
    /// If in development, a default initiator password from the test credentials is already pre-set
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mpesa::{Environment, Mpesa};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     dotenvy::dotenv().ok();
    ///
    ///     let client = Mpesa::new(
    ///         dotenvy::var("CONSUMER_KEY").unwrap(),
    ///         dotenvy::var("CONSUMER_SECRET").unwrap(),
    ///         Environment::Sandbox,
    ///     );
    ///     client.set_initiator_password("your_initiator_password");
    ///     assert!(client.is_connected().await);
    /// }
    /// ```
    pub fn set_initiator_password<S: Into<String>>(&self, initiator_password: S) {
        *self.initiator_password.write().unwrap() = Some(initiator_password.into().into());
    }

    /// set auth token
    pub(crate) fn set_auth_token<S: Into<String>>(&self, token: S, expiry: i64) {
        *self.auth_token.write().unwrap() = token.into().into();
        *self.auth_expiry.write().unwrap() = expiry;
    }

    /// get auth token
    pub(crate) fn auth_token(&self) -> String {
        self.auth_token.read().unwrap().expose_secret().into()
    }

    /// get auth expiry
    pub(crate) fn auth_expiry(&self) -> i64 {
        *self.auth_expiry.read().unwrap()
    }

    /// Check if we have a cached valid auth token
    pub fn has_cached_auth(&self) -> bool {
        chrono::Utc::now().timestamp() < self.auth_expiry() && !self.auth_token().is_empty()
    }

    /// Checks if the client can be authenticated
    pub async fn is_connected(&self) -> bool {
        self.auth().await.is_ok()
    }

    /// This API generates the tokens for authenticating your API calls. This is the first API you will engage with
    /// within the set of APIs available because all the other APIs require authentication information from this API to
    /// work.
    ///
    /// Safaricom API docs [reference](https://developer.safaricom.co.ke/APIs/Authorization)
    ///
    /// Returns auth token as a `String`.
    ///
    /// # Errors
    /// Returns a `MpesaError` on failure
    pub(crate) async fn auth(&self) -> MpesaResult<String> {
        if self.has_cached_auth() {
            return Ok(self.auth_token());
        }
        let res = backoff::future::retry(ExponentialBackoff::default(), || async { auth::auth(self).await }).await?;
        Ok(res)
    }

    #[cfg(feature = "b2c")]
    #[doc = include_str!("../docs/client/b2c.md")]
    #[cfg_attr(docsrs, doc(cfg(feature = "b2c")))]
    pub fn b2c<'a>(&'a self, initiator_name: &'a str) -> B2cBuilder<'a> {
        B2cBuilder::new(self, initiator_name)
    }

    #[cfg(feature = "b2b")]
    #[doc = include_str!("../docs/client/b2b.md")]
    #[cfg_attr(docsrs, doc(cfg(feature = "b2b")))]
    pub fn b2b<'a>(&'a self, initiator_name: &'a str) -> B2bBuilder<'a> {
        B2bBuilder::new(self, initiator_name)
    }

    #[cfg(feature = "bill_manager")]
    #[doc = include_str!("../docs/client/bill_manager/onboard.md")]
    #[cfg_attr(docsrs, doc(cfg(feature = "bill_manager")))]
    pub fn onboard(&self) -> OnboardBuilder<'_> {
        OnboardBuilder::new(self)
    }

    #[cfg(feature = "bill_manager")]
    #[doc = include_str!("../docs/client/bill_manager/onboard_modify.md")]
    #[cfg_attr(docsrs, doc(cfg(feature = "bill_manager")))]
    pub fn onboard_modify(&self) -> OnboardModifyBuilder<'_> {
        OnboardModifyBuilder::new(self)
    }

    #[cfg(feature = "bill_manager")]
    #[doc = include_str!("../docs/client/bill_manager/bulk_invoice.md")]
    #[cfg_attr(docsrs, doc(cfg(feature = "bill_manager")))]
    pub fn bulk_invoice(&self) -> BulkInvoiceBuilder<'_> {
        BulkInvoiceBuilder::new(self)
    }

    #[cfg(feature = "bill_manager")]
    #[doc = include_str!("../docs/client/bill_manager/single_invoice.md")]
    #[cfg_attr(docsrs, doc(cfg(feature = "bill_manager")))]
    pub fn single_invoice(&self) -> SingleInvoiceBuilder<'_> {
        SingleInvoiceBuilder::new(self)
    }

    #[cfg(feature = "bill_manager")]
    #[doc = include_str!("../docs/client/bill_manager/reconciliation.md")]
    #[cfg_attr(docsrs, doc(cfg(feature = "bill_manager")))]
    pub fn reconciliation(&self) -> ReconciliationBuilder<'_> {
        ReconciliationBuilder::new(self)
    }

    #[cfg(feature = "bill_manager")]
    #[doc = include_str!("../docs/client/bill_manager/cancel_invoice.md")]
    #[cfg_attr(docsrs, doc(cfg(feature = "bill_manager")))]
    pub fn cancel_invoice(&self) -> CancelInvoiceBuilder<'_> {
        CancelInvoiceBuilder::new(self)
    }

    #[cfg(feature = "c2b_register")]
    #[doc = include_str!("../docs/client/c2b_register.md")]
    #[cfg_attr(docsrs, doc(cfg(feature = "c2b_register")))]
    pub fn c2b_register(&self) -> C2bRegisterBuilder<'_> {
        C2bRegisterBuilder::new(self)
    }

    #[cfg(feature = "c2b_simulate")]
    #[doc = include_str!("../docs/client/c2b_simulate.md")]
    #[cfg_attr(docsrs, doc(cfg(feature = "c2b_simulate")))]
    pub fn c2b_simulate(&self) -> C2bSimulateBuilder<'_> {
        C2bSimulateBuilder::new(self)
    }

    #[cfg(feature = "account_balance")]
    #[doc = include_str!("../docs/client/account_balance.md")]
    #[cfg_attr(docsrs, doc(cfg(feature = "account_balance")))]
    pub fn account_balance<'a>(&'a self, initiator_name: &'a str) -> AccountBalanceBuilder<'a> {
        AccountBalanceBuilder::new(self, initiator_name)
    }

    #[cfg(feature = "express")]
    #[doc = include_str!("../docs/client/express.md")]
    #[cfg_attr(docsrs, doc(cfg(feature = "express")))]
    pub fn express_request(&self) -> MpesaExpressBuilder<'_> {
        MpesaExpress::builder(self)
    }

    #[cfg(feature = "express")]
    #[doc = include_str!("../docs/client/express.md")]
    #[cfg_attr(docsrs, doc(cfg(feature = "express")))]
    pub fn express_query(&self) -> MpesaExpressQueryBuilder<'_> {
        MpesaExpressQuery::builder(self)
    }

    #[cfg(feature = "transaction_reversal")]
    #[doc = include_str!("../docs/client/transaction_reversal.md")]
    #[cfg_attr(docsrs, doc(cfg(feature = "transaction_reversal")))]
    pub fn transaction_reversal(&self) -> TransactionReversalBuilder<'_> {
        TransactionReversal::builder(self)
    }

    #[cfg(feature = "transaction_status")]
    #[doc = include_str!("../docs/client/transaction_status.md")]
    #[cfg_attr(docsrs, doc(cfg(feature = "transaction_status")))]
    pub fn transaction_status<'a>(&'a self, initiator_name: &'a str) -> TransactionStatusBuilder<'a> {
        TransactionStatusBuilder::new(self, initiator_name)
    }

    #[cfg(feature = "dynamic_qr")]
    #[doc = include_str!("../docs/client/dynamic_qr.md")]
    #[cfg_attr(docsrs, doc(cfg(feature = "dynamic_qr")))]
    pub fn dynamic_qr(&self) -> DynamicQRBuilder<'_> {
        DynamicQR::builder(self)
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "openssl")] {
            /// Generates security credentials
            /// M-Pesa Core authenticates a transaction by decrypting the security credentials.
            /// Security credentials are generated by encrypting the base64 encoded initiator password with M-Pesa’s public key,
            /// a X509 certificate. Returns base64 encoded string.
            ///
            /// # Errors
            /// Returns `EncryptionError` variant of `MpesaError`
            pub(crate) fn gen_security_credentials(&self) -> MpesaResult<String> {
                use openssl::base64;
                use openssl::rsa::Padding;
                use openssl::x509::X509;

                let pem = self.certificate.as_bytes();
                let cert = X509::from_pem(pem)?;
                // getting the public and rsa keys
                let pub_key = cert.public_key()?;
                let rsa_key = pub_key.rsa()?;
                // configuring the buffer
                let buf_len = pub_key.size();
                let mut buffer = vec![0; buf_len];

                rsa_key.public_encrypt(self.initiator_password().as_bytes(), &mut buffer, Padding::PKCS1)?;
                Ok(base64::encode_block(&buffer))
            }
        } else if #[cfg(feature = "no_openssl")] {
            /// Generates security credentials
            /// M-Pesa Core authenticates a transaction by decrypting the security credentials.
            /// Security credentials are generated by encrypting the base64 encoded initiator password with M-Pesa’s public key,
            /// a X509 certificate. Returns base64 encoded string.
            ///
            /// # Errors
            /// Returns `EncryptionError` variant of `MpesaError`
            pub(crate) fn gen_security_credentials(&self) -> MpesaResult<String> {
                use rsa::pkcs8::DecodePublicKey; // required for RsaPublicKey::from_public_key_der
                use rsa::{Pkcs1v15Encrypt, RsaPublicKey};
                use x509_parser::pem::parse_x509_pem;

                use crate::errors::EncryptionErrors;

                let cert_data = self.certificate.as_bytes();
                let (_, pem) = parse_x509_pem(cert_data).map_err(EncryptionErrors::Pem)?;
                let x509 = pem.parse_x509().map_err(EncryptionErrors::X509)?;

                // Get the raw SubjectPublicKeyInfo (SPKI) bytes
                let spki_bytes = x509.tbs_certificate.subject_pki.raw;
                // Load the public key from the extracted DER bytes
                let public_key = RsaPublicKey::from_public_key_der(spki_bytes)
                    .map_err(rsa::pkcs8::Error::PublicKey)
                    .map_err(EncryptionErrors::PublicKey)?;

                let mut rng = rand::thread_rng();
                let encrypted = public_key
                    .encrypt(&mut rng, Pkcs1v15Encrypt, self.initiator_password().as_bytes())
                    .map_err(EncryptionErrors::RsaEncryption)?;

                Ok(encode_block(&encrypted))
            }
        }
    }

    /// Sends a request to the Safaricom API
    /// This method is used by all the builders to send requests to the
    /// Safaricom API
    pub(crate) async fn send<Req, Res>(&self, req: Request<Req>) -> MpesaResult<Res>
    where
        Req: Serialize + Send,
        Res: DeserializeOwned,
    {
        let auth = self.auth().await?;
        let req = Arc::new(req);
        let res = backoff::future::retry(ExponentialBackoff::default(), || async {
            execute::<Req, Res>(self, &req.clone(), auth.clone()).await
        })
        .await?;
        Ok(res)
    }
}

/// Sends a request to the Safaricom API
/// The function has a retry policy with expoential backoff
pub(crate) async fn execute<Req, Res>(client: &Mpesa, req: &Request<Req>, auth: String) -> BackoffMpesaResult<Res>
where
    Req: Serialize + Send,
    Res: DeserializeOwned,
{
    let url = format!("{}/{}", client.base_url, req.path);

    #[cfg(test)]
    let _ = env_logger::builder().try_init();

    let response = client
        .http_client
        .request(req.method.clone(), url)
        .bearer_auth(auth.clone())
        .header(reqwest::header::ACCEPT, "application/json")
        .json(&req.body)
        .send()
        .await
        .map_err(MpesaError::from)
        .map_err(MpesaError::to_retryable)?;

    if response.status().is_success() {
        let text = response
            .text()
            .await
            .map_err(MpesaError::from)
            .map_err(MpesaError::to_retryable)?;
        let body: Res = serde_json::from_str(&text)
            .inspect_err(|e| log::error!("error decoding body err: {}: {}", e, text))
            .map_err(MpesaError::from)
            .map_err(MpesaError::to_retryable)?;
        Ok(body)
    } else {
        let status = response.status();
        let is_content_type_html = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap_or_default())
            .map(|s| s.contains("text/html"))
            .unwrap_or(false);
        let url = response.url().to_string();
        let path = response.url().path().to_string();
        let text = response
            .text()
            .await
            .map_err(MpesaError::from)
            .map_err(MpesaError::to_retryable)?;
        let body: ResponseError = serde_json::from_str(&text).map_err(|err| {
            if (is_content_type_html && status == reqwest::StatusCode::FORBIDDEN)
                || status == reqwest::StatusCode::TOO_MANY_REQUESTS
                || status == reqwest::StatusCode::SERVICE_UNAVAILABLE
            {
                log::debug!(
                    "Transient Error Occurred url: {} status: {} is_html: {}. Can Retry",
                    path,
                    status,
                    is_content_type_html
                );
                MpesaError::to_retryable(MpesaError::TransientError)
            } else {
                log::error!(
                    "error decoding body url: {} status: {} is html: {} err: {} : {}",
                    url,
                    status,
                    is_content_type_html,
                    err,
                    text
                );
                MpesaError::to_retryable(MpesaError::from(err))
            }
        })?;
        Err(MpesaError::to_retryable(MpesaError::Service(body)))
    }
}

pub struct Request<Body: Serialize + Send> {
    pub method: reqwest::Method,
    pub path: &'static str,
    pub body: Body,
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::Sandbox;

    #[test]
    fn test_setting_initator_password() {
        let client = Mpesa::new("consumer_key", "consumer_secret", Sandbox);
        assert_eq!(client.initiator_password(), DEFAULT_INITIATOR_PASSWORD);
        client.set_initiator_password("foo_bar");
        assert_eq!(client.initiator_password(), "foo_bar".to_string());
    }

    #[derive(Clone)]
    struct TestEnvironment;

    impl ApiEnvironment for TestEnvironment {
        fn base_url(&self) -> &str {
            "https://example.com"
        }

        fn get_certificate(&self) -> &str {
            // not a valid pem
            "certificate"
        }
    }

    #[test]
    fn test_custom_environment() {
        let client = Mpesa::new("consumer_key", "consumer_secret", TestEnvironment);
        assert_eq!(&client.base_url, "https://example.com");
        assert_eq!(&client.certificate, "certificate");
    }

    #[cfg(any(feature = "openssl", feature = "no_openssl"))]
    #[test]
    #[should_panic]
    fn test_gen_security_credentials_fails_with_invalid_pem() {
        let client = Mpesa::new("consumer_key", "consumer_secret", TestEnvironment);
        let _ = client.gen_security_credentials().unwrap();
    }

    #[cfg(feature = "no_openssl")]
    #[test]
    fn test_gen_security_credentials_no_openssl() {
        use rsa::pkcs8::DecodePublicKey; // required for RsaPublicKey::from_public_key_der
        use rsa::{Pkcs1v15Encrypt, RsaPublicKey};
        use x509_parser::pem::parse_x509_pem;

        use crate::errors::EncryptionErrors;

        let client = Mpesa::new("consumer_key", "consumer_secret", Sandbox);
        let rr = client.gen_security_credentials();
        assert!(rr.is_ok());
        let r = rr.unwrap();
        println!("Generated security credentials: {}", r);

        let cert_data = client.certificate.as_bytes();
        let pr = parse_x509_pem(cert_data).map_err(EncryptionErrors::Pem);
        assert!(pr.is_ok());
        match pr {
            Ok((_, pem)) => {
                print_certificate_info(&pem);
                let x509r = pem
                    .parse_x509()
                    .map_err(EncryptionErrors::X509)
                    .inspect_err(|e| println!("Error parsing x509 certificate: {}", e));
                assert!(x509r.is_ok());
                let x509 = x509r.unwrap();
                let spki_bytes = x509.tbs_certificate.subject_pki.raw;
                let public_keyr = RsaPublicKey::from_public_key_der(spki_bytes)
                    .map_err(rsa::pkcs8::Error::PublicKey)
                    .map_err(EncryptionErrors::PublicKey)
                    .inspect_err(|e| println!("Error parsing public key: {}", e));
                assert!(public_keyr.is_ok());
                let public_key = public_keyr.unwrap();
                let mut rng = rand::thread_rng();
                let encryptedr = public_key
                    .encrypt(&mut rng, Pkcs1v15Encrypt, client.initiator_password().as_bytes())
                    .map_err(EncryptionErrors::RsaEncryption);
                assert!(encryptedr.is_ok());
                let encrypted = encryptedr.unwrap();
                let r = encode_block(&encrypted);
                println!("Generated security credentials: {}", r);
            }
            Err(e) => println!("Error parsing pem: {}", e),
        }
    }

    #[cfg(feature = "no_openssl")]
    fn print_certificate_info(pem: &x509_parser::pem::Pem) {
        match pem.parse_x509() {
            Ok(x509) => {
                println!("Subject: {}", x509.subject());
                println!("Version: {}", x509.version());
                println!("Issuer: {}", x509.issuer());
                println!("X.509 serial: {}", x509.tbs_certificate.raw_serial_as_string());
                println!("Validity:");
                println!("    NotBefore: {}", x509.validity().not_before);
                println!("    NotAfter:  {}", x509.validity().not_after);
                println!("    is_valid:  {}", x509.validity().is_valid());
                println!("Subject Public Key Info:");
                print_x509_ski(x509.public_key());
            }
            Err(e) => println!("Error parsing pem: {}", e),
        }
    }

    #[cfg(feature = "no_openssl")]
    fn print_x509_ski(public_key: &x509_parser::prelude::SubjectPublicKeyInfo) {
        use x509_parser::public_key::PublicKey;
        match public_key.parsed() {
            Ok(PublicKey::RSA(rsa)) => {
                println!("    RSA Public Key: ({} bit)", rsa.key_size());
                for l in format_number_to_hex_with_colon(rsa.modulus, 16) {
                    println!("        {l}");
                }
                if let Ok(e) = rsa.try_exponent() {
                    println!("    exponent: 0x{e:x} ({e})");
                } else {
                    println!("    exponent: <INVALID>:");
                    print_hex_dump(rsa.exponent, 32);
                }
            }
            Ok(PublicKey::EC(ec)) => {
                println!("    EC Public Key: ({} bit)", ec.key_size());
                for l in format_number_to_hex_with_colon(ec.data(), 16) {
                    println!("        {l}");
                }
            }
            Ok(PublicKey::DSA(y)) => {
                println!("    DSA Public Key: ({} bit)", 8 * y.len());
                for l in format_number_to_hex_with_colon(y, 16) {
                    println!("        {l}");
                }
            }
            Ok(PublicKey::GostR3410(y)) => {
                println!("    GOST R 34.10-94 Public Key: ({} bit)", 8 * y.len());
                for l in format_number_to_hex_with_colon(y, 16) {
                    println!("        {l}");
                }
            }
            Ok(PublicKey::GostR3410_2012(y)) => {
                println!("    GOST R 34.10-2012 Public Key: ({} bit)", 8 * y.len());
                for l in format_number_to_hex_with_colon(y, 16) {
                    println!("        {l}");
                }
            }
            Ok(PublicKey::Unknown(b)) => {
                println!("    Unknown key type");
                print_hex_dump(b, 256);
            }
            Err(_) => {
                println!("    INVALID PUBLIC KEY");
            }
        }
    }

    #[cfg(feature = "no_openssl")]
    fn print_hex_dump(bytes: &[u8], max_len: usize) {
        use x509_parser::nom::HexDisplay;
        let m = std::cmp::min(bytes.len(), max_len);
        print!("{}", &bytes[..m].to_hex(16));
        if bytes.len() > max_len {
            println!("... <continued>");
        }
    }

    fn format_number_to_hex_with_colon(b: &[u8], row_size: usize) -> Vec<String> {
        let mut v = Vec::with_capacity(1 + b.len() / row_size);
        for r in b.chunks(row_size) {
            let s = r
                .iter()
                .fold(String::with_capacity(3 * r.len()), |a, b| a + &format!("{b:02x}:"));
            v.push(s)
        }
        v
    }
}
