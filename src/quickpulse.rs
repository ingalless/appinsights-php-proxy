const PING_INTERVAL_SECONDS: u64 = 5;
const POST_INTERVAL_SECONDS: u64 = 1;

use std::{env, fmt::format};

use crate::performance::PerformanceCollector;
use reqwest::{header::HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use sysinfo::{System, SystemExt};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct HeartbeatBody {
    RoleName: String,
    Instance: String,
    InstrumentationKey: String,
    InvariantVersion: String,
    MachineName: String,
    StreamId: String,
    Timestamp: String,
    Version: String,
    Metrics: Vec<Metric>,
    Documents: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct Metric {
    pub Name: String,
    pub Value: String,
    pub Weight: i32,
}

pub struct Client {
    pub connected: bool,
    hostname: String,
    instance: String,
    stream_id: String,
    appinsights_key: String,
    appinsights_live_endpoint: String,
    metrics: Vec<Metric>,
    performance_collector: PerformanceCollector,
}

impl Client {
    pub fn new(appinsights_key: String, appinsights_live_endpoint: String) -> Self {
        let system = System::new();

        let hostname = system
            .host_name()
            .unwrap_or_else(|| String::from("unknown"));

        let instance = env::var("WEBSITE_INSTANCE_ID").unwrap_or_else(|_| hostname.to_owned());

        Client {
            appinsights_key,
            hostname,
            instance,
            appinsights_live_endpoint,
            connected: false,
            metrics: vec![],
            stream_id: Uuid::new_v4().to_string().replace("-", ""),
            performance_collector: PerformanceCollector::new(),
        }
    }

    pub async fn go(&mut self) {
        let mut interval =
            tokio::time::interval(tokio::time::Duration::from_secs(PING_INTERVAL_SECONDS));

        loop {
            interval.tick().await;

            if !self.connected {
                self.ping().await;
            } else {
                self.performance_collector.collect();
                self.post().await;
                interval =
                    tokio::time::interval(tokio::time::Duration::from_secs(POST_INTERVAL_SECONDS));
                interval.reset();
            };
        }
    }

    fn _add_metric(&mut self, metric: Metric) {
        self.metrics.push(metric);
    }

    fn build_url(&mut self, action: String) -> String {
        format!("{}/QuickPulseService.svc/{}?ikey={}", self.appinsights_live_endpoint, action, self.appinsights_key)
    }

    async fn ping(&mut self) {
        let client = reqwest::Client::new();

        let heartbeat_body = HeartbeatBody {
            RoleName: "Web".to_owned(),
            InstrumentationKey: self.appinsights_key.to_owned(),
            InvariantVersion: "1".to_owned(),
            MachineName: self.hostname.to_owned(),
            Instance: self.instance.to_owned(),
            StreamId: self.stream_id.to_owned(),
            Timestamp: format!("/Date({})/", get_millis_timestamp().to_string()),
            Version: "rust:0.0.1".to_owned(),
            Documents: serde_json::Value::Null,
            Metrics: vec![],
        };

        let mut heartbeat_headers = HeaderMap::new();
        heartbeat_headers.insert(
            "x-ms-qps-transmission-time",
            get_transmission_time().to_string().parse().unwrap(),
        );
        heartbeat_headers.insert("x-ms-qps-stream-id", self.stream_id.parse().unwrap());
        heartbeat_headers.insert("x-ms-qps-machine-name", self.hostname.parse().unwrap());
        heartbeat_headers.insert("x-ms-qps-instance-name", self.instance.parse().unwrap());
        heartbeat_headers.insert("x-ms-qps-role-name", "Web".parse().unwrap());
        heartbeat_headers.insert("x-ms-qps-invariant-version", "1".parse().unwrap());
        heartbeat_headers.insert("Expect", "100-continue".parse().unwrap());
        heartbeat_headers.insert("Content-Type", "application/json".parse().unwrap());

        let response = client
                .post(self.build_url("ping".to_string()))
                .headers(heartbeat_headers.clone())
                .body(serde_json::to_string(&heartbeat_body).unwrap())
                .send()
                .await
                .unwrap();

        if !StatusCode::is_success(&response.status()) {
            return;
        }

        self.connected = match response.headers().get("x-ms-qps-subscribed") {
            None => false,
            Some(header) => {
                if header.to_str().unwrap() == "true" {
                    true
                } else {
                    false
                }
            }
        };
    }

    async fn post(&mut self) {
        let client = reqwest::Client::new();

        let metrics: Vec<Metric> = self
            .performance_collector
            .metrics
            .iter()
            .map(|(key, value)| Metric {
                Name: key.as_str().to_string(),
                Value: value.to_string(),
                Weight: 1,
            })
            .collect();

        let heartbeat_body: Vec<HeartbeatBody> = vec![HeartbeatBody {
            RoleName: "Web".to_owned(),
            InstrumentationKey: self.appinsights_key.to_owned(),
            InvariantVersion: "1".to_owned(),
            MachineName: self.hostname.to_owned(),
            Instance: self.instance.to_owned(),
            StreamId: self.stream_id.to_owned(),
            Timestamp: format!("/Date({})/", get_millis_timestamp().to_string()),
            Version: "rust:0.0.1".to_owned(),
            Documents: serde_json::Value::Null,
            Metrics: metrics,
        }];

        let mut heartbeat_headers = HeaderMap::new();
        heartbeat_headers.insert("Expect", "100-continue".parse().unwrap());
        heartbeat_headers.insert(
            "x-ms-qps-transmission-time",
            get_transmission_time().to_string().parse().unwrap(),
        );

        let response = client
                .post(self.build_url("post".to_string()))
                .headers(heartbeat_headers.clone())
                .body(serde_json::to_string(&heartbeat_body).unwrap())
                .send()
                .await
                .unwrap();

        if !StatusCode::is_success(&response.status()) {
            return;
        }

        self.connected = match response.headers().get("x-ms-qps-subscribed") {
            None => false,
            Some(header) => {
                if header.to_str().unwrap() == "true" {
                    true
                } else {
                    false
                }
            }
        };
    }
}

fn get_transmission_time() -> i64 {
    (chrono::Utc::now().timestamp() + 62135596800000) * 10000
}

fn get_millis_timestamp() -> i64 {
    chrono::Utc::now().timestamp_millis()
}
