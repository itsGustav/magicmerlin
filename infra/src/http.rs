//! Reqwest-based HTTP client wrapper with sane defaults.

use std::time::Duration;

use reqwest::{Client, Proxy};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::InfraError;

const DEFAULT_USER_AGENT: &str = "MagicMerlin/0.1";
const CONNECT_TIMEOUT_SECS: u64 = 30;
const READ_TIMEOUT_SECS: u64 = 120;

/// Thin HTTP client wrapper used across infra services.
#[derive(Clone, Debug)]
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    /// Builds a configured HTTP client with timeouts, TLS, user-agent, and proxy env support.
    pub fn new() -> Result<Self, InfraError> {
        let mut builder = Client::builder()
            .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
            .timeout(Duration::from_secs(READ_TIMEOUT_SECS))
            .user_agent(DEFAULT_USER_AGENT)
            .use_rustls_tls();

        if let Ok(proxy) = std::env::var("HTTP_PROXY") {
            if !proxy.trim().is_empty() {
                builder = builder.proxy(Proxy::http(&proxy).map_err(InfraError::Http)?);
            }
        }

        if let Ok(proxy) = std::env::var("HTTPS_PROXY") {
            if !proxy.trim().is_empty() {
                builder = builder.proxy(Proxy::https(&proxy).map_err(InfraError::Http)?);
            }
        }

        Ok(Self {
            client: builder.build().map_err(InfraError::Http)?,
        })
    }

    /// Returns the wrapped reqwest client for advanced use-cases.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Performs GET and deserializes JSON body.
    pub async fn get_json<T>(&self, url: &str) -> Result<T, InfraError>
    where
        T: DeserializeOwned,
    {
        let response = self.client.get(url).send().await?.error_for_status()?;
        response.json::<T>().await.map_err(InfraError::Http)
    }

    /// Performs POST with JSON body and deserializes JSON response.
    pub async fn post_json<B, T>(&self, url: &str, body: &B) -> Result<T, InfraError>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let response = self
            .client
            .post(url)
            .json(body)
            .send()
            .await?
            .error_for_status()?;
        response.json::<T>().await.map_err(InfraError::Http)
    }
}
