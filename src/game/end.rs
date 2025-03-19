use crate::{cache::CachedCall, CONFIG};
use async_openai::types::ChatCompletionRequestAssistantMessageArgs;
use axum::{extract::Request, response::IntoResponse, Extension};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use twilio::{
    twiml::{Method, Redirect, Say, Twiml, Voice},
    Call, Client,
};


pub async fn end_handler(
    twilio: Extension<Client>,
    cache: Extension<Arc<Mutex<HashMap<String, CachedCall>>>>,
    request: Request,
) -> impl IntoResponse {
    twilio
        .clone()
        .respond_to_webhook_async(request, |call: Call| async move {
            let end_text = {
                let mut cache = cache.lock().await;
                let cached_call = cache
                    .get_mut(&call.sid)
                    .expect("Failed to get message conversation");

                let end_text = cached_call.sponsor.end_text.to_owned();

                cached_call.add_system_message(
                    ChatCompletionRequestAssistantMessageArgs::default()
                        .content(cached_call.sponsor.end_text.to_owned())
                        .build()
                        .expect("Failed to build system message")
                        .into(),
                );

                end_text
            };

            let mut twiml = Twiml::new();

            twiml.add(&Say {
                txt: end_text,
                voice: Voice::Custom(CONFIG.settings.voice.to_owned()),
                language: CONFIG.settings.language.to_owned(),
            });


            twiml.add(&Redirect {
                url: "/judge".to_owned(),
                method: Method::Post,
            });

            twiml
        })
        .await
}
