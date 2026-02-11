use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;

use crate::{Mpesa, MpesaError, MpesaResult, ResponseError};

const AUTHENTICATION_URL: &str = "/oauth/v1/generate?grant_type=client_credentials";

pub(crate) async fn auth(client: &Mpesa) -> MpesaResult<String> {
    let url = format!("{}{}", client.base_url, AUTHENTICATION_URL);

    let response = client
        .http_client
        .get(&url)
        .basic_auth(client.consumer_key(), Some(&client.consumer_secret()))
        .send()
        .await?;

    if response.status().is_success() {
        let value = response.json::<AuthenticationResponse>().await?;
        let access_token = value.access_token;

        return Ok(access_token);
    }

    let error = response.json::<ResponseError>().await?;
    Err(MpesaError::Service(error))
}

/// Response returned from the authentication function
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticationResponse {
    /// Access token which is used as the Bearer-Auth-Token
    pub access_token: String,
    /// Expiry time in seconds
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub expires_in: u64,
}

impl std::fmt::Display for AuthenticationResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "token :{} expires in: {}", self.access_token, self.expires_in)
    }
}
