use crate::{Client, FromMap, TwilioError};
use reqwest::Method;
use serde::Deserialize;
use std::collections::BTreeMap;

pub struct OutboundCall<'a> {
    pub from: &'a str,
    pub to: &'a str,
    pub url: &'a str,
}

impl<'a> OutboundCall<'a> {
    pub fn new(from: &'a str, to: &'a str, url: &'a str) -> OutboundCall<'a> {
        OutboundCall { from, to, url }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CallStatus {
    Queued,
    Ringing,
    InProgress,
    Canceled,
    Completed,
    Failed,
    Busy,
    NoAnswer,
}

#[derive(Debug, Deserialize)]
pub struct Call {
    pub from: String,
    pub to: String,
    pub sid: String,
    pub status: CallStatus,
    pub speech_confidence: Option<f64>,
    pub speech_result: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Recording {
    pub call_sid: String,
    pub sid: String,
}

impl Client {
    pub async fn make_call(&self, call: OutboundCall<'_>) -> Result<Call, TwilioError> {
        let opts = [
            ("To", &*call.to),
            ("From", &*call.from),
            ("Url", &*call.url),
        ];
        self.send_request(Method::POST, "Calls", &opts).await
    }

    pub async fn update_call_url(&self, sid: &str, url: &str) -> Result<Call, TwilioError> {
        let opts = [("Url", url)];
        self.send_request(Method::POST, &format!("Calls/{sid}"), &opts)
            .await
    }

    pub async fn record_call(&self, sid: &str, callback: &str) -> Result<Recording, TwilioError> {
        let opts = [("RecordingStatusCallback", callback)];
        self.send_request(Method::POST, &format!("Calls/{sid}/Recordings.json"), &opts)
            .await
    }

    pub async fn download_recording(&self, recording_sid: &str) -> Result<Vec<u8>, TwilioError> {
        let url = &format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Recordings/{}.mp3",
            self.account_id, recording_sid
        );
        self.client
            .get(url)
            .basic_auth(&self.account_id, Some(&self.auth_token))
            .send()
            .await
            .map_err(TwilioError::ReqwestError)?
            .bytes()
            .await
            .map_err(TwilioError::ReqwestError)
            .map(|b| b.to_vec())
    }
}

impl FromMap for Call {
    fn from_map(mut m: BTreeMap<String, String>) -> Result<Box<Call>, TwilioError> {
        let from = match m.remove("From") {
            Some(v) => v,
            None => return Err(TwilioError::ParsingError),
        };
        let to = match m.remove("To") {
            Some(v) => v,
            None => return Err(TwilioError::ParsingError),
        };
        let sid = match m.remove("CallSid") {
            Some(v) => v,
            None => return Err(TwilioError::ParsingError),
        };
        let stat = match m.get("CallStatus").map(|s| s.as_str()) {
            Some("queued") => CallStatus::Queued,
            Some("ringing") => CallStatus::Ringing,
            Some("in-progress") => CallStatus::InProgress,
            Some("canceled") => CallStatus::Canceled,
            Some("completed") => CallStatus::Completed,
            Some("failed") => CallStatus::Failed,
            Some("busy") => CallStatus::Busy,
            Some("no-answer") => CallStatus::NoAnswer,
            _ => return Err(TwilioError::ParsingError),
        };

        let speech_confidence = m.remove("Confidence").and_then(|c| c.parse().ok());
        let speech_result = m.remove("SpeechResult");

        Ok(Box::new(Call {
            from,
            to,
            sid,
            status: stat,
            speech_confidence,
            speech_result,
        }))
    }
}

impl FromMap for Recording {
    fn from_map(mut m: BTreeMap<String, String>) -> Result<Box<Recording>, TwilioError> {
        let call_sid = match m.remove("CallSid") {
            Some(v) => v,
            None => return Err(TwilioError::ParsingError),
        };
        let sid = match m.remove("RecordingSid") {
            Some(v) => v,
            None => return Err(TwilioError::ParsingError),
        };

        Ok(Box::new(Recording { call_sid, sid }))
    }
}
