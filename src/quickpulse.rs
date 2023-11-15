const PING_INTERVAL_SECONDS: u64 = 5;
const POST_INTERVAL_SECONDS: u64 = 1;

use std::env;

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

trait Collector {
    fn collect(&mut self);
}

struct AppCpuUsage {
    user: i32,
    system: i32,
}

struct CpuUsage {
    model: String,
    speed: i32,
    times: CpuTime,
}

struct CpuTime {
    user: i32,
    nice: i32,
    sys: i32,
    idle: i32,
    irq: i32,
}

struct ProcessedDocumentData {
    count: i32,
    failed: i32,
    time: f32,
}

struct ProcessedExceptionData {
    count: i32,
    time: f32,
}

struct PerformanceCollector {
    enabled: bool,
    system: System,
    last_app_cpu_usage: Option<AppCpuUsage>,
    last_hr_time: Vec<i32>,
    last_cpus: Vec<CpuUsage>,
    last_dependencies: Option<ProcessedDocumentData>,
    last_requests: Option<ProcessedDocumentData>,
    last_exceptions: Option<ProcessedExceptionData>,
}

impl Collector for PerformanceCollector {
    fn collect(&mut self) {
        self.enabled = true;

        self.system.refresh_all();
    }
}

impl PerformanceCollector {}

pub struct Client {
    pub connected: bool,
    hostname: String,
    instance: String,
    stream_id: String,
    appinsights_key: String,
    metrics: Vec<Metric>,
    performance_collector: PerformanceCollector,
}

/*
[
  {
    "Documents": null,
    "InstrumentationKey": "c959f435-704c-41eb-a6e0-56c88fbbc774",
    "Metrics": [
      {
        "Name": "\\\\Processor(_Total)\\\\% Processor Time",
        "Value": 2.1067073090143245,
        "Weight": 4
      },
      {
        "Name": "\\\\Memory\\\\Committed Bytes",
        "Value": 7916893184,
        "Weight": 4
      },
      {
        "Name": "\\\\ApplicationInsights\\\\Requests/Sec",
        "Value": 0,
        "Weight": 4
      },
      {
        "Name": "\\\\ApplicationInsights\\\\Requests Failed/Sec",
        "Value": 0,
        "Weight": 4
      },
      {
        "Name": "\\\\ApplicationInsights\\\\Dependency Calls/Sec",
        "Value": 0,
        "Weight": 4
      },
      {
        "Name": "\\\\ApplicationInsights\\\\Dependency Calls Failed/Sec",
        "Value": 0,
        "Weight": 4
      },
      {
        "Name": "\\\\ApplicationInsights\\\\Exceptions/Sec",
        "Value": 0,
        "Weight": 4
      }
    ],
    "InvariantVersion": 1,
    "Timestamp": "/Date(1700062129429)/",
    "Version": "node:2.9.0",
    "StreamId": "2f59e890b9ce4546915881454d62c7f7",
    "MachineName": "ingalless-Latitude-5401",
    "Instance": "ingalless-Latitude-5401",
    "RoleName": "Web"
  }
]
*/

