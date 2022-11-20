use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::mpsc;
use std::time::Duration;
use rand::distributions::Alphanumeric;
use rand::Rng;
use reqwest::header::HeaderName;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::spawn;
use uuid::Uuid;
use webbrowser::open;
use warp::Filter;
use warp::http::HeaderValue;
use crate::auth::Session;
use crate::web::{Requester, Error};
use crate::auth::microsoft::internals::{RawAccessToken, RawSession};

mod internals;

#[derive(Debug)]
#[derive(Deserialize)]
struct Query {
    code: String,
    state: String
}

fn random_string() -> String {
    rand::thread_rng().sample_iter(Alphanumeric).take(15).map(char::from).collect()
}

#[derive(Debug, Clone)]
pub struct AccessToken {
    pub access_token: String,
    pub expires_in: Duration,
    pub token_type: String
}

pub struct MicrosoftAuthenticator<'a> {
    pub client_id: &'a str,
    pub port: u16,
    refresh_token: Option<String>
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct AuthToken {
    pub token: String,
    pub user_hash: String,
    pub token_type: TokenType
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum TokenType {
    XSLS,
    User
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum MinecraftEdition {
    Java,
    Bedrock
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum XSTSErrorType {
    NoXboxAccount,
    XboxBannedOrNotAvailable,
    NeedsAdultVerification,
    AccountIsChild
}

impl Display for XSTSErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            XSTSErrorType::AccountIsChild => write!(f, "The account is a child (under 18)"),
            XSTSErrorType::NeedsAdultVerification => write!(f, "The account needs adult verification on XboxPage (South Korea)"),
            XSTSErrorType::XboxBannedOrNotAvailable => write!(f, "The account is from a country where Xbox Live is not available/banned"),
            XSTSErrorType::NoXboxAccount => write!(f, "The account doesn't have a Xbox account. Once they sign up for one then they can proceed with the login")
        }
    }
}

impl XSTSErrorType {
    pub fn from_u64(value: u64) -> Self {
        match value {
            2148916233 => Self::NoXboxAccount,
            2148916235 => Self::XboxBannedOrNotAvailable,
            2148916236 | 2148916237 => Self::NeedsAdultVerification,
            2148916238 => Self::AccountIsChild,
            _ => panic!("Got illegal error {} from XSTS Token Endpoint", value)
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct XSTSTokenError {
    pub identity: u16,
    pub error_code: u64,
    pub error_type: XSTSErrorType,
    pub redirect: String,
    pub message: String
}

impl Display for XSTSTokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}) => {}", self.error_type, self.error_code, self.redirect)
    }
}

#[derive(Debug)]
pub struct XSTSError {
    token_error: Option<XSTSTokenError>,
    error_text: Option<String>,
    pub error_code: Option<u8>
}

impl Display for XSTSError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.token_error.clone().is_none() {
            write!(f, "{}", self.token_error.clone().unwrap())
        } else {
            write!(f, "{}", self.error_text.clone().unwrap())
        }
    }
}

impl XSTSError {

    pub fn token_error(token_error: XSTSTokenError) -> Self {
        Self { token_error: Some(token_error), error_code: None, error_text: None }
    }

    pub fn normal(text: String, code: u8) -> Self {
        Self { token_error: None, error_code: Some(code), error_text: Some(text) }
    }

    pub fn to_error(&self) -> Result<Error, ()> {
        if self.token_error.is_some() {
            return Err(());
        }

        Ok(Error::new(self.error_text.clone().unwrap(), self.error_code.unwrap()))
    }

}

impl std::error::Error for XSTSError {}

impl<'a> MicrosoftAuthenticator<'a> {

    pub fn new(client_id: &'a str, port: u16) -> Self {
        Self { client_id, port, refresh_token: None }
    }

    pub async fn request_refresh_token(&mut self) -> Result<String, Error> {
        let state = random_string();

        match open(&format!("https://login.live.com/oauth20_authorize.srf?client_id={}&response_type=code&redirect_uri=http://127.0.0.1:{}\
        &scope=XboxLive.signin%20offline_access&state={}&prompt=select_account", self.client_id, self.port, state)) {
            Ok(_) => {}
            Err(error) => return Err(Error::new(format!("Unable to prompt refresh token login => {}", error.to_string()), 1))
        }

        let query = Self::start_oauth_server(self.port).await;
        if query.state != state {
            return Err(Error::new(format!("Unable to request the refresh token => Illegal response code {} ({} != {})", query.state, query.state, state), 2));
        }

        self.refresh_token = Some(query.code);
        Ok(self.refresh_token.clone().unwrap())
    }

    pub async fn request_access_token(&mut self) -> Result<AccessToken, Error> {
        if self.refresh_token.is_none() {
            self.request_refresh_token().await?;
        }

        let query = json!({
            "client_id": self.client_id,
            "code": self.refresh_token,
            "grant_type": "authorization_code",
            "redirect_uri": format!("http://127.0.0.1:{}", self.port)
        });

        let token = Requester::post_str("https://login.live.com/oauth20_token.srf")
            .form(&query).execute().await;
        if token.is_err() {
            return Err(Error::new(format!("Unable to get access token => {}", token.err().unwrap()), 3));
        }

        let token: serde_json::error::Result<RawAccessToken> = serde_json::from_str(&token.unwrap());
        if token.is_err() {
            return Err(Error::new(format!("Unable to parse access token => {}", token.err().unwrap()), 4));
        }

        let token = token.unwrap();
        Ok(AccessToken {
            access_token: token.access_token,
            token_type: token.token_type,
            expires_in: Duration::from_secs(token.expires_in as u64)
        })
    }

