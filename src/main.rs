use async_openai::Client as OpenaiClient;
use axum::response::IntoResponse;
use axum::{
    routing::{get, post},
    Extension, Router,
};
use cache::CachedCall;
use database::Database;
use reqwest::header::HeaderValue;
use reqwest::Client as ReqwestClient;
use reqwest::StatusCode;
use secrets::Secrets;
use static_toml::static_toml;
use std::{collections::HashMap, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use twilio::Client as TwilioClient;
use twitter_v2::{authorization::Oauth1aToken, TwitterApi};
use axum::Json;

static_toml! { static CONFIG = include_toml!("Config.toml"); }

mod api;
mod cache;
mod claim;
mod database;
mod game;
mod review;
mod secrets;
mod solana;
mod video;
mod webcall;

#[tokio::main]
async fn main() {
    // Intitialize environment and logger
    dotenv::dotenv().ok();
    env_logger::init();

    // Load the secrets
    let secrets = Secrets::from_env();

    // Initialize the database
    log::info!("Connecting to the database");
    let database = Database::new(&secrets).await;

    // Initialize the twilio client
    log::info!("Initializing the Twilio client");
    let twilio = TwilioClient::new(&secrets.twilio_account_sid, &secrets.twilio_auth_token);

    // Initialize the OpenAI client
    log::info!("Initializing the OpenAI client");
    let openai = OpenaiClient::new();

    // Initialize the twitter client
    log::info!("Initializing the Twitter client");
    let twitter_token = Oauth1aToken::new(
        &secrets.twitter_api_key,
        &secrets.twitter_api_secret,
        &secrets.twitter_access_token,
        &secrets.twitter_access_secret,
    );
    let twitter = TwitterApi::new(twitter_token);

    // Initialize the reqwest client
    log::info!("Initializing the Reqwest client");
    let reqwest = ReqwestClient::new();

    // Initialize the conversation cache, maps the call id to all messages
    log::info!("Initializing the conversation cache");
    let cache = Arc::new(Mutex::new(HashMap::<String, CachedCall>::new()));

    // Initialize the TCP listener
    log::info!(
        "Connecting to the server at {}",
        CONFIG.settings.local_address
    );
    let tcp = TcpListener::bind(CONFIG.settings.local_address)
        .await
        .expect("Failed to connect to the server");

    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false);

    // Initialize the webserver routes
    log::info!("Initializing the webserver routes");
    let router = Router::new()
        .route("/health_check", get(health_check))
        .route("/start", post(game::start::start_handler))
        .route("/name", post(game::name::name_handler))
        .route("/challenge/start", post(game::challenge::start_handler))
        .route("/challenge/respond", post(game::challenge::respond_handler))
        .route("/end", post(game::end::end_handler))
        .route("/judge", post(game::judge::judge_handler))
        .route("/recording", post(game::recording::recording_handler))
        .route("/api/attempts/:id", get(api::attempt_single::attempt_single))
        .route("/api/sponsors", post(api::sponsor_list::sponsor_list))
        .route("/api/sponsor/update", post(api::update_sponsor::update_sponsor))
        .route("/api/attempts", get(api::attempt_list::attempt_list))
        .route("/api/launchpad", post(api::launchpad::launchpad))
        .route("/api/payment", post(api::payment::payment))
        .route("/api/deposit", post(api::deposit::deposit))
        .route("/api/activate-sponsor", post(api::activate_sponsor::activate_sponsor))
        .route("/api/verify-winner", post(api::verify_winner::verify_winner))
        .route(
            "/redirect-gather/*path",
            post(game::gather::redirect_gather_handler),
        )
        .nest_service("/", webcall::router())
        .nest_service("/claim", claim::router())
        .nest_service("/review", review::router())
        .nest_service("/static", ServeDir::new("static"))
        .fallback(error_handler)
        .layer(cors)
        .layer(Extension(secrets))
        .layer(Extension(twilio))
        .layer(Extension(openai))
        .layer(Extension(twitter))
        .layer(Extension(reqwest))
        .layer(Extension(database))
        .layer(Extension(cache));

    // Start the webserver
    log::info!("Starting the webserver");
    axum::serve(tcp, router.into_make_service())
        .await
        .expect("Failed to start the server");
}

pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

async fn error_handler() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::GATEWAY_TIMEOUT,
        Json(serde_json::json!({ "error": "Gateway Timeout" })),
    )
}