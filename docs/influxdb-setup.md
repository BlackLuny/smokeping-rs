# InfluxDB Configuration Guide

This guide explains how to configure InfluxDB for smokeping-rs and troubleshoot common issues.

## Overview

Smokeping-rs uses InfluxDB v2.x to store time-series probe data. The application automatically detects and validates the InfluxDB configuration during startup.

## Environment Variables

The following environment variables control InfluxDB connectivity:

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `INFLUXDB_URL` | InfluxDB server URL | - | Yes |
| `INFLUXDB_TOKEN` | Authentication token | - | Yes |
| `INFLUXDB_ORG` | Organization name | `smokeping` | No |
| `INFLUXDB_BUCKET` | Bucket name for data storage | `smokeping` | No |

## Docker Compose Setup

The provided `docker-compose.yml` automatically sets up InfluxDB with the correct configuration:

```yaml
influxdb:
  image: influxdb:2.7
  environment:
    - DOCKER_INFLUXDB_INIT_MODE=setup
    - DOCKER_INFLUXDB_INIT_USERNAME=my-user
    - DOCKER_INFLUXDB_INIT_PASSWORD=my-super-secret-password
    - DOCKER_INFLUXDB_INIT_ORG=my-org
    - DOCKER_INFLUXDB_INIT_BUCKET=smokeping
    - DOCKER_INFLUXDB_INIT_ADMIN_TOKEN=my-super-secret-auth-token
```

## Manual InfluxDB Setup

If you're using an existing InfluxDB instance, ensure the following:

1. **Organization exists**: The organization specified in `INFLUXDB_ORG` must exist
2. **Bucket exists**: The bucket specified in `INFLUXDB_BUCKET` must exist in the organization
3. **Token permissions**: The token must have read/write access to the bucket

### Creating Organization and Bucket

Using InfluxDB CLI:
```bash
# Create organization
influx org create --name my-org

# Create bucket
influx bucket create --name smokeping --org my-org

# Create token with read/write access
influx auth create --org my-org --read-buckets --write-buckets
```

Using InfluxDB UI:
1. Navigate to `http://localhost:8086`
2. Go to "Organizations" and create a new organization
3. Go to "Buckets" and create a new bucket
4. Go to "API Tokens" and create a token with bucket read/write permissions

## Troubleshooting

### Common Error: "organization not found"

**Error message:**
```
Failed to write to InfluxDB: HTTP request returned an error: 404 Not Found, 
`{"code":"not found","message":"organization name \"smokeping\" not found"}`
```

**Solution:**
1. Check that the organization exists in InfluxDB
2. Verify the `INFLUXDB_ORG` environment variable matches the actual organization name
3. Ensure the token has access to the specified organization

### Common Error: "bucket not found"

**Error message:**
```
Failed to write to InfluxDB: HTTP request returned an error: 404 Not Found,
`{"code":"not found","message":"bucket \"smokeping\" not found"}`
```

**Solution:**
1. Check that the bucket exists in the specified organization
2. Verify the `INFLUXDB_BUCKET` environment variable matches the actual bucket name
3. Ensure the token has read/write access to the bucket

### Network Connectivity Issues

**Error message:**
```
Warning: Network connectivity issue detected (proxy or connection problem)
```

**Solution:**
1. Verify InfluxDB is running and accessible at the specified URL
2. Check if a proxy is blocking localhost connections
3. Verify firewall settings allow connections to the InfluxDB port (default: 8086)
4. For Docker setups, ensure the backend service can reach the influxdb service

### Token Permission Issues

**Error message:**
```
Failed to verify write access: insufficient permissions
```

**Solution:**
1. Verify the token has write permissions to the specified bucket
2. Check that the token hasn't expired
3. Ensure the token was created for the correct organization

## Application Startup Behavior

During startup, smokeping-rs performs the following InfluxDB checks:

1. **Connection Test**: Attempts to connect to InfluxDB
2. **Bucket Verification**: Lists buckets to verify the configured bucket exists
3. **Write Test**: Performs a test write to verify permissions
4. **Graceful Degradation**: If InfluxDB is unavailable, the application continues with limited functionality

## Data Schema

Smokeping-rs stores probe data in InfluxDB with the following schema:

**Measurement**: `probe_data`

**Tags**:
- `target_id`: Unique identifier for the monitored target
- `is_lost`: Boolean indicating if the probe was lost (true/false)

**Fields**:
- `rtt_ms`: Round-trip time in milliseconds (float)

**Example data point**:
```
probe_data,target_id=1,is_lost=false rtt_ms=23.45 1640995200000000000
```

## Performance Considerations

- **Retention Policy**: Configure appropriate retention policies for your use case
- **Bucket Sharding**: For high-volume deployments, consider using multiple buckets
- **Indexing**: InfluxDB automatically indexes tags for efficient queries
- **Batch Writes**: The application batches probe results for optimal write performance
