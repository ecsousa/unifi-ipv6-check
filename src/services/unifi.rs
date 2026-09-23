use crate::models::unifi::{LoginRequest, UnifiEnvelope};
use reqwest::Client;
use serde_json::{Value, json};
use tracing::{debug, error, info};

pub struct UnifiService<'a> {
    client: &'a Client,
    base_url: &'a str,
    csrf_token: String,
    session_cookies: String,
    verbose: bool,
}

impl<'a> UnifiService<'a> {
    pub fn new(client: &'a Client, base_url: &'a str, verbose: bool) -> Self {
        Self {
            client,
            base_url,
            csrf_token: String::new(),
            session_cookies: String::new(),
            verbose,
        }
    }

    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let login_url = format!("{}/api/auth/login", self.base_url);
        let login_req = LoginRequest {
            username,
            password,
            remember_me: false,
            token: "",
        };

        let resp = self.client.post(&login_url).json(&login_req).send().await?;
        if !resp.status().is_success() {
            error!("Unifi login failed to {}: {}", self.base_url, resp.status());
            return Err("Unifi login failed".into());
        }

        self.csrf_token = resp
            .headers()
            .get("X-CSRF-Token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();

        let mut cookie_parts = Vec::new();
        for cookie in resp.cookies() {
            cookie_parts.push(format!("{}={}", cookie.name(), cookie.value()));
        }
        self.session_cookies = cookie_parts.join("; ");

        Ok(())
    }

    pub async fn get_ipv6(&self, mac_address: &str) -> Result<String, Box<dyn std::error::Error>> {
        let stat_url = format!(
            "{}/proxy/network/api/s/default/stat/device/{}",
            self.base_url, mac_address
        );
        let mut req = self.client.get(&stat_url);
        if !self.csrf_token.is_empty() {
            req = req.header("X-CSRF-Token", &self.csrf_token);
        }
        if !self.session_cookies.is_empty() {
            req = req.header(reqwest::header::COOKIE, &self.session_cookies);
        }

        let stat_resp = req.send().await?;
        let stat_text = stat_resp.text().await?;

        if self.verbose {
            debug!("Server json payload read from network: {}", stat_text);
        }

        let env: UnifiEnvelope<Value> = serde_json::from_str(&stat_text)?;
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

    async fn get_network_config(
        &self,
        network_id: &str,
    ) -> Result<(Value, UnifiEnvelope<Value>), Box<dyn std::error::Error>> {
        let netconf_url = format!(
            "{}/proxy/network/api/s/default/rest/networkconf/{}",
            self.base_url, network_id
        );
        let mut req = self.client.get(&netconf_url);
        if !self.csrf_token.is_empty() {
            req = req.header("X-CSRF-Token", &self.csrf_token);
        }
        if !self.session_cookies.is_empty() {
            req = req.header(reqwest::header::COOKIE, &self.session_cookies);
        }

        let netconf_resp = req.send().await?;
        let netconf_text = netconf_resp.text().await?;

        if self.verbose {
            debug!("Client json payload read from network: {}", netconf_text);
        }

        let mut env: UnifiEnvelope<Value> = serde_json::from_str(&netconf_text)?;
        let node = env
            .data
            .first_mut()
            .ok_or("No network config found")?
            .clone();

        Ok((node, env))
    }

    async fn set_network_config(
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
        if !self.csrf_token.is_empty() {
            req = req.header("X-CSRF-Token", &self.csrf_token);
        }
        if !self.session_cookies.is_empty() {
            req = req.header(reqwest::header::COOKIE, &self.session_cookies);
        }

        let update_resp = req.send().await?;
        if !update_resp.status().is_success() {
            error!("Updating VPN client failed: {}", update_resp.status());
            Err("Updating VPN client failed".into())
        } else {
            info!("Updating VPN client: SUCCESS");
            Ok(())
        }
    }

    pub async fn update_vpn_client(
        &self,
        network_id: &str,
        ipv6: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (mut node, _) = self.get_network_config(network_id).await?;

        if let Some(obj) = node.as_object_mut() {
            obj.insert("wireguard_client_peer_ip".to_string(), json!(ipv6));
        }

        self.set_network_config(network_id, &node).await
    }
}
