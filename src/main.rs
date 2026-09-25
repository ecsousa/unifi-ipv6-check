mod config;
mod models;
mod services;

use clap::Parser;
use config::AppConfig;
use reqwest::Client;
use std::time::Duration;
use tracing::{debug, error, info};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    once: bool,
    #[arg(short, long)]
    verbose: bool,
}

async fn run_loop(args: &Args, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Resolve DNS_NAME hostname for type AAAA
    let resolved_ipv6 = services::dns::resolve_ipv6(&config.dns_name).await?;

    if args.once {
        info!("Resolved IPV6: {:?}", resolved_ipv6);
    } else if args.verbose {
        debug!("Resolved IPV6: {:?}", resolved_ipv6);
    }

    let http_client = Client::builder()
        .cookie_store(true)
        .timeout(Duration::from_secs(10))
        .build()?;

    // 2. Login to unifi router (SERVER)
    let server_unifi = services::unifi::UnifiService::new(
        &http_client, &config.server_base_url, &config.server_apikey, args.verbose);

    // 3. Make GET request to server router & extract IPv6
    let read_ipv6 = server_unifi.get_ipv6(&config.server_mac_address).await?;

    if args.once {
        info!("IPv6 read from server router: {}", read_ipv6);
    } else if args.verbose {
        debug!("IPv6 read from server router: {}", read_ipv6);
    }

    // 4. Compare IPv6
    if resolved_ipv6.as_deref() == Some(&read_ipv6) {
        if args.verbose {
            debug!("Addresses match, no update needed.");
        }
        return Ok(());
    }

    info!(
        "Addresses don't match (resolved: {:?}, server: {}). Updating...",
        resolved_ipv6, read_ipv6
    );

    // 5. Login to unifi router (CLIENT)
    let client_http = Client::builder()
        .cookie_store(true)
        .timeout(Duration::from_secs(10))
        .build()?;

    let client_unifi =
        services::unifi::UnifiService::new(
            &client_http, &config.client_base_url, &config.client_apikey, args.verbose);

    // 6. Update network configuration
    client_unifi
        .update_vpn_client(&config.client_network_id, &read_ipv6)
        .await?;

    // 7. Call Cloudflare
    let cf_client = Client::builder().timeout(Duration::from_secs(10)).build()?;
    services::cloudflare::update_dns_record(
        &cf_client,
        &config.cf_zone_id,
        &config.cf_apikey,
        &config.dns_name,
        &read_ipv6,
    )
    .await?;

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
