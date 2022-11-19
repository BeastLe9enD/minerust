use std::time::Duration;
use uuid::Uuid;

// TODO: Implement legacy authentication with Mojang
// TODO: Create a facade for all authentications

pub mod microsoft;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Session {
    pub username: Uuid,
    pub roles: Vec<String>,
    pub access_token: String,
    pub token_type: TokenType,
    pub expires_in: Duration
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum TokenType {
    Bearer
}

impl TokenType {
    pub fn from_str(str: &str) -> TokenType {
        match str {
            "Bearer" => TokenType::Bearer,
            _ => panic!("{}", format!("Illegal token {}", str))
        }
    }
}