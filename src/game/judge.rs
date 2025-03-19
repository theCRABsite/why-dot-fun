use crate::{cache::CachedCall, database::Database, secrets::Secrets, video::render_video, CONFIG};
use anyhow::{Context, Result};
use async_openai::{
    config::OpenAIConfig,
    types::{CreateChatCompletionRequestArgs, ResponseFormat, ResponseFormatJsonSchema},
    Client as OpenAIClient,
};
use axum::{extract::Request, response::IntoResponse, Extension};
use reqwest::Client as ReqwestClient;
use reqwest::header::COOKIE;
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use twilio::{twiml::Twiml, Call, Client as TwilioClient, OutboundMessage};
use crate::solana::transfer::transfer_solana_token;
use crate::solana::keys::generate_private_key;
use solana_sdk::signature::Signer;


pub async fn judge_handler(
    twilio: Extension<TwilioClient>,
    reqwest: Extension<ReqwestClient>,
    openai: Extension<OpenAIClient<OpenAIConfig>>,
    cache: Extension<Arc<Mutex<HashMap<String, CachedCall>>>>,
    database: Extension<Database>,
    secrets: Extension<Secrets>,
    request: Request,
) -> impl IntoResponse {
    twilio
        .clone()
        .respond_to_webhook_async(request, |call: Call| async move {
            let cached_call = {
                let mut cache = cache.lock().await;
                let mut cached_call = cache
                    .remove(&call.sid)
                    .expect("Failed to get message conversation");

                cached_call.end_last_message();
                cached_call.clone()
            };

            tokio::spawn(judge_conversation(
                twilio.0,
                reqwest.0,
                call.from,
                openai.0,
                database.0,
                secrets.0,
                call.sid,
                cached_call,
            ));

            Twiml::new()
        })
        .await
}

#[derive(Debug, Deserialize)]
pub struct JudgeResponse {
    pub won_prize: bool,
    pub rating: u8,
    pub explanation: String,
}

async fn judge_conversation(
    twilio: TwilioClient,
    reqwest: ReqwestClient,
    caller_phone_number: String,
    openai: OpenAIClient<OpenAIConfig>,
    database: Database,
    secrets: Secrets,
    call_sid: String,
    cached_call: CachedCall,
) {
    let schema = json!({
        "type": "object",
        "properties": {
            "won_prize": {
                "type": "boolean",
                "description": CONFIG.end.won_schema_property
            },
            "rating": {
                "type": "integer",
                "description": CONFIG.end.rating_schema_property
            },
            "explanation": {
                "type": "string",
                "description": CONFIG.end.explanation_schema_property
            }
        },
        "required": ["won_prize", "rating", "explanation"],
        "additionalProperties": false,
    });

    let response_format = ResponseFormat::JsonSchema {
        json_schema: ResponseFormatJsonSchema {
            description: Some(CONFIG.end.schema_description.to_owned()),
            name: "call_analyzing".to_owned(),
            schema: Some(schema),
            strict: Some(true),
        },
    };

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(CONFIG.end.max_tokens as u32)
        .model(CONFIG.end.model)
        .messages(cached_call.messages.clone())
        .response_format(response_format)
        .build()
        .expect("Failed to build chat completion request");

    let response = openai
        .chat()
        .create(request)
        .await
        .expect("Failed to create chat completion");

    let choice = response
        .choices
        .first()
        .expect("Failed to get first choice");

    let content = choice
        .message
        .content
        .as_ref()
        .expect("Failed to get content");

    println!("user: {}, judgement: {}", caller_phone_number, content);

    let judged: JudgeResponse =
        serde_json::from_str(&content).expect("Failed to judge conversation");


    log::debug!(
        "Judged conversation a {}/10 with explanation: {}",
        judged.rating,
        judged.explanation
    );

    // if caller_phone_number == "+4915142862539" || caller_phone_number == "+31641600877" {
    //     println!("Testing call: {:?}", judged);
    //     return;
    // }

    let _attempt = database
        .update_attempt_judgement(call_sid.clone(), judged.explanation.clone())
        .await
        .context("Updating attempt with judgement")
        .expect("Failed to update attempt with judgement");

    tokio::spawn(render_video(
        reqwest.clone(),
        secrets.clone(),
        call_sid.clone(),
        cached_call.clone(),
        judged.rating,
        database.clone(),
        judged.explanation.clone()
    ));

    let video_url = format!("https://gamecall.ams3.cdn.digitaloceanspaces.com/{call_sid}.mp4");

    let _attempt = database
        .update_attempt_video(caller_phone_number.clone(), video_url.clone(), call_sid.clone())
        .await
        .context("Updating attempt with is_winner true")
        .expect("Failed to update attempt with video url");


    let result = match judged.won_prize {
        true => won_handler(twilio, database, secrets, caller_phone_number, call_sid.clone(), cached_call, video_url).await,
        false => lost_handler(twilio, database, secrets, caller_phone_number, call_sid.clone(), cached_call).await,
    };

    if let Err(e) = result {
        log::error!("Failed to handle call judge result: {e:?}");
    }
}

