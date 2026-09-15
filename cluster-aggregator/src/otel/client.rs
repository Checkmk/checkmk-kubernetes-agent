//! This module contains the OTel client, to export OTLP metrics to an OTel
//! collector.

use anyhow::Context;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use prost::Message;
use std::path::Path;

use crate::error::Result;
use crate::otel::Error;

/// Basic-auth credentials for the OTel collector.
pub struct BasicAuth {
    pub username: String,
    pub password: Option<String>,
}

/// A client to export OTLP metrics to an OTel collector over OTLP/http.
pub struct OtelClient {
    client: reqwest::Client,
    export_url: String,
    basic_auth: Option<BasicAuth>,
}

impl OtelClient {
    pub fn new(
        endpoint: &str,
        ca_cert_file: Option<&Path>,
        basic_auth: Option<BasicAuth>,
    ) -> anyhow::Result<Self> {
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
            basic_auth,
        })
    }

    /// Build the export request, carrying the basic-auth header if credentials
    /// are configured.
    fn export_request(&self, request: ExportMetricsServiceRequest) -> reqwest::RequestBuilder {
        let mut builder = self
            .client
            .post(&self.export_url)
            .header("content-type", "application/x-protobuf")
            .body(request.encode_to_vec());
        if let Some(auth) = &self.basic_auth {
            builder = builder.basic_auth(&auth.username, auth.password.as_ref());
        }
        builder
    }

    /// Send one export request to the collector.
    ///
    /// Errors on connection failure or a non-success HTTP status; the caller
    /// decides whether that is fatal (for the export loop it is not).
    pub(super) async fn export(&self, request: ExportMetricsServiceRequest) -> Result<()> {
        let response = self
            .export_request(request)
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
        assert!(OtelClient::new("https://localhost", Some(&path), None).is_err());
        for pem in [
            "",
            "not a certificate",
            "-----BEGIN CERTIFICATE-----\ninvalid\n-----END CERTIFICATE-----",
        ] {
            std::fs::write(&path, pem)?;
            assert!(OtelClient::new("https://localhost", Some(&path), None).is_err());
        }
        std::fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn credentials_become_an_authorization_header() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        for (password, expected) in [
            (
                Some("McBobberson".to_string()),
                "Basic Ym9iOk1jQm9iYmVyc29u",
            ),
            (Some(String::new()), "Basic Ym9iOg=="),
            (None, "Basic Ym9iOg=="),
        ] {
            let auth = BasicAuth {
                username: "bob".to_string(),
                password,
            };
            let client = OtelClient::new("http://collector:4318", None, Some(auth))
                .expect("a client without a CA bundle should build");
            let request = client
                .export_request(ExportMetricsServiceRequest::default())
                .build()
                .expect("the export request should build");
            assert_eq!(request.headers()["authorization"], expected);
        }
    }

    #[test]
    fn without_credentials_there_is_no_authorization_header() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let client = OtelClient::new("http://collector:4318", None, None)
            .expect("a client without a CA bundle should build");
        let request = client
            .export_request(ExportMetricsServiceRequest::default())
            .build()
            .expect("the export request should build");
        assert!(request.headers().get("authorization").is_none());
    }
}
