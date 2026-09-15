use clap::{Parser, ValueEnum};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum KubeletServerIdentity {
    /// Verify the certificate against the node IP address.
    NodeIp,
    /// Verify the certificate against the Kubernetes node name.
    NodeName,
    /// Verify the certificate chain without verifying its subject alternative names.
    CaOnly,
}

#[derive(Debug, Parser)]
#[command(
    version,
    name = "node-scraper",
    about = "Fetch metrics from a Kubernetes node and send them to cluster-aggregator"
)]
pub struct CliArgs {
    /// CA certificate bundle used to verify the kubelet certificate. When not
    /// specified, kubelet certificate verification is disabled.
    #[arg(long)]
    pub kubelet_ca_cert_file: Option<String>,

    /// Identity used to verify the kubelet certificate.
    #[arg(long, value_enum, default_value = "node-ip")]
    pub kubelet_server_identity: KubeletServerIdentity,

    /// Kubelet stats poll interval in seconds. Must be greater than zero.
    #[arg(long, default_value = "60", value_parser = parse_positive_seconds)]
    pub kubelet_stats_poll_interval: Duration,

    /// Kubelet health poll interval in seconds. Must be greater than zero.
    #[arg(long, default_value = "60", value_parser = parse_positive_seconds)]
    pub kubelet_health_poll_interval: Duration,

    /// System-agent poll interval in seconds. Must be greater than zero.
    #[arg(long, default_value = "60", value_parser = parse_positive_seconds)]
    pub system_agent_poll_interval: Duration,

    /// Timeout in seconds for each system-agent execution. Must be greater
    /// than zero.
    #[arg(long, default_value = "15", value_parser = parse_positive_seconds)]
    pub system_agent_timeout: Duration,

    /// Namespace that cluster-aggregator lives in (used for constructing the
    /// URL to send metrics to).
    #[arg(long, default_value = "checkmk-monitoring")]
    pub cluster_aggregator_namespace: String,

    /// Service name that cluster-aggregator responds on (used for constructing
    /// the URL to send metrics to).
    #[arg(long, default_value = "checkmk-agent-cluster-aggregator")]
    pub cluster_aggregator_service: String,

    /// Port to talk to cluster-aggregator on (corresponds to the service
    /// specified via --cluster-aggregator-service).
    #[arg(long, default_value_t = 10050)]
    pub cluster_aggregator_port: u16,

    /// CA Certificate for connecting to cluster-aggregator. When not
    /// specified, HTTP is used.
    #[arg(long)]
    pub cluster_aggregator_ca_cert_file: Option<String>,
}

fn parse_positive_seconds(value: &str) -> Result<Duration, String> {
    let seconds = value
        .parse::<u64>()
        .map_err(|_| "expected a positive whole number of seconds".to_string())?;
    if seconds == 0 {
        return Err("must be greater than zero".to_string());
    }
    Ok(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scraper_timings_have_defaults_and_accept_overrides_in_seconds() {
        let defaults = CliArgs::try_parse_from(["node-scraper"]).expect("valid defaults");
        assert_eq!(
            defaults.kubelet_server_identity,
            KubeletServerIdentity::NodeIp
        );
        assert_eq!(
            defaults.kubelet_stats_poll_interval,
            Duration::from_secs(60)
        );
        assert_eq!(
            defaults.kubelet_health_poll_interval,
            Duration::from_secs(60)
        );
        assert_eq!(defaults.system_agent_poll_interval, Duration::from_secs(60));
        assert_eq!(defaults.system_agent_timeout, Duration::from_secs(15));

        let configured = CliArgs::try_parse_from([
            "node-scraper",
            "--kubelet-ca-cert-file=/tmp/kubelet-ca.crt",
            "--kubelet-server-identity=node-name",
            "--kubelet-stats-poll-interval=30",
            "--kubelet-health-poll-interval=45",
            "--system-agent-poll-interval=90",
            "--system-agent-timeout=10",
        ])
        .expect("valid timing overrides");
        assert_eq!(
            configured.kubelet_ca_cert_file.as_deref(),
            Some("/tmp/kubelet-ca.crt")
        );
        assert_eq!(
            configured.kubelet_server_identity,
            KubeletServerIdentity::NodeName
        );
        assert_eq!(
            configured.kubelet_stats_poll_interval,
            Duration::from_secs(30)
        );
        assert_eq!(
            configured.kubelet_health_poll_interval,
            Duration::from_secs(45)
        );
        assert_eq!(
            configured.system_agent_poll_interval,
            Duration::from_secs(90)
        );
        assert_eq!(configured.system_agent_timeout, Duration::from_secs(10));
    }

    #[test]
    fn scraper_timings_reject_zero_and_invalid_seconds() {
        for flag in [
            "--kubelet-stats-poll-interval",
            "--kubelet-health-poll-interval",
            "--system-agent-poll-interval",
            "--system-agent-timeout",
        ] {
            for value in ["0", "-1", "0.5", "invalid"] {
                let error = CliArgs::try_parse_from(["node-scraper", &format!("{flag}={value}")])
                    .expect_err("invalid timing must be rejected before starting scrapers");
                assert_eq!(error.kind(), clap::error::ErrorKind::ValueValidation);
            }
        }
    }
}
