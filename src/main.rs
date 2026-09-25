mod config;
mod models;
mod services;

use clap::Parser;
use config::AppConfig;
use reqwest::Client;
use std::time::Duration;
use serde_json::json;
use tracing::{debug, error, info};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    once: bool,
    #[arg(short, long)]
    verbose: bool,
}

async fn check_dns(args: &Args, config: &AppConfig, server_address: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Resolve DNS_NAME hostname for type AAAA
    let resolved_ipv6 = services::dns::resolve_ipv6(&config.dns_name).await?;

    if args.once {
        info!("Resolved IPV6: {:?}", resolved_ipv6);
    } else if args.verbose {
        debug!("Resolved IPV6: {:?}", resolved_ipv6);
    }

    // 4. Compare IPv6
    if resolved_ipv6.as_deref() == Some(server_address) {
        if args.verbose {
            debug!("DNS name matches, no update needed");
        }
        return Ok(());
    }

    info!(
        "Addresses don't match (resolved: {:?}, server: {}). Updating...",
        resolved_ipv6, server_address
    );

    // 7. Call Cloudflare
    let cf_client = Client::builder().timeout(Duration::from_secs(10)).build()?;
    services::cloudflare::update_dns_record(
        &cf_client,
        &config.cf_zone_id,
        &config.cf_apikey,
        &config.dns_name,
        server_address,
    )
        .await?;

    Ok(())

}

async fn check_wireguard(args: &Args, config: &AppConfig, server_address: &str) -> Result<(), Box<dyn std::error::Error>> {

    let client_http = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let unifi =
        services::unifi::UnifiService::new(
            &client_http, &config.client_base_url, &config.client_apikey, args.verbose);

    let (mut node, _) = unifi.get_network_config(&config.client_network_id).await?;

    let resolved_ipv6 = node.get("wireguard_client_peer_ip")
        .and_then(|v| v.as_str());

    if resolved_ipv6 == Some(server_address) {
        if args.verbose {
            debug!("Wireguard server matches, no update needed");
        }
        return Ok(());
    }

    info!(
        "Addresses don't match (resolved: {:?}, server: {}). Updating...",
        resolved_ipv6, server_address
    );

    if let Some(obj) = node.as_object_mut() {
        obj.insert("wireguard_client_peer_ip".to_string(), json!(server_address));
    }

    unifi.set_network_config(&config.client_network_id, &node).await


}

async fn run_loop(args: &Args, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {

    let http_client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let server_unifi = services::unifi::UnifiService::new(
        &http_client, &config.server_base_url, &config.server_apikey, args.verbose);

    // Get server IPv6 address
    let server_address = server_unifi.get_ipv6(&config.server_mac_address).await?;

    if args.once {
        info!("IPv6 read from server router: {}", server_address);
    } else if args.verbose {
        debug!("IPv6 read from server router: {}", server_address);
    }

    if let Err(e) = check_dns(args, config, &server_address).await {
        error!("Error running check: {}", e);
    }

    if let Err(e) = check_wireguard(args, config, &server_address).await {
        error!("Error running check: {}", e);
    }

    Ok(())

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let env_filter = if args.verbose { "debug" } else { "info" };

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    // Read environment variables and validation once
    let config = AppConfig::from_env();

    if args.once {
        if let Err(e) = run_loop(&args, &config).await {
            error!("Error running check: {}", e);
        }
        return Ok(());
    }

    loop {
        if let Err(e) = run_loop(&args, &config).await {
            error!("Error running check: {}", e);
        }
        tokio::time::sleep(Duration::from_secs(5 * 60)).await;
    }
}
