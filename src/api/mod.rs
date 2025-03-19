pub mod attempt_list;
pub mod attempt_single;
pub mod launchpad;
pub mod payment;
pub mod verify_winner;
pub mod deposit;
pub mod activate_sponsor;
pub mod sponsor_list;
pub mod update_sponsor;

use chrono::Utc;
use serde::{Serialize, Deserialize};
use crate::api::launchpad::ReturnSponsor;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attempt {
    // id of the attempt
    pub id: i32,
    // public key of the attempt
    pub pubkey: Option<String>,
    // phone number of the user
    pub phone_number: String,
    // attempt created at
    pub created_at: chrono::DateTime<Utc>,
    // attempt updated at
    pub updated_at: chrono::DateTime<Utc>,
    // video url of attempt
    pub video_url: Option<String>,
    // twitter url of attempt
    pub twitter_url: Option<String>,
    // is the attempt a winner
    pub is_winner: Option<bool>,
    // sponsored question / challenge
    pub sponsor_question: Option<String>,
    // name of the sponsor
    pub sponsor_name: Option<String>,
    // sponsored token mint
    pub sponsor_token_mint: Option<String>,
    // sponsored total reward
    pub sponsor_total_reward: Option<i64>,
    // sponsored reward per attempt
    pub sponsor_attempt_reward: Option<i64>,
    // background url of the sponsor image or video
    pub sponsor_background_url: Option<String>,
    // time user has to complete the challenge
    pub sponsor_challenge_time: Option<i32>,
    // transcript of the challenge
    pub challenge_transcript: Option<String>,
    // status of the challenge
    pub challenge_status: Option<String>,
    // url of the winner
    pub winner_url: String,
    // call sid of the call
    pub call_sid: String,
} 


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptReturn {
    pub id: i32,
    // attempt created at
    pub created_at: chrono::DateTime<Utc>,
    // attempt updated at
    pub updated_at: chrono::DateTime<Utc>,
    // video url of attempt
    pub video_url: Option<String>,
    // twitter url of attempt
    pub twitter_url: Option<String>,
    // is the attempt a winner
    pub is_winner: Option<bool>,
    // sponsored question / challenge
    pub sponsor_question: Option<String>,
    // name of the sponsor
    pub sponsor_name: Option<String>,
    // sponsored token mint
    pub sponsor_token_mint: Option<String>,
    // background url of the sponsor image or video
    pub sponsor_background_url: Option<String>,
    // time user has to complete the challenge
    pub sponsor_challenge_time: Option<i32>,
    // transcript of the challenge
    pub challenge_transcript: Option<String>,
    // status of the challenge
    pub challenge_status: Option<String>,
} 


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SponsorArgs {
    pub name: String,
    pub user_id: String,
    pub background_url: String,
    pub token_mint: String,
    pub original_tokens: i64,
    pub reward_tokens: i64,
    pub challenge_time: i32,
    pub system_instruction: String,
    pub challenge: String,
    pub rating_threshold: i32,
    pub transaction: String,
}


#[derive(Serialize)]
pub struct ResponseData {
    sponsor: ReturnSponsor,
    signature: String,
}