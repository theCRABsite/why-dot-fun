use std::env::var;

#[derive(Debug, Clone)]
pub struct Secrets {
    pub global_url: String,
    pub database_url: String,
    pub twilio_phone_number: String,
    pub twilio_app_sid: String,
    pub twilio_account_sid: String,
    pub twilio_api_key: String,
    pub twilio_api_secret: String,
    pub twilio_auth_token: String,
    pub novita_api_key: String,
    pub review_token: String,
    pub twitter_api_key: String,
    pub twitter_api_secret: String,
    pub twitter_access_token: String,
    pub twitter_access_secret: String,
    pub rpc_url: String,
    pub spaces_secret_key: String,
    pub spaces_access_key: String,
    pub spaces_url: String,
    pub treasury_private_key: String,
    pub treasury_public_key: String,
}

impl Secrets {
    pub fn from_env() -> Self {
        Self {
            global_url: var("GLOBAL_URL").expect("GLOBAL_URL must be set"),
            database_url: var("DATABASE_URL").expect("DATABASE_URL must be set"),
            twilio_phone_number: var("TWILIO_PHONE_NUMBER")
                .expect("TWILIO_PHONE_NUMBER must be set"),
            twilio_app_sid: var("TWILIO_APP_SID").expect("TWILIO_APP_SID must be set"),
            twilio_account_sid: var("TWILIO_ACCOUNT_SID").expect("TWILIO_ACCOUNT_SID must be set"),
            twilio_api_key: var("TWILIO_API_KEY").expect("TWILIO_API_KEY must be set"),
            twilio_api_secret: var("TWILIO_API_SECRET").expect("TWILIO_API_SECRET must be set"),
            twilio_auth_token: var("TWILIO_AUTH_TOKEN").expect("TWILIO_AUTH_TOKEN must be set"),
            novita_api_key: var("NOVITA_API_KEY").expect("NOVITA_API_KEY must be set"),
            review_token: var("REVIEW_TOKEN").expect("REVIEW_TOKEN must be set"),
            twitter_api_key: var("TWITTER_API_KEY").expect("TWITTER_API_KEY must be set"),
            twitter_api_secret: var("TWITTER_API_SECRET").expect("TWITTER_API_SECRET must be set"),
            twitter_access_token: var("TWITTER_ACCESS_TOKEN")
                .expect("TWITTER_ACCESS_TOKEN must be set"),
            twitter_access_secret: var("TWITTER_ACCESS_SECRET")
                .expect("TWITTER_ACCESS_SECRET must be set"),
            rpc_url: var("RPC_URL").expect("RPC_URL must be set"),
            spaces_secret_key: var("SPACES_SECRET_KEY").expect("SPACES_SECRET_KEY must be set"),
            spaces_access_key: var("SPACES_ACCESS_KEY").expect("SPACES_ACCESS_KEY must be set"),
            spaces_url: var("SPACES_URL").expect("SPACES_URL must be set"),
            treasury_private_key: var("TREASURY_PRIVATE_KEY").expect("TREASURY_PRIVATE_KEY must be set"),
            treasury_public_key: var("TREASURY_PUBLIC_KEY").expect("TREASURY_PUBLIC_KEY must be set"),
        }
    }
}
