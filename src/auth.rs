use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;

use crate::{Mpesa, MpesaError, MpesaResult, ResponseError};

const AUTHENTICATION_URL: &str = "/oauth/v1/generate";

pub(crate) async fn auth(client: &Mpesa) -> MpesaResult<String> {
    let url = format!("{}{}", client.base_url, AUTHENTICATION_URL);
    let params = [("grant_type", "client_credentials")];

    #[cfg(test)]
    let _ = env_logger::builder().try_init();

    let response = client
        .http_client
        .get(&url)
        .query(&params)
        .basic_auth(client.consumer_key(), Some(&client.consumer_secret()))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await?;

    if response.status().is_success() {
        let text = response.text().await?;
        let value: AuthenticationResponse = serde_json::from_str(&text)
            .inspect_err(|e| log::error!("Error Decoding error body: {} \nerr: {}", text, e))?;
        let access_token = value.access_token;
        let expires = std::time::Duration::from_secs(value.expires_in);
        let expiry = chrono::Utc::now() + expires;
        client.set_auth_token(access_token.clone(), expiry.timestamp());
        Ok(access_token)
    } else {
        let status = response.status();
        let url = response.url().to_string();
        let text = response.text().await?;
        let body: ResponseError = serde_json::from_str(&text)
            .inspect_err(|e| log::error!("{} {} Error Decoding error body: {} \nerr: {}", status, url, text, e))?;
        Err(MpesaError::Service(body))
    }
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
