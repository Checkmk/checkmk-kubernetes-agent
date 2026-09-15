use bytes::Bytes;
use reqwest::{Client, StatusCode};
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

use crate::cli_args::CliArgs;
use crate::error::Result;
use crate::kubelet::KubeletClient;
use crate::payload::Payload;
use crate::scraper::Scraper;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum KubeletHealth {
    Response { status_code: u16, response: String },
    ConnectionError { message: String },
}

pub(crate) struct KubeletHealthScraper {
    kubelet_client: KubeletClient,
    relay_client: Client,
    args: Arc<CliArgs>,
}

impl KubeletHealthScraper {
    pub(crate) fn new(
        args: Arc<CliArgs>,
        cluster_aggregator_client: Client,
        kubelet_client: KubeletClient,
    ) -> KubeletHealthScraper {
        KubeletHealthScraper {
            kubelet_client,
            relay_client: cluster_aggregator_client,
            args,
        }
    }
}

impl Scraper for KubeletHealthScraper {
    fn poll_interval(&self) -> Duration {
        self.args.kubelet_health_poll_interval
    }

    fn relay_client(&self) -> Client {
        self.relay_client.clone()
    }

    fn args(&self) -> Arc<CliArgs> {
        self.args.clone()
    }

    async fn scrape(&self) -> Result<Payload> {
        let node_name = self.kubelet_client.node_name().to_string();
        let token =
            tokio::fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/token")
                .await?;

        let url = self.kubelet_client.url("/healthz");
        let response = self
            .kubelet_client
            .get("/healthz")
            .bearer_auth(token.trim())
            .send()
            .await;

        let health = match response {
            Ok(response) => {
                let status_code = response.status();
                if status_code == StatusCode::OK {
                    debug!(status = %status_code, url, "kubelet healthz scrape complete");
                } else {
                    warn!(status = %status_code, url, "kubelet healthz scrape returned non-OK status");
                }
                let response_body = response.text().await?;
                KubeletHealth::Response {
                    status_code: status_code.as_u16(),
                    response: response_body,
                }
            }
            Err(e) => KubeletHealth::ConnectionError {
                message: e.to_string(),
            },
        };

        Ok(Payload::KubeletHealth {
            node_name,
            body: Bytes::from(serde_json::to_vec(&health)?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kubelet_health_response_wire_shape() {
        let health = KubeletHealth::Response {
            status_code: 200,
            response: "ok".to_string(),
        };
        assert_eq!(
            serde_json::to_string(&health).expect("KubeletHealth always serializes"),
            r#"{"status_code":200,"response":"ok"}"#
        );
    }

    #[test]
    fn kubelet_health_connection_error_wire_shape() {
        let health = KubeletHealth::ConnectionError {
            message: "connection refused".to_string(),
        };
        assert_eq!(
            serde_json::to_string(&health).expect("KubeletHealth always serializes"),
            r#"{"message":"connection refused"}"#
        );
    }
}
