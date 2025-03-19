use axum::{
    middleware::{self},
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_governor::{governor::GovernorConfig, GovernorLayer};

mod page;
mod verify;

pub fn router() -> Router {
    Router::new()
        .route("/", get(page::page_handler))
        .layer(middleware::from_fn(verify::verify))
        .layer(GovernorLayer {
            config: Arc::new(GovernorConfig::default()),
        })
}