impl Client {
    pub fn new(appinsights_key: String) -> Self {
        let system = System::new();

        let hostname = system
            .host_name()
            .unwrap_or_else(|| String::from("unknown"));

        let instance = env::var("WEBSITE_INSTANCE_ID").unwrap_or_else(|_| hostname.to_owned());

        Client {
            appinsights_key,
            hostname,
            instance,
            connected: false,
            metrics: vec![],
            stream_id: Uuid::new_v4().to_string().replace("-", ""),
            performance_collector: PerformanceCollector {
                enabled: false,
                system: System::new_all(),
                last_cpus: vec![],
                last_hr_time: vec![],
                last_app_cpu_usage: None,
                last_requests: None,
                last_exceptions: None,
                last_dependencies: None,
            },
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
                interval = tokio::time::interval(
                    tokio::time::Duration::from_secs(POST_INTERVAL_SECONDS)
                );
                interval.reset();
            };
        }
    }

    fn _add_metric(&mut self, metric: Metric) {
        self.metrics.push(metric);
    }

    async fn ping(&mut self) {
        let client = reqwest::Client::new();

        let now = chrono::Utc::now().timestamp_millis().to_string();

        let heartbeat_body = HeartbeatBody {
            RoleName: "Web".to_owned(),
            InstrumentationKey: self.appinsights_key.to_owned(),
            InvariantVersion: "1".to_owned(),
            MachineName: self.hostname.to_owned(),
            Instance: self.instance.to_owned(),
            StreamId: self.stream_id.to_owned(),
            Timestamp: format!("/Date({})/", now),
            Version: "rust:0.0.1".to_owned(),
            Documents: serde_json::Value::Null,
            Metrics: self.metrics.to_owned(),
        };

        let transmission_time = (chrono::Utc::now().timestamp() + 62135596800000) * 10000;

        let mut heartbeat_headers = HeaderMap::new();
        heartbeat_headers.insert(
            "x-ms-qps-transmission-time",
            transmission_time.to_string().parse().unwrap(),
        );
        heartbeat_headers.insert("x-ms-qps-stream-id", self.stream_id.parse().unwrap());
        heartbeat_headers.insert("x-ms-qps-machine-name", self.hostname.parse().unwrap());
        heartbeat_headers.insert("x-ms-qps-instance-name", self.instance.parse().unwrap());
        heartbeat_headers.insert("x-ms-qps-role-name", "Web".parse().unwrap());
        heartbeat_headers.insert("x-ms-qps-invariant-version", "1".parse().unwrap());
        heartbeat_headers.insert("Expect", "100-continue".parse().unwrap());
        heartbeat_headers.insert("Content-Type", "application/json".parse().unwrap());

        let response = client
                .post(format!(
                    "https://uksouth.livediagnostics.monitor.azure.com/QuickPulseService.svc/ping?ikey={}",
                    self.appinsights_key
                ))
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

        println!(
            "Request {:?} {:?}",
            heartbeat_headers,
            serde_json::to_string(&heartbeat_body).unwrap()
        );
        println!(
            "Response {} {:?} {:?}",
            response.status(),
            response.headers().to_owned(),
            response.text().await.unwrap()
        );
    }

    async fn post(&mut self) {
        println!("I'm gonna try post, but who knows if it will work");

        let client = reqwest::Client::new();

        let now = chrono::Utc::now().timestamp_millis().to_string();

        let heartbeat_body: Vec<HeartbeatBody> = vec![HeartbeatBody {
            RoleName: "Web".to_owned(),
            InstrumentationKey: self.appinsights_key.to_owned(),
            InvariantVersion: "1".to_owned(),
            MachineName: self.hostname.to_owned(),
            Instance: self.instance.to_owned(),
            StreamId: self.stream_id.to_owned(),
            Timestamp: format!("/Date({})/", now),
            Version: "rust:0.0.1".to_owned(),
            Documents: serde_json::Value::Null,
            Metrics: self.get_metrics(),
        }];

        let transmission_time = (chrono::Utc::now().timestamp() + 62135596800000) * 10000;

        let mut heartbeat_headers = HeaderMap::new();
        heartbeat_headers.insert("Expect", "100-continue".parse().unwrap());
        heartbeat_headers.insert(
            "x-ms-qps-transmission-time",
            transmission_time.to_string().parse().unwrap(),
        );

        let response = client
                .post(format!(
                    "https://uksouth.livediagnostics.monitor.azure.com/QuickPulseService.svc/post?ikey={}",
                    self.appinsights_key
                ))
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

        println!(
            "Request {:?} {:?}",
            heartbeat_headers,
            serde_json::to_string(&heartbeat_body).unwrap()
        );
        println!(
            "Response {} {:?} {:?}",
            response.status(),
            response.headers().to_owned(),
            response.text().await.unwrap()
        );
    }

    fn get_metrics(&self) -> Vec<Metric> {
        let metrics: Vec<Metric> = vec![
            Metric {
                Name: r"\Processor(_Total)\% Processor Time".to_string(),
                Value: "3.0807660283097418".to_string(),
                Weight: 1,
            },
            Metric {
                Name: r"\ApplicationInsights\Requests/Sec".to_string(),
                Value: "0".to_string(),
                Weight: 1,
            },
            Metric {
                Name: r"\ApplicationInsights\Requests Failed/Sec".to_string(),
                Value: "0".to_string(),
                Weight: 1,
            },
            Metric {
                Name: r"\ApplicationInsights\Dependency Calls/Sec".to_string(),
                Value: "0".to_string(),
                Weight: 1,
            },
            Metric {
                Name: r"\ApplicationInsights\Dependency Calls Failed/Sec".to_string(),
                Value: "0".to_string(),
                Weight: 4
            },
            Metric {
                Name: r"\ApplicationInsights\Exceptions/Sec".to_string(),
                Value: "0".to_string(),
                Weight: 4
            },
            Metric {
                Name: r"\Memory\Committed Bytes".to_string(),
                Value: "7471194112".to_string(),
                Weight: 1,
            },
        ];
        println!("{}", serde_json::to_string(&metrics).unwrap());
        return metrics;
    }
}
