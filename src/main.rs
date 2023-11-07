use dotenvy::dotenv;
use reqwest::{self, header::HeaderMap};
use serde::{Deserialize, Serialize};
use std::{env, io};
use tokio::time;

const INSTRUMENTATION_KEY: &str = "APPINSIGHTS_INSTRUMENTATIONKEY";

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct HeartbeatBody {
    RoleName: String,
    InstrumentationKey: String,
    InvariantVersion: String,
    MachineName: String,
    StreamId: String,
    Timestamp: String,
    Version: String,
    Metrics: Vec<String>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv().expect(".env file not found");

    let app_insights_key: String =
        env::var(INSTRUMENTATION_KEY).expect("Var APPINSIGHTS_INSTRUMENTATIONKEY not set");
    let appinsights = appinsights::TelemetryClient::new(app_insights_key.to_string());

    tokio::spawn(async {
        quickpulse_client().await;
    });

    loop {}
}

async fn quickpulse_client() {
    let mut is_collecting = false;
    let mut interval = time::interval(time::Duration::from_secs(10));
    let _metrics: Vec<String> = vec![];
    let _documents: Vec<String> = vec![];

    loop {
        interval.tick().await;

        if !is_collecting {
            ping().await;
            is_collecting = true;
        } else {
            post().await;
        }
    }
}

async fn post() {
    println!("Would have posted, but I don't know how");
}

async fn ping() {
    let client = reqwest::Client::new();

    let now = chrono::Utc::now().timestamp().to_string();

    let app_insights_key = env::var(INSTRUMENTATION_KEY).unwrap();

    let heartbeat_body = HeartbeatBody {
        RoleName: "app-teams-event-proxy".to_owned(),
        InstrumentationKey: app_insights_key.to_owned(),
        InvariantVersion: "1".to_owned(),
        MachineName: "dw0sdwk0005FD".to_owned(),
        StreamId: "ff6f7facd28144de877018b12b4c382c".to_owned(),
        Timestamp: now,
        Version: "rust:0.0.1".to_owned(),
        Metrics: vec![r"\ApplicationInsights\Exceptions/Sec".to_owned()],
    };

    let mut heartbeat_headers = HeaderMap::new();
    heartbeat_headers.insert(
        "x-ms-qps-stream-id",
        "ff6f7facd28144de877018b12b4c382c".parse().unwrap(),
    );
    heartbeat_headers.insert("x-ms-qps-machine-name", "dw0sdwk0005FD".parse().unwrap());
    heartbeat_headers.insert(
        "x-ms-qps-role-name",
        "app-teams-event-proxy".parse().unwrap(),
    );
    heartbeat_headers.insert("x-ms-qps-invariant-version", "1".parse().unwrap());
    heartbeat_headers.insert("Expect", "100-continue".parse().unwrap());

    client
        .post(format!(
            "https://uksouth.livediagnostics.monitor.azure.com/QuickPulseService.svc/ping?ikey={}",
            app_insights_key
        ))
        .headers(heartbeat_headers.clone())
        .body(serde_json::to_string(&heartbeat_body).unwrap())
        .send()
        .await
        .unwrap();
}

// async fn process(client: &TelemetryClient, msg: SyslogMessage) {
//     client.track_trace(msg.msg, map_severity_to_app_insights(msg.severity));
// }

// fn map_severity_to_app_insights(severity: SyslogSeverity) -> SeverityLevel {
//     match severity {
//         SyslogSeverity::SEV_EMERG => SeverityLevel::Critical,
//         SyslogSeverity::SEV_CRIT => SeverityLevel::Critical,
//         SyslogSeverity::SEV_WARNING => SeverityLevel::Warning,
//         SyslogSeverity::SEV_ALERT => SeverityLevel::Warning,
//         SyslogSeverity::SEV_ERR => SeverityLevel::Error,
//         SyslogSeverity::SEV_NOTICE => SeverityLevel::Information,
//         SyslogSeverity::SEV_INFO => SeverityLevel::Information,
//         SyslogSeverity::SEV_DEBUG => SeverityLevel::Verbose,
//     }
// }
