use hickory_resolver::TokioAsyncResolver;
use tracing::debug;

pub async fn resolve_ipv6(dns_name: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let resolver = TokioAsyncResolver::tokio_from_system_conf()?;
    let response = resolver.ipv6_lookup(dns_name).await;
    match response {
        Ok(res) => Ok(res.iter().next().map(|ip| ip.to_string())),
        Err(e) => {
            debug!("DNS resolution failed: {:?}", e);
            Ok(None)
        }
    }
}
