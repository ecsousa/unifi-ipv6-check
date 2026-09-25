use crate::models::unifi::UnifiEnvelope;
use reqwest::Client;
use serde_json::Value;
use tracing::{debug, error, info};

pub struct UnifiService<'a> {
    client: &'a Client,
    base_url: &'a str,
    apikey: &'a str,
    verbose: bool,
}

impl<'a> UnifiService<'a> {
    pub fn new(client: &'a Client, base_url: &'a str, apikey: &'a str, verbose: bool) -> Self {
        Self {
            client,
            base_url,
            apikey,
            verbose,
        }
    }

    pub async fn get_ipv6(&self, mac_address: &str) -> Result<String, Box<dyn std::error::Error>> {
        let stat_url = format!(
            "{}/proxy/network/api/s/default/stat/device/{}",
            self.base_url, mac_address
        );
        let mut req = self.client.get(&stat_url);
        req = req.header("X-API-KEY", self.apikey);

        let stat_resp = req.send().await?;
        let stat_status = stat_resp.status();
        let stat_text = stat_resp.text().await?;

        if !stat_status.is_success() {
            return Err(format!("Unifi API error in get_ipv6: {} - {}", stat_status, stat_text).into());
        }

        if self.verbose {
            debug!("Server json payload read from network: {}", stat_text);
        }

        let env: UnifiEnvelope<Value> = serde_json::from_str(&stat_text).map_err(|e| {
            format!(
                "Failed to parse Unifi stats response: {}. Raw payload: {}",
                e, stat_text
            )
        })?;
        let mut read_ipv6 = None;
        if let Some(device) = env.data.first() {
            if let Some(ipv6_arr) = device
                .get("wan1")
                .and_then(|w| w.get("ipv6"))
                .and_then(|v| v.as_array())
            {
                for ip_val in ipv6_arr {
                    if let Some(ip_str) = ip_val.as_str() {
                        if !ip_str.starts_with("fe80:") {
                            let ip_clean = ip_str.split('/').next().unwrap_or(ip_str);
                            read_ipv6 = Some(ip_clean.to_string());
                            break;
                        }
                    }
                }
            }
        }

        read_ipv6.ok_or_else(|| {
            error!("Could not find a valid non-link-local IPv6 address in router stats");
            "No valid IPv6 found".into()
        })
    }

    pub async fn get_network_config(
        &self,
        network_id: &str,
    ) -> Result<(Value, UnifiEnvelope<Value>), Box<dyn std::error::Error>> {
        let netconf_url = format!(
            "{}/proxy/network/api/s/default/rest/networkconf/{}",
            self.base_url, network_id
        );
        let mut req = self.client.get(&netconf_url);
        req = req.header("X-API-KEY", self.apikey);

        let netconf_resp = req.send().await?;
        let netconf_status = netconf_resp.status();
        let netconf_text = netconf_resp.text().await?;

        if !netconf_status.is_success() {
            return Err(format!("Unifi API error in get_network_config: {} - {}", netconf_status, netconf_text).into());
        }

        if self.verbose {
            debug!("Client json payload read from network: {}", netconf_text);
        }

        let mut env: UnifiEnvelope<Value> = serde_json::from_str(&netconf_text).map_err(|e| {
            format!(
                "Failed to parse Unifi network config response: {}. Raw payload: {}",
                e, netconf_text
            )
        })?;
        let node = env
            .data
            .first_mut()
            .ok_or("No network config found")?
            .clone();

        Ok((node, env))
    }

    pub async fn set_network_config(
        &self,
        network_id: &str,
        payload: &Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let netconf_url = format!(
            "{}/proxy/network/api/s/default/rest/networkconf/{}",
            self.base_url, network_id
        );

        if self.verbose {
            debug!(
                "Client json payload set to network: {}",
                serde_json::to_string(payload)?
            );
        }

        let mut req = self.client.put(&netconf_url).json(payload);
        req = req.header("X-API-KEY", self.apikey);

        let update_resp = req.send().await?;
        let update_status = update_resp.status();

        if !update_status.is_success() {
            let err_text = update_resp.text().await.unwrap_or_default();
            let err_msg = format!("Updating VPN client failed: {} - {}", update_status, err_text);
            error!("{}", err_msg);
            Err(err_msg.into())
        } else {
            info!("Updating VPN client: SUCCESS");
            Ok(())
        }
    }

}
