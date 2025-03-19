use tokio::process::Command;

pub async fn run_ffmpeg(
    backgrounds_video_path: &str,
    subtitles_path: &str,
    audio_path: &str,
    output_path: &str,
) {
    let showwaves = "[1:a]showwaves=size=400x400:colors=white:draw=full:mode=cline[v]";
    let rounding = "[v]format=rgba,geq='p(mod((2*W/(2*PI))*(PI+atan2(0.5*H-Y,X-W/2)),W),H-2*hypot(0.5*H-Y,X-W/2))':a='1*alpha(mod((2*W/(2*PI))*(PI+atan2(0.5*H-Y,X-W/2)),W),H-2*hypot(0.5*H-Y,X-W/2))'[vout]";
    let overlay = "[0:v][vout]overlay=(W-w)/2:(H-h)/2";
    let subtitles = format!("subtitles={subtitles_path}");
    let pad = "pad=ceil(iw/2)*2:ceil(ih/2)*2[outv]";
    let filter_complex = format!("{showwaves};{rounding};{overlay},{subtitles},{pad}");

    // Render the final video with audio, subtitles and soundwave
    match Command::new("ffmpeg")
        .args(&["-i", &backgrounds_video_path])
        .args(&["-i", &audio_path])
        .args(&["-filter_complex", &filter_complex])
        .args(&["-map", "[outv]"])
        .args(&["-map", "1:a"])
        .args(&["-c:v", "libx264"])
        .args(&["-profile:v", "high"])
        .args(&["-level", "4.1"])
        .args(&["-pix_fmt", "yuv420p"])
        .args(&["-c:a", "aac"])
        .args(&["-r", "30"])
        .arg("-shortest")
        .arg(&output_path)
        .output()
        .await
    {
        Err(e) => log::error!("Failed to create video: {}", e),
        Ok(output) => match output.status.success() {
            true => {
                log::debug!("Successfully created video");
            }
            false => {
                log::error!(
                    "Failed to create video: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        },
    }
}
