use std::str::FromStr;

use figment::Figment;
use figment::providers::{Env, Serialized};
use mpesa::services::{
    B2cResponse, C2bRegisterResponse, C2bSimulateResponse, MpesaExpressQueryResponse, MpesaExpressResponse,
    TransactionStatusResponse,
};
use mpesa::{ApiEnvironment, Environment as MpesaEnvironment, Mpesa, MpesaError};
use serde::{Deserialize, Serialize};

pub type AppResult<T> = std::result::Result<T, Box<figment::Error>>;

/// Mpesa client for interacting with the Daraja API.
#[derive(Debug, Clone)]
pub struct MpesaClient {
    /// Configuration for our Mpesa client.
    pub(crate) config: MpesaConfig,
    /// The mpesa crate's client for interacting with the Daraja API.
    inner: Mpesa,
}

/// Configuration for the Mpesa client.
/// This struct holds all necessary details for initializing the mpesa crate's
/// client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MpesaConfig {
    /// The consumer key provided by Safaricom for your application.
    pub consumer_key: String,
    /// The consumer secret provided by Safaricom for your application.
    pub consumer_secret: String,
    /// The business short code provided by Safaricom for your application.
    /// Used for stk push requests to generate the password.
    pub business_short_code: String,
    /// The pass key provided by Safaricom for your application.
    /// Used for stk push requests to generate the password.
    pub passkey: String,
    /// The initiator's name provided by Safaricom for your application
    /// Used for account_balance,c2b,b2b,b2c,reversal,transaction_status
    pub initiator_name: String,
    /// The initiator's password provided by Safaricom for your application
    pub initiator_password: String,
    /// The callback url to receive stk push notifications
    pub express_callback_url: String,
    /// The url to receive c2b payment confirmations
    pub c2b_confirmation_url: String,
    /// The url to receive c2b payment validation requests
    pub c2b_validation_url: String,
    /// The url to receive b2c payment results
    pub b2c_result_url: String,
    /// The url to receive b2c payment timeouts
    pub b2c_timeout_url: String,
    /// The url to receive b2b payment results
    pub b2b_result_url: String,
    /// The url to receive b2b payment timeouts
    pub b2b_timeout_url: String,
    /// The url to receive account balance results
    pub bal_result_url: String,
    /// The url to receive account balance timeouts
    pub bal_timeout_url: String,
    /// The url to receive transaction reversal results
    pub txn_reversal_result_url: String,
    /// The url to receive transaction reversal timeouts
    pub txn_reversal_timeout_url: String,
    /// The url to receive transaction status results
    pub txn_status_result_url: String,
    /// The url to receive transaction status timeouts
    pub txn_status_timeout_url: String,
    /// The callback url to receive bill manager onboard notifications
    pub onboard_bm_callback_url: String,
    /// short code to use for testing c2b/b2b/b2c/transaction_status payments
    pub party_a: String,
    /// Another test short code to use for testing b2b payments
    pub party_b: String,
    /// The test phone number to use for testing
    pub msisdn: String,
    /// The environment to use case-insensitive: any of ("sandbox" | "dev" |
    /// "test") or ("production" | "live" | "prod")
    pub environment: String,
}

impl MpesaConfig {
    /// Returns the environment to use for the Mpesa client.
    ///
    /// You can implement your own evironment to use your own environment for the
    /// mpesa client. This will allow you to specify the X509 certificates to
    /// use for the mpesa client.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// pub enum MyEnvironment {
    ///     Production,
    ///     Sandbox,
    /// }
    ///
    /// impl ApiEnvironment for MyEnvironment {
    ///     /// Matches to base_url based on `MyEnvironment` variant
    ///     fn base_url(&self) -> &str {
    ///         match self {
    ///             MyEnvironment::Production => "https://api.safaricom.co.ke",
    ///             MyEnvironment::Sandbox => "https://sandbox.safaricom.co.ke",
    ///         }
    ///     }
    ///
    ///     /// Match to X509 public key certificate based on `MyEnvironment`
    ///     fn get_certificate(&self) -> &str {
    ///         match self {
    ///             MyEnvironment::Production => {
    ///                 include_str!("./certificates/ProductionCertificate.cer")
    ///             }
    ///             MyEnvironment::Sandbox => {
    ///                 include_str!("./certificates/SandboxCertificate.cer")
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn get_environment(&self) -> impl ApiEnvironment {
        match MpesaEnvironment::from_str(&self.environment) {
            Ok(env) => env,
            Err(_) => match self.environment.as_str().to_lowercase().as_str() {
                "dev" | "test" => MpesaEnvironment::Sandbox,
                "live" | "prod" => MpesaEnvironment::Production,
                _ => MpesaEnvironment::Sandbox,
            },
        }
    }

