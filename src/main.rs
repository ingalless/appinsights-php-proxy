use axum::{
    error_handling::HandleErrorLayer,
    response::{self},
    routing::{get, post},
    BoxError, Router,
};
use axum_extra::response::JsonLines;
use dotenvy::dotenv;
use futures_util::StreamExt;
use quickpulse::Client;
use serde_json::{json, Value};
use std::{env, io};
use tower::ServiceBuilder;
use tower_http::decompression::RequestDecompressionLayer;

mod performance;
mod quickpulse;

const INSTRUMENTATION_KEY: &str = "APPINSIGHTS_INSTRUMENTATIONKEY";
const APPLICATIONINSIGHTS_CONNECTION_STRING: &str = "APPLICATIONINSIGHTS_CONNECTION_STRING";
const APPINSIGHTS_PROXY_SERVER_PORT: &str = "APPINSIGHTS_PROXY_SERVER_PORT";

struct Config {
    ikey: String,
    ingestion_endpoint: String,
    live_endpoint: String,
}

impl Config {
    fn new(connstring: String) -> Config {
        let mut ikey = "";
        let mut ingestion_endpoint = "";
        let mut live_endpoint = "";
        for part in connstring.split(";") {
            println!("{:?}", part);
            let (key, value) = part.split_once("=").unwrap();
            match key {
                "InstrumentationKey" => ikey = value,
                "IngestionEndpoint" => ingestion_endpoint = value,
                "LiveEndpoint" => live_endpoint = value,
                _ => ()
            }
        }

        Config {
            ikey: ikey.to_string(),
            ingestion_endpoint: ingestion_endpoint.to_string(),
            live_endpoint: live_endpoint.to_string(),
        }
    }
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

    let mut client = Client::new(config.ikey);

    tokio::spawn(async move {
        client.go().await;
    });

    let app = Router::new()
        .route("/status", get(status))
        .route("/v2.1/track", post(track))
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
async fn track(mut stream: JsonLines<Value>) {
    while let Some(chunk) = stream.next().await {
        println!("Line: {:?}", chunk.unwrap());
        println!("--------------");
        println!("");
        println!("");
    }
}
