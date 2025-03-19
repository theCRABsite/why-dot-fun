use crate::{
    cache::{CachedCall, CachedMessage},
    secrets::Secrets,
};
use reqwest::{header::CONTENT_TYPE, Client as ReqwestClient};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use tokio::{process::Command, task::JoinSet, time::sleep};

pub async fn download_background_video(
    reqwest: ReqwestClient,
    cached_call: &CachedCall,
    call_sid: &str,
) -> String {
    log::debug!("Downloading sponsor background video for call {call_sid}");

    // Download the sponsor's background video
    let response = reqwest
        .get(&cached_call.sponsor.background_url)
        .send()
        .await
        .expect("Failed to download background video");

    // Try to get the content type of the background video
    let content_type = response.headers().get(CONTENT_TYPE).map(|ct| {
        ct.to_str()
            .expect("Failed to parse content type")
            .to_string()
    });

    let background = response
        .bytes()
        .await
        .expect("Failed to download background image");

    // Determine the extension of the background video by content type
    // or by inferring the content type from the bytes themselves
    let content_type = match content_type {
        Some(content_type) => content_type,
        None => infer::get(&background)
            .expect("Failed to infer content type")
            .mime_type()
            .to_string(),
    };

    let extension = match content_type.as_str() {
        "video/mp4" => "mp4",
        "video/quicktime" => "mov",
        "image/jpeg" => "jpg",
        "image/png" => "png",
        _ => panic!("Unsupported content type: {content_type}"),
    };

    // Write the default background image to disk
    let background_path = format!("cache/recordings/{call_sid}/background.{extension}",);
    tokio::fs::write(&background_path, background)
        .await
        .expect("Failed to write background image");

    background_path
}

pub async fn generate_background_video(
    reqwest: ReqwestClient,
    secrets: &Secrets,
    call_sid: &str,
    cached_messages: &[CachedMessage],
) -> String {
    log::debug!("Generating AI backgrounds for call {call_sid}");

    let concat_path = format!("cache/recordings/{call_sid}/backgrounds.txt",);
    let backgrounds_video_path = format!("cache/recordings/{call_sid}/backgrounds.mp4");
    let mut concat_content = String::new();
    let mut join_set = JoinSet::new();

    // Generate all AI backgrounds in parallel
    for (index, cached_message) in cached_messages.iter().enumerate() {
        let path = format!("cache/recordings/{call_sid}/background_{index}.jpeg");
        join_set.spawn(tokio::spawn(generate_ai_image(
            reqwest.clone(),
            secrets.clone(),
            cached_message.message.clone(),
            path,
        )));

        // Write the background file to the CONCAT file
        let duration = (cached_message.timespan.end - cached_message.timespan.start).as_secs_f32();
        let content = format!("file 'background_{index}.jpeg'\nduration {duration}\n");
        concat_content.push_str(&content);
    }

    // Wait for all AI backgrounds to be generated
    join_set.join_all().await;

    // Write the CONCAT file to disk
    tokio::fs::write(&concat_path, concat_content)
        .await
        .expect("Failed to write backgrounds.txt");

    // Render the background video with dynamic images concatenated
    match Command::new("ffmpeg")
        .args(&["-f", "concat"])
        .args(&["-safe", "0"])
        .args(&["-i", &concat_path])
        .args(&["-c:v", "libx264"])
        .args(&["-r", "30"])
        .arg(&backgrounds_video_path)
        .output()
        .await
    {
        Err(e) => log::error!("Failed to create background video: {}", e),
        Ok(output) => match output.status.success() {
            true => {
                log::debug!("Successfully created background video");
            }
            false => {
                log::error!(
                    "Failed to create background video: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        },
    }

    backgrounds_video_path
}

#[derive(Debug, Deserialize)]
struct TaskResponse {
    task_id: String,
}

#[derive(Debug, Deserialize)]
struct ImagesResponse {
    images: Vec<ImageResponse>,
}

#[derive(Debug, Deserialize)]
struct ImageResponse {
    image_url: String,
}

async fn generate_ai_image(
    reqwest: ReqwestClient,
    secrets: Secrets,
    message: String,
    path: String,
) {

    let full_prompt = format!("{} {}", "8K photo quality. Realistic. ", message);

    let payload = json!({
        "extra": {
            "response_image_type": "jpeg"
        },
        "request": {
            "prompt": full_prompt,
            "model_name": "sd_xl_base_1.0.safetensors",
            "negative_prompt": "nsfw",
            "width": 1024,
            "height": 1024,
            "image_num": 1,
            "steps": 20,
            "seed": -1,
            "clip_skip": 1,
            "sampler_name": "Euler a",
            "guidance_scale": 7.5
        }
    });

    let response = reqwest
        .post("https://api.novita.ai/v3/async/txt2img")
        .bearer_auth(&secrets.novita_api_key)
        .json(&payload)
        .send()
        .await;

    let response = match response {
        Ok(response) => response,
        Err(e) => {
            log::error!("Failed to generate AI background: {e:?}");
            return;
        }
    };

    let task_id = match response.json::<TaskResponse>().await {
        Ok(response) => response.task_id,
        Err(e) => {
            log::error!("Failed to parse task ID: {e:?}");
            return;
        }
    };

    for _ in 0..10 {
        sleep(Duration::from_secs(5)).await;
        let response = reqwest
            .get(&format!(
                "https://api.novita.ai/v3/async/task-result?task_id={task_id}"
            ))
            .bearer_auth(&secrets.novita_api_key)
            .send()
            .await;

        let response = match response {
            Ok(response) => response,
            Err(e) => {
                log::error!("Failed to get task result: {e:?}");
                continue;
            }
        };

        let image_url = match response.json::<ImagesResponse>().await {
            Ok(response) => response.images.first().map(|image| image.image_url.clone()),
            Err(e) => {
                log::error!("Failed to parse image URL: {e:?}");
                continue;
            }
        };

        let image_url = match image_url {
            Some(image_url) => image_url,
            None => continue,
        };

        let response = reqwest.get(&image_url).send().await;

        let bytes = match response {
            Ok(response) => response.bytes().await,
            Err(e) => {
                log::error!("Failed to download image: {e:?}");
                continue;
            }
        };

        let bytes = match bytes {
            Ok(bytes) => bytes,
            Err(e) => {
                log::error!("Failed to get image bytes: {e:?}");
                continue;
            }
        };

        let _ = tokio::fs::write(&path, bytes).await;

        return;
    }

    log::error!("Failed to poll AI background after 5 attempts");
}
