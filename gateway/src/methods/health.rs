use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub ok: bool,
    pub uptime_seconds: u64,
    pub version: &'static str,
    pub channel_statuses: Vec<ChannelStatus>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelStatus {
    pub platform: String,
    pub healthy: bool,
    pub details: String,
}

pub fn build_health_response(
    uptime_seconds: u64,
    version: &'static str,
    channel_statuses: Vec<ChannelStatus>,
) -> HealthResponse {
    HealthResponse {
        ok: channel_statuses.iter().all(|row| row.healthy),
        uptime_seconds,
        version,
        channel_statuses,
    }
}
