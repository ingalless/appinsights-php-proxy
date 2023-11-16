use dotenvy::dotenv;
use quickpulse::Client;
use std::{env, io};

mod quickpulse;
mod performance;

const INSTRUMENTATION_KEY: &str = "APPINSIGHTS_INSTRUMENTATIONKEY";

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv().expect(".env file not found");

    let app_insights_key: String =
        env::var(INSTRUMENTATION_KEY).expect("Var APPINSIGHTS_INSTRUMENTATIONKEY not set");
    let _appinsights = appinsights::TelemetryClient::new(app_insights_key.to_string());

    let mut client = Client::new(app_insights_key);

    // tokio::spawn(async move {
    //     client.go().await;
    // });

    client.go().await;
    Ok(())

    // loop {}
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
