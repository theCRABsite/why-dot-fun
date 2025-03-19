use crate::{cache::CachedCall, secrets::Secrets, CONFIG};
use background::{download_background_video, generate_background_video};
use ffmpeg::run_ffmpeg;
use reqwest::Client as ReqwestClient;
use std::time::Duration;
use subtitles::generate_subtitles_srt;
use tokio::time::{sleep, timeout};
use axum::Error;
use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use aws_sdk_s3::types::ObjectCannedAcl;
use crate::Database;
use reqwest::header::COOKIE;


mod background;
mod ffmpeg;
mod subtitles;

pub async fn render_video(
    reqwest: ReqwestClient,
    secrets: Secrets,
    call_sid: String,
    cached_call: CachedCall,
    rating: u8,
    database: Database,
    judgement: String,
) {
    // Ensure the necessary directories exist
    let _ = tokio::fs::create_dir_all(format!("cache/recordings/{call_sid}")).await;
    let _ = tokio::fs::create_dir_all("cache/drafts").await;

    let audio_path = format!("cache/recordings/{call_sid}/audio.mp3");

    // Wait at most N seconds for the recording to be downloaded
    let duration = Duration::from_secs(CONFIG.settings.recording_timeout as u64);
    if timeout(duration, wait_for_recording(&audio_path))
        .await
        .is_err()
    {
        log::error!("Recording for call {call_sid} timed out, video rendering aborted");
        return;
    }

    // Get all of the messages from the call
    let cached_messages = cached_call.get_cached_messages();

    // List all needed files
    let subtitles_path = format!("cache/recordings/{call_sid}/subtitles.srt");
    let comment_path = format!("cache/recordings/{call_sid}/comment.txt");
    let output_path = format!("cache/drafts/{call_sid}.mp4");

    // Generate the SRT subtitles file
    generate_subtitles_srt(&cached_messages, &cached_call, &subtitles_path);

    // Generate a background video or download it depending on the rating
    let background_video_path = match rating >= cached_call.sponsor.rating_threshold as u8 {
        true => generate_background_video(reqwest.clone(), &secrets, &call_sid, &cached_messages).await,
        false => download_background_video(reqwest.clone(), &cached_call, &call_sid).await,
    };

    let attempt = database
        .get_attempt_by_sid(call_sid.clone())
        .await
        .expect("Failed to get attempt by sid")
        .expect("Attempt not found");

    // Store the comment for the reviewer
    let comment = format!("{} Sponsored by {}.", attempt.challenge_status.unwrap_or("".to_string()), cached_call.sponsor.name);
    tokio::fs::write(&comment_path, comment)
        .await
        .expect("Failed to write comment");

    // Render the video using ffmpeg
    run_ffmpeg(
        &background_video_path,
        &subtitles_path,
        &audio_path,
        &output_path,
    )
    .await;


    let file_name = format!("cache/drafts/{call_sid}.mp4");
    let bucket_name = String::from("gamecall");
    let key = format!("{call_sid}.mp4"); // in aws s3 a key = filename
    let spaces_url = secrets.spaces_url;

    // note here that the "None" is in place of a session token
    let creds = Credentials::new(
        secrets.spaces_access_key, 
        secrets.spaces_secret_key, 
        None, 
        None, 
        "digitalocean"
    );

    let cfg = aws_config::from_env()
        .endpoint_url(spaces_url)
        .region(Region::new("us-east-1"))
        .credentials_provider(creds)
        .load().await;

    let s3 = Client::new(&cfg);

    let body = aws_sdk_s3::primitives::ByteStream::from_path(std::path::Path::new(&file_name))
        .await
        .expect("Failed to read file");

    let _result = s3
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(body)
        .acl(ObjectCannedAcl::PublicRead)
        .content_type("video/mp4")
        .send()
        .await
        .map_err(|e| Error::new(e));


        // Create a form with the draft
        let form = [
            ("call_sid", call_sid.clone()),
            ("comment", judgement.clone()),
        ];
    

    if rating >= 6 {

        // Make the HTTP POST request to the approve_draft endpoint with the form
        let response = reqwest.post("https://gamecall-jvp99.ondigitalocean.app/review/approve")
            .header(COOKIE, format!("review_token={}", secrets.review_token))
            .form(&form)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                log::debug!("Draft approved successfully");
            }
            Ok(resp) => {
                log::error!("Failed to approve draft: {:?}", resp.text().await);
            }
            Err(e) => {
                log::error!("Error making request to approve draft: {:?}", e);
            }
        }

    }


}

/// Waits for the recording to be received by the twilio webhook.
/// This is necessary because the recording is not always available immediately.
async fn wait_for_recording(path: &str) {
    loop {
        match tokio::fs::try_exists(path).await {
            Ok(true) => break,
            _ => sleep(Duration::from_secs(1)).await,
        }
    }
}
