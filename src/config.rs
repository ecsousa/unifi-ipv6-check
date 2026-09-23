#[derive(Clone, Debug)]
pub struct AppConfig {
    pub dns_name: String,
    pub cf_zone_id: String,
    pub cf_apikey: String,
    pub server_base_url: String,
    pub server_username: String,
    pub server_password: String,
    pub server_mac_address: String,
    pub client_base_url: String,
    pub client_username: String,
    pub client_password: String,
    pub client_network_id: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            dns_name: std::env::var("DNS_NAME").expect("DNS_NAME must be set"),
            cf_zone_id: std::env::var("CF_ZONE_ID").expect("CF_ZONE_ID must be set"),
            cf_apikey: std::env::var("CF_APIKEY").expect("CF_APIKEY must be set"),
            server_base_url: std::env::var("SERVER_BASE_URL").expect("SERVER_BASE_URL must be set"),
            server_username: std::env::var("SERVER_USERNAME").expect("SERVER_USERNAME must be set"),
            server_password: std::env::var("SERVER_PASSWORD").expect("SERVER_PASSWORD must be set"),
            server_mac_address: std::env::var("SERVER_MAC_ADDRESS")
                .expect("SERVER_MAC_ADDRESS must be set"),
            client_base_url: std::env::var("CLIENT_BASE_URL").expect("CLIENT_BASE_URL must be set"),
            client_username: std::env::var("CLIENT_USERNAME").expect("CLIENT_USERNAME must be set"),
            client_password: std::env::var("CLIENT_PASSWORD").expect("CLIENT_PASSWORD must be set"),
            client_network_id: std::env::var("CLIENT_NETWORK_ID")
                .expect("CLIENT_NETWORK_ID must be set"),
        }
    }
}
