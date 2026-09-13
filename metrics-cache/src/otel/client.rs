//! This module contains the OTel client, to export OTLP metrics to an OTel
//! collector.

use anyhow::Context;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use prost::Message;
use std::path::Path;

use crate::error::Result;
use crate::otel::Error;

/// A client to export OTLP metrics to an OTel collector over OTLP/http.
pub struct OtelClient {
    client: reqwest::Client,
    export_url: String,
}

impl OtelClient {
    pub fn new(endpoint: &str, ca_cert_file: Option<&Path>) -> anyhow::Result<Self> {
        let mut client = reqwest::Client::builder();
        if let Some(path) = ca_cert_file {
            let pem = std::fs::read(path)
                .with_context(|| format!("could not read OTel CA bundle {}", path.display()))?;
            let certificates = reqwest::Certificate::from_pem_bundle(&pem)
                .context("could not parse OTel CA bundle")?;
            anyhow::ensure!(
                !certificates.is_empty(),
                "OTel CA bundle contains no certificates"
            );
            client = client.tls_certs_merge(certificates);
        }
        Ok(Self {
            client: client.build().context("could not build OTel HTTP client")?,
            export_url: format!("{}/v1/metrics", endpoint.trim_end_matches('/')),
        })
    }

    /// Send one export request to the collector.
    ///
    /// Errors on connection failure or a non-success HTTP status; the caller
    /// decides whether that is fatal (for the export loop it is not).
    pub(super) async fn export(&self, request: ExportMetricsServiceRequest) -> Result<()> {
        let response = self
            .client
            .post(&self.export_url)
            .header("content-type", "application/x-protobuf")
            .body(request.encode_to_vec())
            .send()
            .await
            .map_err(Error::Http)?;
        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<failed to read body>".to_string());
            return Err(Error::Rejected { status, body }.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn rejects_missing_or_invalid_ca_bundle() -> anyhow::Result<()> {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let path = std::env::temp_dir().join(format!("otel-ca-{}.pem", Uuid::new_v4()));
        assert!(OtelClient::new("https://localhost", Some(&path)).is_err());
        for pem in [
            "",
            "not a certificate",
            "-----BEGIN CERTIFICATE-----\ninvalid\n-----END CERTIFICATE-----",
        ] {
            std::fs::write(&path, pem)?;
            assert!(OtelClient::new("https://localhost", Some(&path)).is_err());
        }
        std::fs::remove_file(path)?;
        Ok(())
    }
}
