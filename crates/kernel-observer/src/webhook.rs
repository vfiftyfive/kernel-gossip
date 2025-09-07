use reqwest::Client;
use anyhow::Result;
use tracing::{info, error};
use crate::parser::EbpfEvent;

#[derive(Clone)]
pub struct WebhookClient {
    client: Client,
    webhook_url: String,
}

impl WebhookClient {
    pub fn new(webhook_url: String) -> Self {
        Self {
            client: Client::new(),
            webhook_url,
        }
    }

    pub async fn send_event(&self, event: EbpfEvent) -> Result<()> {
        info!("Sending eBPF event to operator: {:?}", event);
        
        let response = self
            .client
            .post(&self.webhook_url)
            .header("Content-Type", "application/json")
            .json(&event)
            .send()
            .await?;

        if response.status().is_success() {
            info!("✅ Successfully sent eBPF event to operator");
        } else {
            error!("❌ Failed to send event: {}", response.status());
        }

        Ok(())
    }
}