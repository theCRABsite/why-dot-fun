use crate::{cache::CachedCall, CONFIG};
use axum::{
    extract::{Path, Request},
    response::IntoResponse,
    Extension, RequestExt,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use twilio::{
    twiml::{Gather, GatherInput, SpeechTimeout, Twiml},
    Call, Client,
};

pub async fn redirect_gather_handler(
    twilio: Extension<Client>,
    cache: Extension<Arc<Mutex<HashMap<String, CachedCall>>>>,
    mut request: Request,
) -> impl IntoResponse {
    let path = request
        .extract_parts::<Path<String>>()
        .await
        .expect("Failed to extract path")
        .0;

    twilio
        .respond_to_webhook_async(request, |call: Call| async move {
            {
                // Update the last timestamp in the conversation cache
                cache
                    .lock()
                    .await
                    .get_mut(&call.sid)
                    .expect("Failed to get message conversation")
                    .end_last_message();
            }

            let mut twiml = Twiml::new();

            log::debug!("Gathering user response");

            // Collect the user's response and send it to the name handler
            twiml.add(&Gather {
                timeout_seconds: CONFIG.settings.timeout as u32,
                action: Some(format!("/{path}")),
                input: Some(GatherInput::Speech),
                speech_timeout: Some(SpeechTimeout::Auto),
                speech_model: Some(CONFIG.settings.speech_model.to_owned()),
                ..Default::default()
            });

            twiml
        })
        .await
}
