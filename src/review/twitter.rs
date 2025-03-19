use super::Draft;
use crate::secrets::Secrets;
use anyhow::{anyhow, Result};
use reqwest::{
    multipart::{Form, Part},
    Client as ReqwestClient,
};
use reqwest_oauth1::{OAuthClientProvider, Secrets as OauthSecrets};
use serde::Deserialize;
use std::{iter, time::Duration, time::Instant};
use tokio::time::sleep;
use twitter_v2::{authorization::Oauth1aToken, TwitterApi};
use crate::database::Database;


const CHUNK_SIZE: usize = 5 * 1024 * 1024;
const STATUS_CHECK_INTERVAL: Duration = Duration::from_secs(2);

pub async fn upload_video(
    reqwest: &ReqwestClient,
    secrets: &Secrets,
    draft: &Draft,
) -> Result<u64> {
    let oauth = OauthSecrets::new(&secrets.twitter_api_key, &secrets.twitter_api_secret).token(
        &secrets.twitter_access_token,
        &secrets.twitter_access_secret,
    );

    sleep(Duration::from_secs(10)).await;

    let video_data = tokio::fs::read(format!("cache/drafts/{}.mp4", draft.call_sid)).await?;

    let media_id = init_video_upload(reqwest.clone(), oauth.clone(), video_data.len()).await?;

    for (i, chunk) in video_data.chunks(CHUNK_SIZE).enumerate() {
        append_video_upload(reqwest.clone(), oauth.clone(), media_id, i, chunk).await?;
    }

    finalize_video_upload(reqwest.clone(), oauth.clone(), media_id).await?;
    wait_video_upload_successful(reqwest.clone(), oauth.clone(), media_id).await?;

    Ok(media_id)
}

pub async fn post_tweet(
    twitter: &TwitterApi<Oauth1aToken>,
    media_id: u64,
    draft: &Draft,
) -> Result<()> {

    let sponsored = String::from("\n.\n.\n.\npowered by @theCRABsite");

    let tweet_text = format!("{} {}", draft.comment, sponsored);

    let tweet_object = twitter
        .post_tweet()
        .text(tweet_text.to_owned())
        .add_media(iter::once(media_id), iter::empty::<u64>())
        .send()
        .await?;

    dotenv::dotenv().ok();
    let secrets = Secrets::from_env();
    let database = Database::new(&secrets).await;
    database.update_attempt_twitter_url(tweet_object.url().to_string(), draft.call_sid.clone()).await?;

    Ok(())
}

async fn init_video_upload(
    reqwest: ReqwestClient,
    secrets: OauthSecrets<'_>,
    total_bytes: usize,
) -> Result<u64> {

    let init_params = [
        ("command", "INIT"),
        ("media_type", "video/mp4"),
        ("total_bytes", &total_bytes.to_string()),
    ];

    let init_response: MediaInitResponse = reqwest
        .oauth1(secrets)
        .post("https://upload.twitter.com/1.1/media/upload.json")
        .form(&init_params)
        .send()
        .await?
        .json()
        .await?;

    match init_response {
        MediaInitResponse::Success { media_id, .. } => Ok(media_id),
        MediaInitResponse::Error { errors } => {
            let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
            Err(anyhow!("Failed to initialize video upload: {:?}", error_messages))
        }
    }
}

async fn append_video_upload(
    reqwest: ReqwestClient,
    secrets: OauthSecrets<'_>,
    media_id: u64,
    segment_index: usize,
    media: &[u8],
) -> Result<()> {

    let form = Form::new()
        .text("command", "APPEND")
        .text("media_id", media_id.to_string())
        .text("segment_index", segment_index.to_string())
        .part("media", Part::bytes(media.to_vec()));

    reqwest
        .oauth1(secrets)
        .post("https://upload.twitter.com/1.1/media/upload.json")
        .multipart(form)
        .send()
        .await?;

    Ok(())
}

async fn finalize_video_upload(
    reqwest: ReqwestClient,
    secrets: OauthSecrets<'_>,
    media_id: u64,
) -> Result<()> {

    let finalize_params = [("command", "FINALIZE"), ("media_id", &media_id.to_string())];
    reqwest
        .oauth1(secrets)
        .post("https://upload.twitter.com/1.1/media/upload.json")
        .form(&finalize_params)
        .send()
        .await?;

    Ok(())
}

async fn wait_video_upload_successful(
    reqwest: ReqwestClient,
    secrets: OauthSecrets<'_>,
    media_id: u64,
) -> Result<()> {
    let reqwest = reqwest.oauth1(secrets);

    loop {
        let status_params = [("command", "STATUS"), ("media_id", &media_id.to_string())];
        let status_response: MediaStatusResponse = reqwest
            .get("https://upload.twitter.com/1.1/media/upload.json")
            .query(&status_params)
            .send()
            .await?
            .json()
            .await?;

        if let Some(processing_info) = status_response.processing_info {
            if processing_info.state == "succeeded" {
                break;
            } else if processing_info.state == "failed" {
                return Err(anyhow!("Failed to process video"));
            }
        }

        sleep(STATUS_CHECK_INTERVAL).await;
    }

    Ok(())
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum MediaInitResponse {
    Success {
        media_id: u64,
        media_id_string: String,
        size: Option<u64>,
        expires_after_secs: u64,
        video: Option<VideoDetails>,
    },
    Error {
        errors: Vec<ErrorDetail>,
    },
}

#[derive(Deserialize, Debug)]
struct ErrorDetail {
    code: u32,
    message: String,
}

#[derive(Deserialize)]
struct MediaStatusResponse {
    processing_info: Option<MediaProcessingInfo>,
}

#[derive(Deserialize)]
struct MediaProcessingInfo {
    state: String,
}

#[derive(Deserialize, Debug)]
struct VideoDetails {
    video_type: String,
    w: u32,
    h: u32,
}
