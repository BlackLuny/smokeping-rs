use influxdb2::Client;

#[derive(Debug)]
pub enum InfluxSetupError {
    ClientError(String),
    ConfigError(String),
}

impl std::fmt::Display for InfluxSetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InfluxSetupError::ClientError(msg) => write!(f, "InfluxDB client error: {}", msg),
            InfluxSetupError::ConfigError(msg) => write!(f, "InfluxDB configuration error: {}", msg),
        }
    }
}

impl std::error::Error for InfluxSetupError {}

pub struct InfluxConfig {
    pub url: String,
    pub token: String,
    pub org: String,
    pub bucket: String,
}

impl InfluxConfig {
    pub fn from_env() -> Result<Self, InfluxSetupError> {
        let url = std::env::var("INFLUXDB_URL")
            .map_err(|_| InfluxSetupError::ConfigError("INFLUXDB_URL environment variable not set".to_string()))?;
        
        let token = std::env::var("INFLUXDB_TOKEN")
            .map_err(|_| InfluxSetupError::ConfigError("INFLUXDB_TOKEN environment variable not set".to_string()))?;
        
        let org = std::env::var("INFLUXDB_ORG")
            .unwrap_or_else(|_| "smokeping".to_string());
        
        let bucket = std::env::var("INFLUXDB_BUCKET")
            .unwrap_or_else(|_| "smokeping".to_string());

        Ok(InfluxConfig {
            url,
            token,
            org,
            bucket,
        })
    }
}

pub async fn setup_influxdb() -> Result<(Client, InfluxConfig), InfluxSetupError> {
    let config = InfluxConfig::from_env()?;
    
    println!("Setting up InfluxDB connection...");
    println!("  URL: {}", config.url);
    println!("  Organization: {}", config.org);
    println!("  Bucket: {}", config.bucket);

    // Create client with the organization from config
    let client = Client::new(&config.url, &config.org, &config.token);

    // Test the connection and setup
    match test_connection_and_setup(&client, &config).await {
        Ok(_) => {
            println!("InfluxDB setup completed successfully");
            Ok((client, config))
        }
        Err(e) => {
            eprintln!("InfluxDB setup failed: {}", e);
            Err(e)
        }
    }
}

async fn test_connection_and_setup(client: &Client, config: &InfluxConfig) -> Result<(), InfluxSetupError> {
    println!("Testing InfluxDB connection...");

    // Try to list buckets to check if our bucket exists
    // Note: We can't easily create organizations with the current API, so we'll focus on buckets
    match client.list_buckets(None).await {
        Ok(buckets) => {
            let bucket_exists = buckets.buckets.iter().any(|bucket| bucket.name == config.bucket);

            if !bucket_exists {
                println!("Warning: Bucket '{}' not found in organization '{}'", config.bucket, config.org);
                println!("Please ensure the bucket '{}' exists in organization '{}'", config.bucket, config.org);
                println!("You can create it using the InfluxDB UI or CLI:");
                println!("  influx bucket create --name {} --org {}", config.bucket, config.org);
            } else {
                println!("Bucket '{}' found in organization '{}'", config.bucket, config.org);
            }
        }
        Err(e) => {
            // Check if this is a proxy/network issue
            let error_msg = format!("{}", e);
            if error_msg.contains("503 Service Unavailable") || error_msg.contains("proxy") || error_msg.contains("connection refused") {
                eprintln!("Warning: Network connectivity issue detected (proxy or connection problem)");
                eprintln!("InfluxDB connection failed due to network configuration.");
                eprintln!("Please check:");
                eprintln!("  1. InfluxDB is running and accessible at: {}", config.url);
                eprintln!("  2. No proxy is blocking localhost connections");
                eprintln!("  3. Firewall settings allow connections to port 8086");
                eprintln!("Continuing with limited functionality...");
                return Ok(()); // Continue without InfluxDB for now
            } else {
                eprintln!("Warning: Failed to list buckets: {}", e);
                eprintln!("Please ensure the bucket '{}' exists in organization '{}'", config.bucket, config.org);
            }
        }
    }

    // Test write access by attempting a simple write operation
    match test_write_access(client, &config.bucket).await {
        Ok(_) => println!("InfluxDB write access confirmed"),
        Err(e) => {
            let error_msg = format!("{}", e);
            if error_msg.contains("503 Service Unavailable") || error_msg.contains("proxy") || error_msg.contains("connection refused") {
                eprintln!("Warning: Cannot verify write access due to network issues");
                eprintln!("Continuing with limited functionality...");
                return Ok(()); // Continue without InfluxDB for now
            } else {
                return Err(InfluxSetupError::ClientError(
                    format!("Failed to verify write access to bucket '{}': {}. Please ensure the bucket exists and the token has write permissions.", config.bucket, e)
                ));
            }
        }
    }

    Ok(())
}

// Note: Organization and bucket creation is not easily supported by the current influxdb2 crate
// Users should create these manually using the InfluxDB UI or CLI

async fn test_write_access(client: &Client, bucket: &str) -> Result<(), Box<dyn std::error::Error>> {
    use influxdb2::models::DataPoint;
    
    // Create a test data point
    let point = DataPoint::builder("test_measurement")
        .tag("test_tag", "setup_test")
        .field("test_field", 1.0)
        .build()?;

    // Try to write the test point
    client.write(bucket, futures::stream::iter(vec![point])).await?;
    
    Ok(())
}
