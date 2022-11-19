use std::fmt::{Display, Formatter};
use reqwest::{Client, RequestBuilder};
use reqwest::header::{HeaderName, InvalidHeaderName, InvalidHeaderValue};
use serde_json::Value;
use warp::http::HeaderValue;

#[derive(Debug)]
pub struct Error {
    message: String,
    pub code: u8
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

impl Error {
    pub(crate) fn new(message: String, code: u8) -> Self {
        Error { code, message }
    }

    pub(crate) fn new_str(message: &'static str, code: u8) -> Self {
        Error { code, message: String::from(message) }
    }
}


pub struct Requester {
    request_builder: RequestBuilder
}

impl Requester {
    pub fn get_str(url: &'static str) -> Self {
        Self { request_builder: Client::new().get(url) }
    }

    pub fn get(url: String) -> Self {
        Self { request_builder: Client::new().get(url) }
    }

    pub fn post_str(url: &'static str) -> Self {
        Self { request_builder: Client::new().post(url) }
    }

    pub fn form(self, string: &Value) -> Self {
        Self { request_builder: self.request_builder.form(string) }
    }

    pub fn body_str(self, string: &'static str) -> Self {
        Self { request_builder: self.request_builder.body(string) }
    }

    pub fn body(self, string: String) -> Self {
        Self { request_builder: self.request_builder.body(string) }
    }

    pub fn json(self, string: &Value) -> Self {
        Self { request_builder: self.request_builder.json(string) }
    }

    pub fn header(self, name: Result<HeaderName, InvalidHeaderName>, value: Result<HeaderValue, InvalidHeaderValue>) -> Self {
        Self { request_builder: self.request_builder.header(name.unwrap(), value.unwrap()) }
    }

    pub async fn execute(self) -> Result<String, reqwest::Error> {
        self.request_builder.send().await?.text().await
    }
}