    /// The consumer key provided by Safaricom for your application.
    pub fn consumer_key(&self) -> &str {
        &self.consumer_key
    }

    /// The consumer secret provided by Safaricom for your application.
    pub fn consumer_secret(&self) -> &str {
        &self.consumer_secret
    }

    /// The business short code provided by Safaricom for your application.
    /// Used for stk push requests to generate the password.
    pub fn business_short_code(&self) -> &str {
        &self.business_short_code
    }

    /// The pass key provided by Safaricom for your application.
    /// Used to generate the password for stk push requests.
    pub fn passkey(&self) -> &str {
        &self.passkey
    }

    /// The initiator's name provided by Safaricom for your application
    pub fn initiator_name(&self) -> &str {
        &self.initiator_name
    }

    /// The initiator's password provided by Safaricom for your application
    pub fn initiator_password(&self) -> &str {
        &self.initiator_password
    }

    /// The callback url to receive stk push result notifications
    pub fn express_callback_url(&self) -> &str {
        &self.express_callback_url
    }

    /// The url to receive c2b payment confirmations
    pub fn c2b_confirmation_url(&self) -> &str {
        &self.c2b_confirmation_url
    }

    /// The url to receive c2b payment validation requests
    pub fn c2b_validation_url(&self) -> &str {
        &self.c2b_validation_url
    }

    /// The url to receive b2c payment results
    pub fn b2c_result_url(&self) -> &str {
        &self.b2c_result_url
    }

    /// The url to receive b2c payment timeouts
    pub fn b2c_timeout_url(&self) -> &str {
        &self.b2c_timeout_url
    }

    /// The url to receive b2b payment results
    pub fn b2b_result_url(&self) -> &str {
        &self.b2b_result_url
    }

    /// The url to receive b2b payment timeouts
    pub fn b2b_timeout_url(&self) -> &str {
        &self.b2b_timeout_url
    }

    /// The url to receive account balance results
    pub fn bal_result_url(&self) -> &str {
        &self.bal_result_url
    }

    /// The url to receive account balance timeouts
    pub fn bal_timeout_url(&self) -> &str {
        &self.bal_timeout_url
    }

    /// The url to receive transaction status results
    pub fn txn_status_result_url(&self) -> &str {
        &self.txn_status_result_url
    }

    /// The url to receive transaction status timeouts
    pub fn txn_status_timeout_url(&self) -> &str {
        &self.txn_status_timeout_url
    }

    /// The url to receive transaction reversal results
    pub fn txn_reversal_result_url(&self) -> &str {
        &self.txn_reversal_result_url
    }

    /// The url to receive transaction reversal timeouts
    pub fn txn_reversal_timeout_url(&self) -> &str {
        &self.txn_reversal_timeout_url
    }

    /// The callback url to receive bill manager onboard notifications
    pub fn onboard_bm_callback_url(&self) -> &str {
        &self.onboard_bm_callback_url
    }

    /// test shortcode of the organization receiving the transaction.
    /// short code to use for
    /// account_balance/c2b/b2b/b2c/reversal/transaction_status payments
    pub fn shortcode_a(&self) -> &str {
        &self.party_a
    }

    /// Another test short code to use for testing b2b payments
    /// short code to use for b2b payments
    pub fn shortcode_b(&self) -> &str {
        &self.party_b
    }

    /// The test phone number to use for testing b2c payments and/or stk push
    /// payments Without the leading + sign
    /// Can be also used as the `party_a` the phone number sending money value,
    /// during stk push requests
    pub fn msisdn(&self) -> &str {
        &self.msisdn
    }
}

impl MpesaClient {
    /// Registers the callback urls for C2B payments
    #[cfg(feature = "c2b_register")]
    pub async fn register_c2b_urls(&self) -> Result<C2bRegisterResponse, MpesaError> {
        self.inner
            .c2b_register()
            .short_code(self.config.shortcode_a())
            .confirmation_url(self.config.c2b_confirmation_url())
            .validation_url(self.config.c2b_validation_url())
            .send()
            .await
    }

