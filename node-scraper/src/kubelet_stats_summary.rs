use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, trace};

use crate::cli_args::CliArgs;
use crate::error::Result;
use crate::kubelet::KubeletClient;
use crate::payload::Payload;
use crate::scraper::Scraper;

pub(crate) struct KubeletStatsSummaryScraper {
    kubelet_client: KubeletClient,
    relay_client: Client,
    args: Arc<CliArgs>,
}

impl KubeletStatsSummaryScraper {
    pub(crate) fn new(
        args: Arc<CliArgs>,
        cluster_aggregator_client: Client,
        kubelet_client: KubeletClient,
    ) -> KubeletStatsSummaryScraper {
        KubeletStatsSummaryScraper {
            kubelet_client,
            relay_client: cluster_aggregator_client,
            args,
        }
    }
}

impl Scraper for KubeletStatsSummaryScraper {
    fn poll_interval(&self) -> Duration {
        self.args.kubelet_stats_poll_interval
    }

    fn relay_client(&self) -> Client {
        self.relay_client.clone()
    }

    fn args(&self) -> Arc<CliArgs> {
        self.args.clone()
    }

    /// Query the Kubelet /stats/summary response and return the raw JSON payload.
    ///
    /// We do not parse the payload or perform any calculations on it here, leaving
    /// these tasks for cluster-aggregator to do.
    async fn scrape(&self) -> Result<Payload> {
        trace!("reading kubelet token");
        let token = std::fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/token")?;
        debug!("fetching Kubelet /stats/summary");
        let response = self
            .kubelet_client
            .get("/stats/summary")
            .bearer_auth(token.trim())
            .send()
            .await?;
        debug!(status = %response.status(), "scrape complete");
        Ok(Payload::KubeletStatsSummary(response.bytes().await?))
    }
}
