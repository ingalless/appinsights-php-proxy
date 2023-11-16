use std::collections::HashMap;
use serde::Serialize;
use sysinfo::{System, SystemExt, CpuExt};

pub struct AppCpuUsage {
    pub user: i32,
    pub system: i32,
}

pub struct CpuUsage {
    pub model: String,
    pub speed: i32,
    pub times: CpuTime,
}

pub struct CpuTime {
    pub user: i32,
    pub nice: i32,
    pub sys: i32,
    pub idle: i32,
    pub irq: i32,
}

pub struct ProcessedDocumentData {
    pub count: i32,
    pub failed: i32,
    pub time: f32,
}

pub struct ProcessedExceptionData {
    pub count: i32,
    pub time: f32,
}

pub struct PerformanceCollector {
    pub enabled: bool,
    pub system: System,
    pub metrics: HashMap<QuickPulseMetricNames, String>,
}

#[derive(PartialEq, Eq, Hash, Serialize, Clone)]
pub enum QuickPulseMetricNames {
    TotalProcessorTime,
    RequestsPerSecond,
    FailedRequestsPerSecond,
    DependencyCallsPerSecond,
    FailedDependencyCallsPerSecond,
    ExceptionsPerSecond,
    MemoryCommittedBytes,
}

impl QuickPulseMetricNames {
    pub fn as_str(&self) -> &'static str {
        match self {
            QuickPulseMetricNames::TotalProcessorTime => r"\Processor(_Total)\% Processor Time",
            QuickPulseMetricNames::RequestsPerSecond => r"\ApplicationInsights\Requests/Sec",
            QuickPulseMetricNames::FailedRequestsPerSecond => {
                r"\ApplicationInsights\Requests Failed/Sec"
            }
            QuickPulseMetricNames::DependencyCallsPerSecond => {
                r"\ApplicationInsights\Dependency Calls/Sec"
            }
            QuickPulseMetricNames::FailedDependencyCallsPerSecond => {
                r"\ApplicationInsights\Dependency Calls Failed/Sec"
            }
            QuickPulseMetricNames::ExceptionsPerSecond => r"\ApplicationInsights\Exceptions/Sec",
            QuickPulseMetricNames::MemoryCommittedBytes => r"\Memory\Committed Bytes",
        }
    }
}

impl PerformanceCollector {
    pub fn new() -> Self {
        Self {
            enabled: false,
            system: System::new_all(),
            metrics: HashMap::new(),
        }
        
    }

    pub fn collect(&mut self) {
        self.enabled = true;
        self.system.refresh_all();

        self.metrics.clear();

        self.metrics.insert(QuickPulseMetricNames::RequestsPerSecond, "0".to_string());
        self.metrics.insert(QuickPulseMetricNames::FailedRequestsPerSecond, "0".to_string());
        self.metrics.insert(QuickPulseMetricNames::DependencyCallsPerSecond, "0".to_string());
        self.metrics.insert(QuickPulseMetricNames::FailedDependencyCallsPerSecond, "0".to_string());
        self.metrics.insert(QuickPulseMetricNames::ExceptionsPerSecond, "0".to_string());
        self.metrics.insert(QuickPulseMetricNames::MemoryCommittedBytes, self.system.used_memory().to_string());
        self.metrics.insert(QuickPulseMetricNames::TotalProcessorTime, self.system.global_cpu_info().cpu_usage().to_string());
    }
}
