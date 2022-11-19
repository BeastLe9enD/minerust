use serde::Deserialize;

#[derive(Deserialize)]
pub struct RawAccessToken {
    pub access_token: String,
    pub expires_in: u16,
    pub token_type: String
}

#[derive(Deserialize)]
pub struct RawSession {
    pub username: String,
    pub roles: Vec<String>,
    pub token_type: String,
    pub expires_in: u32,
    pub access_token: String
}