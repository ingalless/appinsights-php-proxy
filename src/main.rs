use axum::{
    error_handling::HandleErrorLayer,
    response::{self},
    routing::{get, post},
    BoxError, Router,
};
use axum_extra::response::JsonLines;
use config::Config;
use dotenvy::dotenv;
use futures_util::StreamExt;
use quickpulse::Client;
use serde_json::{json, Value};
use std::{env, io, sync::Arc};
use tower::ServiceBuilder;
use tower_http::decompression::RequestDecompressionLayer;

mod performance;
mod quickpulse;
mod config;
mod envelope;

const APPLICATIONINSIGHTS_CONNECTION_STRING: &str = "APPLICATIONINSIGHTS_CONNECTION_STRING";
const APPINSIGHTS_PROXY_SERVER_PORT: &str = "APPINSIGHTS_PROXY_SERVER_PORT";

struct ServicesState {
    _appinsights_client: appinsights::TelemetryClient,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv().expect(".env file not found");

    let appinsights_connstring: String = env::var(APPLICATIONINSIGHTS_CONNECTION_STRING).unwrap();
    let config = Config::new(appinsights_connstring);

    let proxy_server_port: i16 = match env::var(APPINSIGHTS_PROXY_SERVER_PORT) {
        Ok(v) => v.parse().unwrap(),
        Err(_) => 3000,
    };

    let mut quickpulse_client = Client::new(config.ikey.to_owned(), config.live_endpoint);
    let _appinsights_client = appinsights::TelemetryClient::new(config.ikey.to_owned());

    let shared_state = Arc::new(ServicesState { _appinsights_client });

    tokio::spawn(async move {
        quickpulse_client.go().await;
    });

    let app = Router::new()
        .route("/status", get(status))
        .route("/v2.1/track", post(track))
        .with_state(shared_state)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .layer(RequestDecompressionLayer::new()),
        );

    axum::Server::bind(&format!("0.0.0.0:{}", proxy_server_port).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn handle_error(_err: BoxError) -> (axum::http::StatusCode, String) {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "Something went wrong".to_string(),
    )
}

async fn status() -> response::Json<Value> {
    response::Json(json!({ "message": "ok" }))
}

/**
* https://github.com/MohanGsk/ApplicationInsights-Home/blob/master/EndpointSpecs/ENDPOINT-PROTOCOL.md
*/
async fn track(mut stream: JsonLines<envelope::EnvelopeTelemetry>) {
    while let Some(chunk) = stream.next().await {
        let envelope = chunk.unwrap();
        match envelope.data {
            envelope::BaseData::MetricData(data) => println!("metric: {:?}", data),
            envelope::BaseData::RequestData(data) => println!("request: {:?}", data),
        }
    }
}