    /// Simulates C2B payments
    ///
    /// # Arguments
    ///
    /// * `amount` - The amount to send
    /// * `bill_ref` - The bill reference to use for the simulation
    #[cfg(feature = "c2b_simulate")]
    pub async fn simulate_c2b<N: Into<f64>>(
        &self,
        amount: N,
        bill_ref: &str,
    ) -> Result<C2bSimulateResponse, MpesaError> {
        self.inner
            .c2b_simulate()
            .short_code(self.config.shortcode_a())
            .msisdn(self.config.msisdn())
            .amount(amount)
            .bill_ref_number(bill_ref)
            .send()
            .await
    }

    /// Initiates a stk push request to a phone number with the specified amount
    ///
    /// # Arguments
    ///
    /// * `till_number` - Optional till number to override the business short code from config
    /// * `phone_number` - The phone number to send the stk push request to
    /// * `amount` - The amount to send
    /// * `acct_ref` - The account reference to use for the stk push request max 12 chars
    /// * `description` - The description to use for the stk push request max 13 chars
    #[cfg(feature = "express")]
    pub async fn stk_push_request<N: Into<u32>>(
        &self,
        till_number: Option<&str>,
        phone_number: &str,
        amount: N,
        acct_ref: &str,
        description: &str,
    ) -> Result<MpesaExpressResponse, MpesaError> {
        self.inner
            .express_request()
            .business_short_code(self.config.business_short_code())
            .phone_number(phone_number)
            .party_a(phone_number)
            .party_b(till_number.unwrap_or(self.config.business_short_code()))
            .amount(amount.into())
            .account_ref(acct_ref) // max 12 chars
            .transaction_desc(description) // max 13 chars
            .transaction_type(mpesa::CommandId::CustomerPayBillOnline)
            .pass_key(self.config.passkey())
            .try_callback_url(self.config.express_callback_url())?
            .build()?
            .send()
            .await
    }

    /// Checks the status of a stk push request
    ///
    /// # Arguments
    ///
    /// * `checkout_request_id` - The checkout request id to check
    ///
    /// # Returns
    ///
    /// * `MpesaExpressQueryResponse` - The response from the Daraja API
    ///   - `ResultCode` of 0 indicates successful transaction processing
    ///   - `ResultCode` of any other value indicates failed transaction
    ///   - `ResponseCode` of 0 indicates successful transaction submission
    ///   - `ResponseCode` of any other value indicates failed transaction submission
    #[cfg(feature = "express")]
    pub async fn stk_push_status(&self, checkout_request_id: &str) -> Result<MpesaExpressQueryResponse, MpesaError> {
        self.inner
            .express_query()
            .business_short_code(self.config.business_short_code())
            .checkout_request_id(checkout_request_id)
            .pass_key(self.config.passkey())
            .build()?
            .send()
            .await
    }

    /// Initiates a b2c payment to a phone number with the specified amount
    ///
    /// # Arguments
    /// * `phone_number` - The phone number to send the b2c payment to
    /// * `amount` - The amount to send
    /// * `originator_conversation_id` - The originator conversation id to use for the b2c payment
    /// * `remarks` - Optional remarks to use for the b2c payment
    /// * `occasion` - Optional occasion to use for the b2c payment
    #[cfg(feature = "b2c")]
    pub async fn b2c_payment<N: Into<f64>>(
        &self,
        phone_number: &str,
        amount: N,
        originator_conversation_id: &str,
        remarks: Option<&str>,
        occasion: Option<&str>,
    ) -> Result<B2cResponse, MpesaError> {
        self.inner
            .b2c(self.config.initiator_name())
            .command_id(mpesa::CommandId::SalaryPayment)
            .originator_conversation_id(originator_conversation_id)
            .amount(amount.into())
            .party_a(self.config.shortcode_a())
            .party_b(phone_number)
            .remarks(remarks.unwrap_or("Test"))
            .occasion(occasion.unwrap_or("Service Provider Payout"))
            .result_url(self.config.b2c_result_url())
            .timeout_url(self.config.b2c_timeout_url())
            .send()
            .await
    }

