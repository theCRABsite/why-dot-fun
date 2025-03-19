use crate::{
    cache::CachedCall,
    database::{Database, Sponsor},
    secrets::Secrets,
    CONFIG,
};
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
};
use axum::{extract::Request, response::IntoResponse, Extension};
use chrono::Utc;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use twilio::{
    twiml::{Method, Redirect, Reject, Say, Twiml, Voice},
    Call, Client as TwilioClient,
};

pub async fn start_handler(
    twilio: Extension<TwilioClient>,
    cache: Extension<Arc<Mutex<HashMap<String, CachedCall>>>>,
    database: Extension<Database>,
    secrets: Extension<Secrets>,
    request: Request,
) -> impl IntoResponse {
    twilio
        .clone()
        .respond_to_webhook_async(request, |call: Call| async move {
            log::debug!("Received call from {} with id {}", call.from, call.sid);

            // Get or insert the user into the database
            let mut user = database
                .get_or_insert_user_by_phone_number(&call.from)
                .await
                .expect("Failed to get or insert user");

            log::debug!(
                "User {} has {} attempts today, last attempt at {}",
                user.phone_number,
                user.attempts_today,
                user.last_attempt
            );

            // If the user is banned, reject the call
            if user.banned {
                log::debug!("Rejecting call from banned user {}", user.phone_number);
                return generate_reject_twiml();
            }

            // Reset the daily attempt count if the last attempt was not today
            if user.last_attempt.date_naive() != Utc::now().date_naive() {
                user.attempts_today = 1;
            }

            // Update the user in the database
            database
                .update_user(&user)
                .await
                .expect("Failed to update user");

            // If the user has exceeded the daily response limit, reject the call
            if user.attempts_today > CONFIG.settings.daily_response_limit as i32 {
                log::debug!("Rejecting call from {} without response", call.from);
                return generate_reject_twiml();
            }

            // If the user has exceeded the daily attempt limit, respond with a messsage
            // notifying the user that they have exceeded the limit
            if user.attempts_today > CONFIG.settings.daily_attempt_limit as i32 {
                log::debug!("Rejecting call from {} with response", call.from);
                return generate_out_of_attempts_twiml();
            }

            // Get the sponsor for the call
            let sponsor = database
                .get_random_sponsor()
                .await
                .expect("Failed to get sponsor");


            // Create the attempt in the database
            database
                .create_attempt_with_sponsor(&user, &sponsor, call.sid.clone())
                .await
                .expect("Failed to create attempt");

            
            let twiml = generate_start_twiml(&sponsor.greeting_text);

            // let mut twiml = Twiml::new();

            // twiml.add(&Say {
            //     txt: sponsor.start_text.to_owned(),
            //     voice: Voice::Custom(CONFIG.settings.voice.to_owned()),
            //     language: CONFIG.settings.language.to_owned(),
            // });
        
            // twiml.add(&Redirect {
            //     method: Method::Post,
            //     url: "/challenge/start".to_owned(),
            // });

            // Add the call to the cache
            initialize_cached_call(&cache, call.sid.clone(), sponsor).await;

            // Start call recording
            tokio::spawn(start_call_recording(twilio.0, secrets.0, call.sid.clone()));

            twiml
        })
        .await
}

/// Generate the TwiML for the start of the call.
/// 1. Greet the user
/// 2. Redirect to the /name route to start the name query process
fn generate_start_twiml(greeting: &str) -> Twiml {
    let mut twiml = Twiml::new();

    twiml.add(&Say {
        txt: greeting.to_owned(),
        voice: Voice::Custom(CONFIG.settings.voice.to_owned()),
        language: CONFIG.settings.language.to_owned(),
    });

    twiml.add(&Redirect {
        method: Method::Post,
        url: "/redirect-gather/name".to_owned(),
    });

    twiml
}

/// Generate the TwiML for a banned user.
fn generate_reject_twiml() -> Twiml {
    let mut twiml = Twiml::new();

    twiml.add(&Reject::default());

    twiml
}

/// Generate the TwiML for a user who has exceeded the daily attempt limit.
fn generate_out_of_attempts_twiml() -> Twiml {
    let mut twiml = Twiml::new();

    twiml.add(&Say {
        txt: CONFIG.texts.out_of_attempts.replace(
            "$attempts",
            &CONFIG.settings.daily_attempt_limit.to_string(),
        ),
        voice: Voice::Custom(CONFIG.settings.voice.to_owned()),
        language: CONFIG.settings.language.to_owned(),
    });

    twiml
}

/// Initialize the conversation cache with two messages:
/// - The system message with the sponsor's system instruction
/// - The assistant message with the sponsor's greeting text
///
/// The system message is not audible and ignored by the subtitle
/// generation, it only serves to instruct the model on how to respond.
async fn initialize_cached_call(
    cache: &Arc<Mutex<HashMap<String, CachedCall>>>,
    call_sid: String,
    sponsor: Sponsor,
) {
    let mut cached_call = CachedCall::new(sponsor.clone());
    cached_call.add_system_message(
        ChatCompletionRequestSystemMessageArgs::default()
            .content(sponsor.system_instruction)
            .build()
            .expect("Failed to build system message")
            .into(),
    );
    cached_call.add_system_message(
        ChatCompletionRequestAssistantMessageArgs::default()
            .content(sponsor.greeting_text)
            .build()
            .expect("Failed to build system message")
            .into(),
    );

    cache.lock().await.insert(call_sid, cached_call);
}

/// Start the call recording. The recording may fail to start if the call status
/// on twilio's backend has not yet updated to `in-progress`. In this case, the
/// recording will be retried a number of times before giving up. This is a known
/// limitation of the Twilio API and the recommended approach by twilio evangelists.
async fn start_call_recording(twilio: TwilioClient, secrets: Secrets, call_sid: String) {
    for _ in 0..CONFIG.settings.record_retry {
        if let Ok(recording) = twilio
            .record_call(&call_sid, &format!("{}/recording", secrets.global_url))
            .await
        {
            log::debug!("Recording started with id {}", recording.sid);
            return;
        }
    }

    log::error!(
        "Failed to start recording after {} retries",
        CONFIG.settings.record_retry
    );
}
