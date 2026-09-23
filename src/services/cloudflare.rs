use reqwest::Client;
use serde_json::{Value, json};
use tracing::{error, info};

pub async fn update_dns_record(
    client: &Client,
    zone_id: &str,
    apikey: &str,
    dns_name: &str,
    ipv6: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let fetch_url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records?type=AAAA&name={}",
        zone_id,
        urlencoding::encode(dns_name)
    );
    let cf_res = client
        .get(&fetch_url)
        .header("Authorization", format!("Bearer {}", apikey))
        .send()
        .await?;

    let cf_json: Value = cf_res.json().await?;
    let record_id = cf_json
        .get("result")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|r| r.get("id"))
        .and_then(|id| id.as_str());

    let payload = json!({
        "type": "AAAA",
        "name": dns_name,
        "content": ipv6,
        "ttl": 60,
        "proxied": false
    });

    let resp = if let Some(id) = record_id {
        let update_url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            zone_id, id
        );
        client
            .put(&update_url)
            .header("Authorization", format!("Bearer {}", apikey))
            .json(&payload)
            .send()
            .await?
    } else {
        let create_url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        );
        client
            .post(&create_url)
            .header("Authorization", format!("Bearer {}", apikey))
            .json(&payload)
            .send()
            .await?
    };

    if !resp.status().is_success() {
        let err_text = resp.text().await?;
        error!("Updating Cloudflare failed: {}", err_text);
    } else {
        info!("Updating Cloudflare: SUCCESS");
    }

    Ok(())
}