    /// Checks the status of a b2c/b2b/c2b payment
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - The transaction id to check
    #[cfg(feature = "transaction_status")]
    pub async fn transaction_status(&self, transaction_id: &str) -> Result<TransactionStatusResponse, MpesaError> {
        self.inner
            .transaction_status(self.config.initiator_name())
            .party_a(self.config.shortcode_a())
            .transaction_id(transaction_id)
            .result_url(self.config.txn_status_result_url())
            .timeout_url(self.config.txn_status_timeout_url())
            .send()
            .await
    }
}

impl From<MpesaConfig> for Mpesa {
    fn from(config: MpesaConfig) -> Self {
        Self::from(&config)
    }
}

impl From<&MpesaConfig> for Mpesa {
    fn from(config: &MpesaConfig) -> Self {
        let client = Self::new(
            config.consumer_key(),
            config.consumer_secret(),
            config.get_environment(),
        );
        client.set_initiator_password(config.initiator_password());
        client
    }
}

impl From<MpesaConfig> for MpesaClient {
    fn from(config: MpesaConfig) -> Self {
        let client = Mpesa::from(&config);
        Self { config, inner: client }
    }
}

impl From<&MpesaConfig> for MpesaClient {
    fn from(config: &MpesaConfig) -> Self {
        let client = Mpesa::from(config);
        Self {
            config: config.clone(),
            inner: client,
        }
    }
}

impl MpesaConfig {
    /// Creates a figment instance for the MpesaConfig struct.
    pub fn figment() -> Figment {
        Figment::from(Serialized::defaults(Self::default())).merge(Self::figment_sources())
    }

    /// Creates a figment instance for the MpesaConfig struct with the sources for
    /// the configuration.
    pub fn figment_sources() -> Figment {
        Figment::new().merge(Env::raw())
    }

    /// Returns a MpesaConfig struct with the default profile selected.
    pub fn get_default() -> AppResult<Self> {
        Self::figment().extract().map_err(Box::new)
    }
}

impl figment::Provider for MpesaConfig {
    fn metadata(&self) -> figment::Metadata {
        figment::Metadata::named("MpesaConfig")
    }

    fn data(&self) -> figment::Result<figment::value::Map<figment::Profile, figment::value::Dict>> {
        Serialized::defaults(Self::default()).data()
    }
}

fn get_test_config() -> std::sync::Arc<MpesaConfig> {
    dotenvy::dotenv().ok();
    let _ = env_logger::builder().try_init();
    std::sync::Arc::new(MpesaConfig::get_default().unwrap())
}

#[test]
fn test_load_mpesa_config() {
    dotenvy::dotenv().ok();
    let _ = env_logger::builder().try_init();
    let config_res = MpesaConfig::get_default();
    assert!(config_res.is_ok());
    let config = config_res.unwrap();
    log::trace!("Mpesa config: {:#?}\n", config);
    assert!(!config.consumer_key().is_empty());
    assert!(!config.consumer_secret().is_empty());
    assert!(!config.business_short_code().is_empty());
    assert!(!config.passkey().is_empty());
    assert!(!config.initiator_name().is_empty());
    assert!(!config.initiator_password().is_empty());
    assert!(!config.environment.is_empty());
}

#[test]
fn test_from_mpesa_config() {
    let config = get_test_config();
    let client = MpesaClient::from(config.as_ref());
    assert_eq!(client.config, *(config.as_ref()));
}

fn get_test_client() -> std::sync::Arc<MpesaClient> {
    let config = get_test_config();
    std::sync::Arc::new(MpesaClient::from(config.as_ref()))
}

use std::sync::LazyLock;

static CLIENT: LazyLock<std::sync::Arc<MpesaClient>> = LazyLock::new(|| get_test_client());

#[tokio::test]
#[serial_test::serial]
async fn test_auth() {
    let client = &*CLIENT;
    log::debug!("Has Cached Auth: {}", client.inner.has_cached_auth());
    let res = client.inner.is_connected().await;
    log::info!("Is connected: {}", res);
    log::debug!("Has Cached Auth: {}\n", client.inner.has_cached_auth());
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
}

#[cfg(feature = "c2b_register")]
#[tokio::test]
#[serial_test::serial]
async fn test_register_c2b_urls() {
    let client = &*CLIENT;
    log::debug!("Has Cached Auth: {}", client.inner.has_cached_auth());
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    let res = client.register_c2b_urls().await;
    match res {
        Ok(res) => log::info!("C2B register response: {:#?}\n", res),
        Err(err) => print_mpesa_error("C2B register error", err),
    }
}

