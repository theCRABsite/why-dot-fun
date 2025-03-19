mod call;
mod message;
pub mod twiml;
mod webhook;

pub use call::{Call, OutboundCall, Recording};
use headers::HeaderMapExt;
use hyper::body::{Body, Bytes};
use hyper::Response;
pub use message::{Message, OutboundMessage};
use reqwest::{Client as ReqwestClient, Method, StatusCode};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::future::Future;

#[derive(Clone)]
pub struct Client {
    account_id: String,
    auth_token: String,
    client: ReqwestClient,
}

#[derive(Debug)]
pub enum TwilioError {
    ReqwestError(reqwest::Error),
    HTTPError(StatusCode),
    ParsingError,
    AuthError,
    BadRequest,
}

impl Display for TwilioError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            TwilioError::ReqwestError(ref e) => e.fmt(f),
            TwilioError::HTTPError(ref s) => write!(f, "Invalid HTTP status code: {}", s),
            TwilioError::ParsingError => f.write_str("Parsing error"),
            TwilioError::AuthError => f.write_str("Missing `X-Twilio-Signature` header in request"),
            TwilioError::BadRequest => f.write_str("Bad request"),
        }
    }
}

impl Error for TwilioError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            TwilioError::ReqwestError(ref e) => Some(e),
            _ => None,
        }
    }
}

pub trait FromMap {
    fn from_map(m: BTreeMap<String, String>) -> Result<Box<Self>, TwilioError>;
}

impl Client {
    pub fn new(account_id: &str, auth_token: &str) -> Client {
        Client {
            account_id: account_id.to_string(),
            auth_token: auth_token.to_string(),
            client: ReqwestClient::new(),
        }
    }

    async fn send_request<T>(
        &self,
        method: Method,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T, TwilioError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = &format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/{}.json",
            self.account_id, endpoint
        );

        let response = self
            .client
            .request(method, url)
            .basic_auth(&self.account_id, Some(&self.auth_token))
            .form(&params)
            .send()
            .await
            .map_err(TwilioError::ReqwestError)?;

        match response.status() {
            StatusCode::CREATED | StatusCode::OK => {}
            other => {
                let content = response.text().await;
                println!("Error: {:?}", content);

                return Err(TwilioError::HTTPError(other));
            }
        };

        response
            .json::<T>()
            .await
            .map_err(|_| TwilioError::ParsingError)
    }

    pub async fn respond_to_webhook<B, T: FromMap, F>(
        &self,
        req: hyper::Request<B>,
        logic: F,
    ) -> Response<String>
    where
        B: Body<Data = Bytes>,
        F: FnOnce(T) -> twiml::Twiml,
    {
        let o = match self.parse_request::<B, T>(req).await {
            Ok(obj) => *obj,
            Err(_) => return Response::new("Error.".to_string()),
        };

        let twiml = logic(o).as_twiml();
        let mut res = Response::new(twiml);
        res.headers_mut().typed_insert(headers::ContentType::xml());
        res
    }

    pub async fn respond_to_webhook_async<B, T: FromMap, F, Fut>(
        &self,
        req: hyper::Request<B>,
        logic: F,
    ) -> Response<String>
    where
        B: Body<Data = Bytes>,
        F: FnOnce(T) -> Fut,
        Fut: Future<Output = twiml::Twiml>,
    {
        let o = match self.parse_request::<B, T>(req).await {
            Ok(obj) => *obj,
            Err(_) => return Response::new("Error.".to_string()),
        };

        let twiml = logic(o).await.as_twiml();
        let mut res = Response::new(twiml);
        res.headers_mut().typed_insert(headers::ContentType::xml());
        res
    }
}
