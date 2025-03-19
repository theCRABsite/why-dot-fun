use crate::cache::{CachedCall, CachedMessage};
use srtlib::{Subtitle, Subtitles, Timestamp};

/// Collect all user and assistant messages together in a SRT file,
/// which can be used to display subtitles with correct and accurate
/// timestamps under the video recording of the call.
pub fn generate_subtitles_srt(
    cached_messages: &[CachedMessage],
    cached_call: &CachedCall,
    subtitles_path: &str,
) {
    // Build the subtitles from the cached messages
    let mut subtitles = Subtitles::new();
    for (index, cached_message) in cached_messages.iter().enumerate() {
        let start = Timestamp::from_milliseconds(
            (cached_message.timespan.start - cached_call.start).as_millis() as _,
        );

        let end = Timestamp::from_milliseconds(
            (cached_message.timespan.end - cached_call.start).as_millis() as _,
        );

        subtitles.push(Subtitle::new(
            index,
            start,
            end,
            cached_message.message.clone(),
        ));
    }

    // Write the subtitles to a SRT file using UTF-8 encoding
    if let Err(e) = subtitles.write_to_file(subtitles_path, None) {
        log::error!("Failed to write subtitles to file: {e:?}");
    }
}