async fn won_handler(
    twilio: TwilioClient,
    database: Database,
    secrets: Secrets,
    caller_phone_number: String,
    call_sid: String,
    cached_call: CachedCall,
    video_url: String,
) -> Result<()> {
    log::debug!("Won prize for sponsor: {}", cached_call.sponsor.name);

    // Withdraw tokens from the sponsor
    let withdrawn = database
        .withdraw_tokens(cached_call.sponsor.id)
        .await
        .context("Withdrawing tokens")?;

    // If withdrawing tokens failed, redirect to lost handler
    if withdrawn.is_none() {
        return lost_handler(twilio, database, secrets, caller_phone_number, call_sid.clone(), cached_call).await;
    };

    // Generate a winner entry in the database
    let _winner = database
        .create_winner(cached_call.name.clone(), cached_call.sponsor.id)
        .await
        .context("Creating winner")?;


    let _attempt = database
        .update_attempt_winner(caller_phone_number.clone(), true, call_sid.clone())
        .await
        .context("Updating attempt with is_winner true")?;


    let receiver_private_key = generate_private_key();
    let receiver_public_key = receiver_private_key.pubkey();


    let signature = transfer_solana_token(
        &secrets,
        cached_call.sponsor.private_key,
        receiver_public_key,
        cached_call.sponsor.token_mint,
        cached_call.sponsor.reward_tokens.try_into().unwrap()
    ).await.expect("Failed to transfer tokens");
    println!("user: {}, signature: {}", caller_phone_number, signature);

    // Generate the winning link
    let link = format!("https://claim.why.fun/?key={}", receiver_private_key.to_base58_string());
    println!("user: {}, link: {}", caller_phone_number, link);

    database.update_attempt_winner_url(
        caller_phone_number.clone(), 
        link.clone(), 
        call_sid.clone()
    ).await.context("Updating attempt with winner url")?;

    // Generate the winning text
    let text = cached_call
        .sponsor
        .won_text
        .replace("{name}", &cached_call.name)
        .replace("{link}", &link)
        .replace("{video_url}", &video_url);

    twilio
        .send_message(OutboundMessage {
            from: &secrets.twilio_phone_number,
            to: &caller_phone_number,
            body: &text,
        })
        .await
        .context("Sending message")?;

    Ok(())
}

async fn lost_handler(
    twilio: TwilioClient,
    database: Database,
    secrets: Secrets,
    caller_phone_number: String,
    call_sid: String,
    cached_call: CachedCall,
) -> Result<()> {
    log::debug!("Lost prize for sponsor: {}", cached_call.sponsor.name);

    let _attempt = database
        .update_attempt_winner(caller_phone_number.clone(), false, call_sid.clone())
        .await
        .context("Updating attempt with is_winner false")?;

    // Generate the loosing text
    let text = cached_call
        .sponsor
        .lost_text
        .replace("{name}", &cached_call.name);

    twilio
        .send_message(OutboundMessage {
            from: &secrets.twilio_phone_number,
            to: &caller_phone_number,
            body: &text,
        })
        .await
        .context("Sending message")?;

    Ok(())
}
