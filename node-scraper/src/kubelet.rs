use std::net::{IpAddr, SocketAddr};

use reqwest::{Certificate, Client, RequestBuilder};

use crate::cli_args::{CliArgs, KubeletServerIdentity};
use crate::error::{Error, Result};

#[derive(Clone)]
pub(crate) struct KubeletClient {
    client: Client,
    base_url: String,
    node_name: String,
}

impl KubeletClient {
    pub(crate) async fn new(args: &CliArgs) -> Result<Self> {
        let node_ip = std::env::var("NODE_IP").map_err(|source| Error::EnvVar {
            name: "NODE_IP".to_string(),
            source,
        })?;
        let node_name = std::env::var("NODE_NAME").map_err(|source| Error::EnvVar {
            name: "NODE_NAME".to_string(),
            source,
        })?;
        let ip = parse_node_ip(&node_ip)?;
        let socket_addr = SocketAddr::new(ip, 10250);
        let mut builder = Client::builder().no_proxy();

        let base_url = match args.kubelet_ca_cert_file.as_deref() {
            None => {
                builder = builder.danger_accept_invalid_certs(true);
                ip_base_url(ip)
            }
            Some(file) => {
                let pem = tokio::fs::read(file).await?;
                builder = builder.tls_certs_only(Certificate::from_pem_bundle(&pem)?);
                match args.kubelet_server_identity {
                    KubeletServerIdentity::NodeIp => ip_base_url(ip),
                    KubeletServerIdentity::NodeName => {
                        builder = builder.resolve(&node_name, socket_addr);
                        format!("https://{node_name}:10250")
                    }
                    KubeletServerIdentity::CaOnly => {
                        builder = builder.tls_danger_accept_invalid_hostnames(true);
                        ip_base_url(ip)
                    }
                }
            }
        };

        Ok(Self {
            client: builder.build()?,
            base_url,
            node_name,
        })
    }

    pub(crate) fn get(&self, path: &str) -> RequestBuilder {
        self.client.get(self.url(path))
    }

    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    pub(crate) fn node_name(&self) -> &str {
        &self.node_name
    }
}

fn parse_node_ip(node_ip: &str) -> Result<IpAddr> {
    let ip = node_ip
        .parse::<IpAddr>()
        .map_err(|source| Error::InvalidNodeIp {
            value: node_ip.to_string(),
            source,
        })?;
    Ok(ip)
}

fn ip_base_url(ip: IpAddr) -> String {
    format!("https://{}", SocketAddr::new(ip, 10250))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_ipv4_and_ipv6_urls() {
        assert_eq!(
            ip_base_url("192.0.2.1".parse().expect("valid IPv4 address")),
            "https://192.0.2.1:10250"
        );
        assert_eq!(
            ip_base_url("2001:db8::1".parse().expect("valid IPv6 address")),
            "https://[2001:db8::1]:10250"
        );
    }
}