    pub async fn authenticate(&self, access_token: AccessToken) -> Result<AuthToken, Error> {
        let json = json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={}", access_token.access_token)
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        });

        let requester = Requester::post_str("https://user.auth.xboxlive.com/user/authenticate")
            .json(&json).execute().await;
        if requester.is_err() {
            return Err(Error::new(format!("Unable to authenticate => {}", requester.err().unwrap()), 5));
        }

        let json: serde_json::error::Result<Value> = serde_json::from_str(&requester.unwrap());
        if json.is_err() {
            return Err(Error::new(format!("Unable to parse auth response => {}", json.err().unwrap()), 6));
        }
        let json = json.unwrap();

        Ok(AuthToken {
            token: json["Token"].to_string().replace("\"", ""),
            user_hash: json["DisplayClaims"]["xui"][0]["uhs"].to_string().replace("\"", ""),
            token_type: TokenType::User
        })
    }

    // TODO: Add support for:
    // 2148916233            - No Xbox Account found
    // 2148916235            - Country where Xbox Service unavailable/banned
    // 2148916236/2148916237 - Need adult verification on Xbox page (South Korea)
    // 2148916238            - Account is from a child
    // Error format:
    // {
    //    "Identity": "0",
    //    "XErr": 2148916238,
    //    "Message": "",
    //    "Redirect: "https://start.ui.xboxlive.com/AddChildToFamily"
    pub async fn request_xsts_token(&self, auth_token: AuthToken, edition: MinecraftEdition) -> Result<AuthToken, XSTSError> {
        let json = json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [
                    auth_token.token
                ]
            },
            "RelyingParty": if edition == MinecraftEdition::Java {
                "rp://api.minecraftservices.com/"
            } else {
                "https://pocket.realms.minecraft.net/"
            },
            "TokenType": "JWT"
        });

        let requester = Requester::post_str("https://xsts.auth.xboxlive.com/xsts/authorize")
            .json(&json).execute().await;
        if requester.is_err() {
            return Err(XSTSError::normal(format!("Unable to authenticate => {}", requester.err().unwrap()), 7));
        }

        let json: serde_json::error::Result<Value> = serde_json::from_str(&requester.unwrap());
        if json.is_err() {
            return Err(XSTSError::normal(format!("Unable to parse auth response => {}", json.err().unwrap()), 8));
        }
        let json = json.unwrap();
        if json.get("Token").is_none() {
            return Err(XSTSError::token_error(XSTSTokenError {
                error_code: json["XErr"].as_u64().unwrap(),
                error_type: XSTSErrorType::from_u64(json["XErr"].as_u64().unwrap()),
                redirect: json["Redirect"].to_string(),
                identity: str::parse::<u16>(&json["Identity"].to_string()).unwrap(),
                message: json["Message"].to_string()
            }));
        }

        Ok(AuthToken {
            token: json["Token"].to_string().replace("\"", ""),
            user_hash: json["DisplayClaims"]["xui"][0]["uhs"].to_string().replace("\"", ""),
            token_type: TokenType::XSLS
        })
    }

    pub async fn authenticate_minecraft(auth_token: AuthToken) -> Result<Session, Error> {
        if auth_token.token_type != TokenType::XSLS {
            return Err(Error::new("Unable to authenticate with Minecraft => The specified token isn't a XSLS token".to_string(), 7));
        }

        let json = json!({
            "identityToken": format!("XBL3.0 x={};{}", auth_token.user_hash, auth_token.token)
        });

        let requester = Requester::post_str("https://api.minecraftservices.com/authentication/login_with_xbox")
            .json(&json).execute().await;
        if requester.is_err() {
            return Err(Error::new(format!("Unable to authenticate => {}", requester.err().unwrap()), 9));
        }

        let session: serde_json::error::Result<RawSession> = serde_json::from_str(&requester.unwrap());
        if session.is_err() {
            return Err(Error::new(format!("Unable to parse access token => {}", session.err().unwrap()), 10));
        }
        let session = session.unwrap();

        Ok(Session {
            token_type: crate::auth::TokenType::from_str(&session.token_type),
            username: Uuid::from_str(&session.username).unwrap(),
            expires_in: Duration::from_secs(session.expires_in as u64),
            roles: session.roles,
            access_token: session.access_token
        })
    }

    pub async fn has_minecraft(session: Session) -> Result<bool, Error> {
        let requester = Requester::get_str("https://api.minecraftservices.com/entitlements/mcstore")
            .header(HeaderName::from_str("Authorization"), HeaderValue::from_str(&format!("Bearer {}", session.access_token)))
            .execute().await;
        if requester.is_err() {
            return Err(Error::new(format!("Unable to authenticate => {}", requester.err().unwrap()), 11));
        }

        let json: serde_json::error::Result<Value> = serde_json::from_str(&requester.unwrap());
        if json.is_err() {
            return Err(Error::new(format!("Unable to parse auth response => {}", json.err().unwrap()), 12));
        }
        let json = json.unwrap();
        match &json["items"] {
            Value::Array(values) => {
                Ok(values.len() > 0)
            },
            _ => Err(Error::new("Items array isn't a array".to_string(), 13))
        }
    }

    async fn start_oauth_server(port: u16) -> Query {
        let (sender, receiver) = mpsc::sync_channel(14);
        let route = warp::get()
            .and(warp::filters::query::query())
            .map(move |query: Query| {
                sender.send(query).expect("Unable to send query through sender");
                "Successfully received query"
            });

        spawn(warp::serve(route).run(([127, 0, 0, 1], port)));
        receiver.recv().expect("Channel has hang up")
    }

}