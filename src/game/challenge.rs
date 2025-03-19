use crate::{cache::CachedCall, secrets::Secrets, CONFIG};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client as OpenAIClient,
};
use axum::{extract::Request, response::IntoResponse, Extension};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use twilio::{
    twiml::{Gather, GatherInput, Method, Redirect, Say, SpeechTimeout, Twiml, Voice},
    Call, Client,
};

pub async fn start_handler(
    twilio: Extension<Client>,
    cache: Extension<Arc<Mutex<HashMap<String, CachedCall>>>>,
    secrets: Extension<Secrets>,
    request: Request,
) -> impl IntoResponse {
    twilio
        .clone()
        .respond_to_webhook_async(request, |call: Call| async move {
            {
                let mut cache = cache.lock().await;
                let cached_call = cache
                    .get_mut(&call.sid)
                    .expect("Failed to get message conversation");

                let challenge_time = cached_call.sponsor.challenge_time;
                cached_call.end_last_message();

                // Start the timer that will redirect the call to the /end route
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(challenge_time as _)).await;
                    let url = format!("{}/end", secrets.global_url);
                    let _ = twilio.update_call_url(&call.sid, &url).await;
                });
            }

            let mut twiml = Twiml::new();

            log::debug!("Gathering user response");

            twiml.add(&Gather {
                timeout_seconds: CONFIG.settings.timeout as u32,
                action: Some("/challenge/respond".to_owned()),
                input: Some(GatherInput::Speech),
                speech_timeout: Some(SpeechTimeout::Auto),
                speech_model: Some(CONFIG.settings.speech_model.to_owned()),
                ..Default::default()
            });

            twiml
        })
        .await
}

pub async fn respond_handler(
    twilio: Extension<Client>,
    openai: Extension<OpenAIClient<OpenAIConfig>>,
    cache: Extension<Arc<Mutex<HashMap<String, CachedCall>>>>,
    request: Request,
) -> impl IntoResponse {
    twilio
        .clone()
        .respond_to_webhook_async(request, |call: Call| async move {
            log::debug!(
                "Understood: {:?} with confidence {:?}",
                call.speech_result,
                call.speech_confidence
            );

            let mut twiml = Twiml::new();

            // If there is a transcription, it is a response to a previous user message.
            // If no transcription is available, the challenge has just been started and the cache
            // should be updated with the system message and the timer should be started to end
            // the gameshow after a certain amount of time.
            if let Some(speech_result) = call.speech_result {
                // Load the conversation from the cache
                let mut cached_call = {
                    let cache = cache.lock().await;
                    cache
                        .get(&call.sid)
                        .expect("Failed to get message conversation")
                        .clone()
                };
                log::debug!(
                    "Loaded {} messages: {:?}",
                    cached_call.messages.len(),
                    cached_call.messages
                );

                // Add the user message to the conversation
                cached_call.add_user_message(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(speech_result)
                        .build()
                        .expect("Failed to build user message")
                        .into(),
                );

                // Generate a response to the conversation
                let completion = generate_response(&openai, &cached_call.messages)
                    .await
                    .expect("Failed to generate response");

                log::debug!("Generated completion: {}", completion);

                // Add the assistant message to the conversation
                cached_call.add_system_message(
                    ChatCompletionRequestAssistantMessageArgs::default()
                        .content(completion.clone())
                        .build()
                        .expect("Failed to build assistant message")
                        .into(),
                );

                {
                    let mut cache = cache.lock().await;
                    cache.insert(call.sid, cached_call);
                }

                // Speak the generated response
                twiml.add(&Say {
                    txt: completion,
                    voice: Voice::Custom(CONFIG.settings.voice.to_owned()),
                    language: CONFIG.settings.language.to_owned(),
                });
            }

            // This redirect is necessary to extract the timestamp in between
            // the system message and the user response
            twiml.add(&Redirect {
                method: Method::Post,
                url: "/redirect-gather/challenge/respond".to_owned(),
            });

            twiml
        })
        .await
}

async fn generate_response(
    openai: &Extension<OpenAIClient<OpenAIConfig>>,
    messages: &[ChatCompletionRequestMessage],
) -> Option<String> {
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(CONFIG.challenge.max_tokens as u32)
        .model(CONFIG.challenge.model)
        .messages(messages)
        .build()
        .ok()?;

    openai
        .chat()
        .create(request)
        .await
        .ok()?
        .choices
        .first()?
        .message
        .content
        .clone()
}