#[cfg(feature = "c2b_simulate")]
#[tokio::test]
#[serial_test::serial]
async fn test_simulate_c2b() {
    let client = &*CLIENT;
    log::debug!("Has Cached Auth: {}", client.inner.has_cached_auth());
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let res = client.simulate_c2b(1.0, "123456").await;
    match res {
        Ok(res) => log::info!("C2B simulate response: {:#?}\n", res),
        Err(err) => print_mpesa_error("C2B simulate error", err),
    }
}

#[cfg(feature = "express")]
#[tokio::test]
#[serial_test::serial]
async fn test_stk_push() {
    let client = &*CLIENT;
    log::debug!("Has Cached Auth: {}", client.inner.has_cached_auth());
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let res = client
        .stk_push_request(None, "254741997729", 1u32, "123456", "ciqu escrow deposit")
        .await;
    let mut co_req_id = String::new();
    match res {
        Ok(res) => {
            co_req_id = res.checkout_request_id.clone();
            log::info!("STK push response: {:#?}", res)
        }
        Err(err) => print_mpesa_error("STK push error", err),
    }
    log::debug!("Has Cached Auth: {}", client.inner.has_cached_auth());
    if !co_req_id.is_empty() {
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        let res = client.stk_push_status(&co_req_id).await;
        match res {
            Ok(res) => log::info!("STK push status response: {:#?}\n", res),
            Err(err) => print_mpesa_error("STK push status error", err),
        }
    }
    log::debug!("Has Cached Auth: {}", client.inner.has_cached_auth());
    tokio::time::sleep(std::time::Duration::from_secs(8)).await;
    let res = client
        .stk_push_request(Some("5050980"), "254741997729", 1u32, "123456", "escrow deposit")
        .await;
    let mut co_req_id = String::new();
    match res {
        Ok(res) => {
            co_req_id = res.checkout_request_id.clone();
            log::info!("STK push response: {:#?}", res)
        }
        Err(err) => print_mpesa_error("STK push error", err),
    }
    log::debug!("Has Cached Auth: {}", client.inner.has_cached_auth());
    if !co_req_id.is_empty() {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let res = client.stk_push_status(&co_req_id).await;
        match res {
            Ok(res) => log::info!("STK push status response: {:#?}\n", res),
            Err(err) => print_mpesa_error("STK push status error", err),
        }
    }
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
}

#[cfg(feature = "b2c")]
#[tokio::test]
#[serial_test::serial]
async fn test_b2c_payment() {
    let client = &*CLIENT;
    log::debug!("Has Cached Auth: {}", client.inner.has_cached_auth());
    let date = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
    let date = format!("Test-{}", date);
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let res = client
        .b2c_payment(client.config.msisdn(), 20.0, &date, Some("Test"), None)
        .await;
    let mut originator_conv_id = String::new();
    match res {
        Ok(res) => {
            originator_conv_id = res.originator_conversation_id.clone();
            log::info!("B2C payment response: {:#?}", res)
        }
        Err(err) => print_mpesa_error("B2C payment error", err),
    };
    log::debug!("Has Cached Auth: {}", client.inner.has_cached_auth());
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    #[cfg(feature = "transaction_status")]
    if !originator_conv_id.is_empty() {
        tokio::time::sleep(std::time::Duration::from_secs(6)).await;
        let res = client.transaction_status(&originator_conv_id).await;
        match res {
            Ok(res) => log::info!("B2C payment status response: {:#?}\n", res),
            Err(err) => print_mpesa_error("B2C payment status error", err),
        }
    }
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
}

#[cfg(feature = "transaction_status")]
#[tokio::test]
#[serial_test::serial]
async fn test_transaction_status() {
    let client = &*CLIENT;
    log::debug!("Has Cached Auth: {}", client.inner.has_cached_auth());
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let res = client.transaction_status("UB00000000").await;
    match res {
        Ok(res) => log::info!("Transaction status response: {:?}\n", res),
        Err(err) => print_mpesa_error("Transaction status error", err),
    }
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
}

fn print_mpesa_error(prefix: &str, err: MpesaError) {
    log::warn!("{prefix}: {:?}\n", err)
}
