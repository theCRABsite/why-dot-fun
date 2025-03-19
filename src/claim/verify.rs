use crate::database::Database;
use axum::{
    extract::{Query, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Extension,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ClaimQuery {
    key: String,
}

/// Middleware that checks if the claim key is valid and hydrates
/// the request with important info about the winner and sponsor.
pub async fn verify(
    database: Extension<Database>,
    query: Query<ClaimQuery>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Hydrate the request with the winner if the key is valid
    let winner = match database.get_winner_by_key(&query.key).await {
        Ok(Some(winner)) => {
            request.extensions_mut().insert(winner.clone());
            winner
        }
        Ok(None) => return Err(StatusCode::UNAUTHORIZED),
        Err(_) => {
            log::error!("Failed to check claim key");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Hydrate the request with the sponsor
    match database.get_sponsor_by_id(winner.sponsor_id).await {
        Ok(sponsor) => request.extensions_mut().insert(sponsor),
        Err(e) => {
            log::error!("Failed to get sponsor: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(next.run(request).await)
}
