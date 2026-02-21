use backoff_macro::backoff;
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;

use crate::errors::BackoffMpesaResult;
use crate::{Mpesa, MpesaError, ResponseError};

const AUTHENTICATION_URL: &str = "/oauth/v1/generate";

#[backoff]
pub(crate) async fn auth(client: &Mpesa) -> BackoffMpesaResult<String> {
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
        .await
        .map_err(MpesaError::from)
        .map_err(MpesaError::to_retryable)?;

    if response.status().is_success() {
        let text = response
            .text()
            .await
            .map_err(MpesaError::from)
            .map_err(MpesaError::to_retryable)?;
        let value: AuthenticationResponse = serde_json::from_str(&text)
            .inspect_err(|e| log::error!("error decoding body err: {}: {}", e, text))
            .map_err(MpesaError::from)
            .map_err(MpesaError::to_retryable)?;
        let access_token = value.access_token;
        let expires = std::time::Duration::from_secs(value.expires_in);
        let expiry = chrono::Utc::now() + expires;
        client.set_auth_token(access_token.clone(), expiry.timestamp());
        Ok(access_token)
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
                    status,
                    url,
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
