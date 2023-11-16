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
const APPINSIGHTS_PROXY_SERVER_PORT: &str = "APPINSIGHTS_PROXY_SERVER_PORT";

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv().expect(".env file not found");

    let app_insights_key: String =
        env::var(INSTRUMENTATION_KEY).expect("Var APPINSIGHTS_INSTRUMENTATIONKEY not set");
    let proxy_server_port: i16 = match env::var(APPINSIGHTS_PROXY_SERVER_PORT) {
        Ok(v) => v.parse().unwrap(),
        Err(_) => 3000,
    };

    let mut client = Client::new(app_insights_key);

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

async fn track(mut stream: JsonLines<Value>) {
    while let Some(chunk) = stream.next().await {
        println!("Line: {:?}", chunk.unwrap());
        println!("--------------");
        println!("");
        println!("");
    }
}
