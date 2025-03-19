use super::check::check_token;
use crate::{secrets::Secrets, CONFIG};
use axum::Extension;
use axum_auth::AuthBearer;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use reqwest::StatusCode;
use serde::Serialize;

#[derive(Serialize)]
struct Grants {
    identity: String,
    voice: VoiceGrant,
}

#[derive(Serialize)]
struct VoiceGrant {
    incoming: IncomingVoiceGrant,
    outgoing: OutgoingVoiceGrant,
}

#[derive(Serialize)]
struct IncomingVoiceGrant {
    allow: bool,
}

#[derive(Serialize)]
struct OutgoingVoiceGrant {
    application_sid: String,
}

#[derive(Serialize)]
struct Claims {
    jti: String,
    iss: String,
    sub: String,
    iat: i64,
    nbf: i64,
    exp: i64,
    grants: Grants,
}

pub async fn generate_jwt(
    secrets: Extension<Secrets>,
    AuthBearer(token): AuthBearer,
) -> Result<String, StatusCode> {
    // Check whether the user should be allowed to generate a token
    if !check_token(&token, &secrets) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Set current time and expiration
    let now = Utc::now();
    let iat = now.timestamp();
    let nbf = iat;
    let exp = (now + Duration::seconds(CONFIG.settings.twilio_token_expiry)).timestamp();

    // Build grants
    let grants = Grants {
        identity: "webapp".to_string(),
        voice: VoiceGrant {
            incoming: IncomingVoiceGrant { allow: false },
            outgoing: OutgoingVoiceGrant {
                application_sid: secrets.twilio_app_sid.clone(),
            },
        },
    };

    // Build claims
    let claims = Claims {
        jti: format!("{}-{}", secrets.twilio_api_key, iat),
        iss: secrets.twilio_api_key.clone(),
        sub: secrets.twilio_account_sid.clone(),
        iat,
        nbf,
        exp,
        grants,
    };

    // Create header
    let header = Header {
        cty: Some("twilio-fpa;v=1".to_string()),
        ..Default::default()
    };

    // Encode the token
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secrets.twilio_api_secret.as_bytes()),
    )
    .map_err(|e| {
        log::error!("Error encoding Twilio token: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}
