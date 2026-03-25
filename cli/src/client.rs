use anyhow::{bail, Context, Result};
use reqwest::Client;
use std::time::Duration;

use crate::types::*;

pub struct SyncfuClient {
    base_url: String,
    client: Client,
}

impl SyncfuClient {
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build HTTP client");
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    pub async fn send_notification(&self, req: &NotifyRequest) -> Result<NotifyResponse> {
        let resp = self
            .client
            .post(format!("{}/notify", self.base_url))
            .json(req)
            .send()
            .await
            .context(connection_error(&self.base_url))?;

        if resp.status() == 201 || resp.status() == 200 {
            Ok(resp.json().await?)
        } else {
            bail!("server returned {}", resp.status());
        }
    }

    pub async fn update_notification(&self, id: &str, req: &UpdateRequest) -> Result<()> {
        let resp = self
            .client
            .post(format!("{}/notify/{}/update", self.base_url, id))
            .json(req)
            .send()
            .await
            .context(connection_error(&self.base_url))?;

        match resp.status().as_u16() {
            200 => Ok(()),
            404 => bail!("notification not found: {id}"),
            s => bail!("server returned {s}"),
        }
    }

    pub async fn trigger_action(&self, id: &str, action_id: &str) -> Result<WebhookResult> {
        let req = ActionRequest {
            action_id: action_id.to_string(),
        };
        let resp = self
            .client
            .post(format!("{}/notify/{}/action", self.base_url, id))
            .json(&req)
            .send()
            .await
            .context(connection_error(&self.base_url))?;

        match resp.status().as_u16() {
            200 => Ok(resp.json().await?),
            404 => bail!("notification not found: {id}"),
            s => bail!("server returned {s}"),
        }
    }

    pub async fn dismiss(&self, id: &str) -> Result<()> {
        let resp = self
            .client
            .post(format!("{}/notify/{}/dismiss", self.base_url, id))
            .send()
            .await
            .context(connection_error(&self.base_url))?;

        match resp.status().as_u16() {
            200 => Ok(()),
            404 => bail!("notification not found: {id}"),
            s => bail!("server returned {s}"),
        }
    }

    pub async fn dismiss_all(&self) -> Result<DismissAllResponse> {
        let resp = self
            .client
            .post(format!("{}/dismiss-all", self.base_url))
            .send()
            .await
            .context(connection_error(&self.base_url))?;

        if resp.status() == 200 {
            Ok(resp.json().await?)
        } else {
            bail!("server returned {}", resp.status());
        }
    }

    pub async fn health(&self) -> Result<HealthResponse> {
        let resp = self
            .client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .context(connection_error(&self.base_url))?;

        if resp.status() == 200 {
            Ok(resp.json().await?)
        } else {
            bail!("server returned {}", resp.status());
        }
    }

    pub async fn list_active(&self) -> Result<Vec<serde_json::Value>> {
        let resp = self
            .client
            .get(format!("{}/active", self.base_url))
            .send()
            .await
            .context(connection_error(&self.base_url))?;

        if resp.status() == 200 {
            Ok(resp.json().await?)
        } else {
            bail!("server returned {}", resp.status());
        }
    }
}

fn connection_error(server: &str) -> String {
    format!("cannot connect to syncfu at {server}. Is the app running?")
}
