use axum::{extract::Request, response::IntoResponse, Extension};
use twilio::{twiml::Twiml, Client as TwilioClient, Recording};

pub async fn recording_handler(
    twilio: Extension<TwilioClient>,
    request: Request,
) -> impl IntoResponse {
    twilio
        .clone()
        .respond_to_webhook_async(request, |recording: Recording| async move {
            log::debug!(
                "Received recording for call {} with id {}",
                recording.call_sid,
                recording.sid
            );

            // Download the recording
            let mp3 = twilio
                .download_recording(&recording.sid)
                .await
                .expect("Failed to download recording");

            let audio_path = format!("cache/recordings/{}/audio.mp3", recording.call_sid);

            // Ensure the necessary directory exists
            let _ =
                tokio::fs::create_dir_all(format!("cache/recordings/{}", recording.call_sid)).await;

            // Write the audio to disk
            tokio::fs::write(&audio_path, mp3)
                .await
                .expect("Failed to write recording");

            Twiml::new()
        })
        .await
}
