use crate::database::{Sponsor, Winner};
use askama::Template;
use axum::Extension;

pub async fn page_handler(
    Extension(winner): Extension<Winner>,
    Extension(sponsor): Extension<Sponsor>,
) -> ClaimPage {
    // Render the html page with winner and sponsor info
    ClaimPage { winner, sponsor }
}

#[derive(Debug, Template)]
#[template(path = "claim.html")]
pub struct ClaimPage {
    winner: Winner,
    sponsor: Sponsor,
}
