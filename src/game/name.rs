use crate::cache::CachedCall;
use crate::database::Sponsor;
use crate::CONFIG;
use anyhow::{anyhow, Context, Result};
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs, ResponseFormat, ResponseFormatJsonSchema,
};
use async_openai::{config::OpenAIConfig, Client as OpenAIClient};
use axum::{extract::Request, response::IntoResponse, Extension};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use twilio::twiml::{Method, Redirect};
use twilio::{
    twiml::{Say, Twiml, Voice},
    Call, Client as TwilioClient,
};

pub async fn name_handler(
    twilio: Extension<TwilioClient>,
    openai: Extension<OpenAIClient<OpenAIConfig>>,
    cache: Extension<Arc<Mutex<HashMap<String, CachedCall>>>>,
    request: Request,
) -> impl IntoResponse {
    twilio
        .clone()
        .respond_to_webhook_async(request, |call: Call| async move {
            let speech_confidence = call.speech_confidence.map(|c| c * 100.0);
            log::debug!(
                "Understood: {:?} with confidence {:?}%",
                call.speech_result,
                speech_confidence
            );

            // Extract the sponsor from the cache
            let sponsor = {
                let cache = cache.lock().await;
                cache
                    .get(&call.sid)
                    .expect("Failed to get message conversation")
                    .sponsor
                    .clone()
            };

            // Try to extract the name from the transcription
            let name = match &call.speech_result {
                Some(text) => match extract_name(openai, text).await {
                    Ok(name) => name,
                    Err(e) => {
                        log::error!("Failed to extract name: {:?}", e);
                        None
                    }
                },
                None => None,
            };
            log::debug!("Extracted name: {:?}", name);

            // Generate the response based on the extracted name
            let (twiml, response) = generate_name_response(name.clone(), sponsor).await;

            // Update the conversation cache
            update_conversation_cache(
                &cache,
                call.sid,
                call.speech_result.unwrap_or_default(),
                name,
                response,
            )
            .await;

            twiml
        })
        .await
}

/// Updates the cached call messages:
/// 1. Adds the recognized user message
/// 2. Adds the generated assistant message
async fn update_conversation_cache(
    cache: &Arc<Mutex<HashMap<String, CachedCall>>>,
    call_sid: String,
    user_message: String,
    name: Option<String>,
    assistant_message: String,
) {
    let mut cache = cache.lock().await;
    let cached_call = cache
        .get_mut(&call_sid)
        .expect("Failed to get message conversation");

    if let Some(name) = name {
        cached_call.name = name;
    }

    cached_call.add_user_message(
        ChatCompletionRequestUserMessageArgs::default()
            .content(user_message)
            .build()
            .expect("Failed to build user message")
            .into(),
    );

    cached_call.add_system_message(
        ChatCompletionRequestAssistantMessageArgs::default()
            .content(assistant_message)
            .build()
            .expect("Failed to build assistant message")
            .into(),
    )
}

/// Generates the response based on the extracted name (if any):
/// 1. If a name was found, start the challenge
/// 2. If no name was found, ask for the name again
async fn generate_name_response(name: Option<String>, sponsor: Sponsor) -> (Twiml, String) {
    // If a name could be extracted, start the challenge, otherwise ask for the name again
    let response = match &name {
        Some(name) => sponsor
            .start_text
            .replace("{name}", name)
            .replace("{duration}", &sponsor.challenge_time.to_string()),
        None => CONFIG.texts.name_not_found.to_owned(),
    };

    let next_url = match &name {
        Some(_) => "/challenge/start".to_owned(),
        None => "/redirect-gather/name".to_owned(),
    };

    // Generate the twilio response
    let mut twiml = Twiml::new();
    twiml.add(&Say {
        txt: response.clone(),
        voice: Voice::Custom(CONFIG.settings.voice.to_owned()),
        language: CONFIG.settings.language.to_owned(),
    });

    twiml.add(&Redirect {
        url: next_url,
        method: Method::Post,
    });

    (twiml, response)
}

#[derive(Debug, Deserialize)]
struct ExtractedName {
    name: String,
}

async fn extract_name(
    openai: Extension<OpenAIClient<OpenAIConfig>>,
    text: &str,
) -> Result<Option<String>> {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {
                "type": ["string", "null"],
                "description": CONFIG.name.schema_property
            }
        },
        "required": ["name"],
        "additionalProperties": false,
    });

    let response_format = ResponseFormat::JsonSchema {
        json_schema: ResponseFormatJsonSchema {
            description: Some("".to_owned()),
            name: "name_extraction".to_owned(),
            schema: Some(schema),
            strict: Some(true),
        },
    };

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(CONFIG.name.max_tokens as u32)
        .model(CONFIG.name.model)
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(text)
            .build()?
            .into()])
        .response_format(response_format)
        .build()?;

    let response = openai.chat().create(request).await?;
    let choice = response
        .choices
        .first()
        .ok_or_else(|| anyhow!("No first completion choice could be generated"))?;
    let content = choice
        .message
        .content
        .as_ref()
        .ok_or_else(|| anyhow!("No content in the completion choice"))?;

    let extracted: ExtractedName =
        serde_json::from_str(&content).context("Extracting name from completion choice")?;

    match extracted.name.as_str() {
        "null" => Ok(None),
        _ => Ok(Some(extracted.name)),
    }
}
