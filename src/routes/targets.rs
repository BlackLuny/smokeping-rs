use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{ActiveModelTrait, EntityTrait, NotSet, Set};
use serde::{Deserialize, Serialize};
use crate::models::target;
use crate::AppState;
use influxdb2::FromMap;
use influxdb2::models::Query as InfluxQuery;

#[derive(Deserialize, Debug)]
pub struct TargetInput {
    pub name: String,
    pub host: String,
    #[serde(default = "default_probe_type")]
    pub probe_type: String,
    pub probe_interval_secs: i32,
    pub is_active: bool,
}

fn default_probe_type() -> String {
    "icmp".to_string()
}

#[derive(Deserialize)]
pub struct ProbeDataQuery {
    pub start_time: String,
    pub end_time: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct InfluxProbeDataPoint {
    pub target_id: String,
    pub is_lost: String,
    pub rtt_ms: f64,
    pub _time: String,
}

impl FromMap for InfluxProbeDataPoint {
    fn from_genericmap(map: std::collections::BTreeMap<String, influxdb2_structmap::value::Value>) -> Self {
        use influxdb2_structmap::value::Value;

        // Helper function to safely extract string values
        let get_string = |key: &str| -> String {
            map.get(key).and_then(|v| match v {
                Value::String(s) => Some(s.as_str()),
                _ => None,
            }).unwrap_or("").to_string()
        };

        // Helper function to safely extract numeric values
        let get_numeric = |key: &str| -> f64 {
            map.get(key).and_then(|v| match v {
                Value::Double(d) => Some(d.into_inner()),
                Value::Long(l) => Some(*l as f64),
                Value::UnsignedLong(ul) => Some(*ul as f64),
                _ => None,
            }).unwrap_or(0.0)
        };

        InfluxProbeDataPoint {
            target_id: get_string("target_id"),
            is_lost: get_string("is_lost"),
            rtt_ms: get_numeric("rtt_ms"),
            _time: get_string("_time"),
        }
    }
}

#[derive(Serialize)]
pub struct ProbeDataPoint {
    pub time: String,
    pub rtt_ms: f64,
    pub is_lost: bool,
}

// Handler to list all targets
pub async fn list_targets(State(state): State<AppState>) -> impl IntoResponse {
    match target::Entity::find().all(state.db.as_ref()).await {
        Ok(targets) => Json(targets).into_response(),
        Err(e) => {
            eprintln!("Database error listing targets: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Database error",
                "message": "Unable to retrieve targets"
            }))).into_response()
        }
    }
}

// Handler to get a single target by ID
pub async fn get_target(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    println!("Received get target ID: {}", id);
    match target::Entity::find_by_id(id).one(state.db.as_ref()).await {
        Ok(Some(target)) => Json(target).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Target not found").into_response(),
        Err(e) => {
            eprintln!("Database error getting target {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Database error",
                "message": "Unable to retrieve target"
            }))).into_response()
        }
    }
}

// Handler to create a new target
pub async fn create_target(
    State(state): State<AppState>,
    Json(input): Json<TargetInput>,
) -> impl IntoResponse {
    println!("Received target input: {:?}", input);
    let new_target = target::ActiveModel {
        id: NotSet,
        name: Set(input.name.to_owned()),
        host: Set(input.host.to_owned()),
        probe_type: Set(input.probe_type.to_owned()),
        probe_interval_secs: Set(input.probe_interval_secs),
        is_active: Set(input.is_active),
        created_at: Set(chrono::Utc::now()),
    };
    match new_target.insert(state.db.as_ref()).await {
        Ok(result) => (StatusCode::CREATED, Json(result)).into_response(),
        Err(e) => {
            eprintln!("Database error creating target: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Database error",
                "message": "Unable to create target"
            }))).into_response()
        }
    }
}

// Handler to update an existing target
pub async fn update_target(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<TargetInput>,
) -> impl IntoResponse {
    let mut target: target::ActiveModel = match target::Entity::find_by_id(id).one(state.db.as_ref()).await {
        Ok(Some(target)) => target.into(),
        Ok(None) => return (StatusCode::NOT_FOUND, "Target not found").into_response(),
        Err(e) => {
            eprintln!("Database error finding target {}: {}", id, e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Database error",
                "message": "Unable to find target for update"
            }))).into_response();
        }
    };

    target.name = Set(input.name.to_owned());
    target.host = Set(input.host.to_owned());
    target.probe_type = Set(input.probe_type.to_owned());
    target.probe_interval_secs = Set(input.probe_interval_secs);
    target.is_active = Set(input.is_active);

    match target.update(state.db.as_ref()).await {
        Ok(result) => Json(result).into_response(),
        Err(e) => {
            eprintln!("Database error updating target {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Database error",
                "message": "Unable to update target"
            }))).into_response()
        }
    }
}

// Handler to delete a target
pub async fn delete_target(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match target::Entity::delete_by_id(id).exec(state.db.as_ref()).await {
        Ok(result) => {
            if result.rows_affected == 1 {
                (StatusCode::NO_CONTENT, "").into_response()
            } else {
                (StatusCode::NOT_FOUND, "Target not found").into_response()
            }
        }
        Err(e) => {
            eprintln!("Database error deleting target {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Database error",
                "message": "Unable to delete target"
            }))).into_response()
        }
    }
}

pub async fn get_probe_data(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Query(query): Query<ProbeDataQuery>,
) -> impl IntoResponse {
    let flux_query = format!(
        "from(bucket: \"{}\")
        |> range(start: {}, stop: {})
        |> filter(fn: (r) => r._measurement == \"probe_data\")
        |> filter(fn: (r) => r.target_id == \"{}\")
        |> filter(fn: (r) => r._field == \"rtt_ms\")
        |> keep(columns: [\"_time\", \"_value\", \"target_id\", \"is_lost\"])
        |> rename(columns: {{\"_value\": \"rtt_ms\"}})",
        state.influx_config.bucket,
        query.start_time,
        query.end_time,
        id
    );

    // Handle InfluxDB query errors gracefully
    let result: Vec<InfluxProbeDataPoint> = match state.influx_client.query(Some(InfluxQuery::new(flux_query))).await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("InfluxDB query failed: {}", e);
            // Return empty data set instead of panicking
            return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "error": "InfluxDB service unavailable",
                "message": "Unable to retrieve probe data at this time"
            }))).into_response();
        }
    };

    let data_points: Vec<ProbeDataPoint> = result.into_iter().map(|p| {
        ProbeDataPoint {
            time: p._time,
            rtt_ms: p.rtt_ms,
            is_lost: p.is_lost.parse().unwrap_or(false),
        }
    }).collect();

    Json(data_points).into_response()
}