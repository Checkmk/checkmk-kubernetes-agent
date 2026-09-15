//! Self-health section(s) for Checkmk Kubernetes Agent.
//!
//! The sections here provide information about the current health of Checkmk
//! Kubernetes Agent and its two components (cluster-aggregator and
//! node-scraper).
//!
//! These get emitted on the _cluster_ piggyback host.

use serde::{Serialize, Serializer};
use std::collections::BTreeMap;
use std::time::Duration;

use crate::section::Section;
use crate::snapshot::self_health::SelfHealth;

/// By default serde serializes Duration as a dict of time units (mins, secs,
/// etc.). This is less useful for parsing later on, we want pure seconds, so
/// this is used to teach serde how to serialize a cache map from the snapshot's
/// [`SelfHealth`] in seconds.
fn duration_to_secs<S: Serializer>(duration: &Option<Duration>, ser: S) -> Result<S::Ok, S::Error> {
    duration.map(|d| d.as_secs_f64()).serialize(ser)
}

#[derive(Debug, Serialize)]
struct NodeScraperIngestionHealth<'a> {
    #[serde(serialize_with = "duration_to_secs")]
    last_heard_age_secs: Option<Duration>,
    #[serde(serialize_with = "duration_to_secs")]
    scrape_time_secs: Option<Duration>,
    version: Option<&'a str>,
    git_sha: Option<&'a str>,
}

impl<'a> From<&'a crate::snapshot::self_health::NodeScraperIngestionHealth>
    for NodeScraperIngestionHealth<'a>
{
    fn from(value: &'a crate::snapshot::self_health::NodeScraperIngestionHealth) -> Self {
        Self {
            last_heard_age_secs: value.last_heard_age,
            scrape_time_secs: value.scrape_time,
            version: value.version.as_deref(),
            git_sha: value.git_sha.as_deref(),
        }
    }
}

#[derive(Debug, Serialize)]
struct NodeNodeScraperHealth<'a> {
    kubelet_stats: NodeScraperIngestionHealth<'a>,
    kubelet_health: NodeScraperIngestionHealth<'a>,
    system_agent: NodeScraperIngestionHealth<'a>,
}

impl<'a> From<&'a crate::snapshot::self_health::NodeNodeScraperHealth>
    for NodeNodeScraperHealth<'a>
{
    fn from(value: &'a crate::snapshot::self_health::NodeNodeScraperHealth) -> Self {
        Self {
            kubelet_stats: NodeScraperIngestionHealth::from(&value.kubelet_stats),
            kubelet_health: NodeScraperIngestionHealth::from(&value.kubelet_health),
            system_agent: NodeScraperIngestionHealth::from(&value.system_agent),
        }
    }
}

#[derive(Debug, Serialize)]
struct ReflectorHealth {
    has_been_initialized: bool,
    #[serde(serialize_with = "duration_to_secs")]
    relist_started_age_secs: Option<Duration>,
    #[serde(serialize_with = "duration_to_secs")]
    relist_completed_age_secs: Option<Duration>,
    #[serde(serialize_with = "duration_to_secs")]
    relist_duration_secs: Option<Duration>,
    #[serde(serialize_with = "duration_to_secs")]
    last_error_age_secs: Option<Duration>,
    errors_total: u64,
}

impl From<&crate::snapshot::self_health::ReflectorHealth> for ReflectorHealth {
    fn from(value: &crate::snapshot::self_health::ReflectorHealth) -> Self {
        Self {
            has_been_initialized: value.has_been_initialized,
            relist_started_age_secs: value.relist_started_age,
            relist_completed_age_secs: value.relist_completed_age,
            relist_duration_secs: value.relist_duration,
            last_error_age_secs: value.last_error_age,
            errors_total: value.errors_total,
        }
    }
}

#[derive(Debug, Serialize)]
struct ClusterAggregatorMetadata<'a> {
    version: &'a str,
    git_sha: Option<&'a str>,
}

impl<'a> ClusterAggregatorMetadata<'a> {
    fn new(version: &'a str, git_sha: Option<&'a str>) -> Self {
        Self { version, git_sha }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct KubeAgentHealthV1<'a> {
    node_scrapers: BTreeMap<&'a str, NodeNodeScraperHealth<'a>>,
    reflector_healths: BTreeMap<&'static str, ReflectorHealth>,
    cluster_aggregator: ClusterAggregatorMetadata<'a>,
}

impl<'a> KubeAgentHealthV1<'a> {
    pub fn from_self_health(self_health: &'a SelfHealth) -> KubeAgentHealthV1<'a> {
        let reflector_healths = self_health
            .reflector_healths
            .iter()
            .map(|(k, v)| (*k, ReflectorHealth::from(v)))
            .collect();
        let node_scrapers = self_health
            .node_node_scrapers
            .iter()
            .map(|(node, health)| (node.as_str(), NodeNodeScraperHealth::from(health)))
            .collect();

        KubeAgentHealthV1 {
            node_scrapers,
            reflector_healths,
            cluster_aggregator: ClusterAggregatorMetadata::new(
                self_health.cluster_aggregator_build_info.version,
                self_health.cluster_aggregator_build_info.git_sha,
            ),
        }
    }
}

impl Section for KubeAgentHealthV1<'_> {
    const NAME: &'static str = "kube_agent_health_v1";
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::snapshot;
    use crate::snapshot::self_health::ClusterAggregatorBuildInfo;
    use crate::test_support::*;

    #[test]
    fn kube_agent_health_v1() {
        let node_node_scrapers = BTreeMap::from([
            (
                s("node01"),
                snapshot::self_health::NodeNodeScraperHealth {
                    kubelet_stats: snapshot::self_health::NodeScraperIngestionHealth {
                        last_heard_age: Some(Duration::from_secs(26)),
                        scrape_time: Some(Duration::from_millis(150)),
                        version: Some(s("1.1000.0")),
                        git_sha: Some(s("39ead942a6da3e42c8b0fb37810eca12344a56c7")),
                    },
                    kubelet_health: snapshot::self_health::NodeScraperIngestionHealth {
                        last_heard_age: Some(Duration::from_secs(24)),
                        scrape_time: Some(Duration::from_millis(45)),
                        version: Some(s("1.1000.0")),
                        git_sha: Some(s("39ead942a6da3e42c8b0fb37810eca12344a56c7")),
                    },
                    system_agent: snapshot::self_health::NodeScraperIngestionHealth {
                        last_heard_age: None,
                        scrape_time: None,
                        version: None,
                        git_sha: None,
                    },
                },
            ),
            (
                s("offline01"),
                snapshot::self_health::NodeNodeScraperHealth::default(),
            ),
        ]);
        let reflector_healths = BTreeMap::from([
            ("Pod", snapshot::self_health::ReflectorHealth::default()),
            (
                "ReplicaSet",
                snapshot::self_health::ReflectorHealth::default(),
            ),
            (
                "DaemonSet",
                snapshot::self_health::ReflectorHealth::default(),
            ),
        ]);
        let cluster_aggregator_build_info = ClusterAggregatorBuildInfo {
            version: "1.1000.1",
            git_sha: Some("40abc123a6da3e42c8b0fb37810eca54321a56a1"),
        };
        let self_health = SelfHealth {
            node_node_scrapers,
            reflector_healths,
            cluster_aggregator_build_info,
        };
        let section = KubeAgentHealthV1::from_self_health(&self_health);
        insta::assert_json_snapshot!(section);
    }
}
