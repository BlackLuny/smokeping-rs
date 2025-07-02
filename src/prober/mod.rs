use crate::models::target::Model as Target;
use influxdb2::Client;
use influxdb2::models::DataPoint;
use serde_json::json;
use std::time::Duration;
use surge_ping::{Client as PingClient, Config, PingIdentifier, PingSequence};
use tokio::sync::broadcast;
use tokio::time;
use std::net::IpAddr;

pub async fn run_prober(target: Target, client: Client, bucket: String, tx: broadcast::Sender<String>) {
    let mut interval = time::interval(Duration::from_secs(target.probe_interval_secs as u64));

    // Create ping client
    let config = Config::default();
    let ping_client = match PingClient::new(&config) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to create ping client for target {}: {}", target.id, e);
            return;
        }
    };

    // Parse host IP once and handle error
    let host_ip: IpAddr = match target.host.parse() {
        Ok(ip) => ip,
        Err(e) => {
            eprintln!("Invalid IP address '{}' for target {}: {}", target.host, target.id, e);
            return;
        }
    };

    loop {
        interval.tick().await;

        // Create pinger and perform ping
        let mut pinger = ping_client.pinger(host_ip, PingIdentifier(0)).await;
        let result = pinger.ping(PingSequence(0), &[0u8; 64]).await;

        let (is_lost, rtt) = match result {
            Ok((_, duration)) => (false, duration.as_millis() as f64),
            Err(_) => (true, 0.0),
        };

        let point = match DataPoint::builder("probe_data")
            .tag("target_id", target.id.to_string())
            .tag("is_lost", is_lost.to_string())
            .field("rtt_ms", rtt)
            .build()
        {
            Ok(point) => point,
            Err(e) => {
                eprintln!("Failed to build data point for target {}: {}", target.id, e);
                continue;
            }
        };

        if let Err(e) = client.write(&bucket, futures::stream::iter(vec![point])).await {
            eprintln!("Failed to write to InfluxDB: {}", e);
        }

        let ws_msg = json!({ "target_id": target.id, "is_lost": is_lost, "rtt_ms": rtt }).to_string();
        if let Err(_) = tx.send(ws_msg) {
            // eprintln!("Failed to send WebSocket message: {}", e);
        }
    }
}
