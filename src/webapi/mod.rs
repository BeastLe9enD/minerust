use std::str::FromStr;
use reqwest::header::HeaderName;
use serde_json::Value;
use uuid::Uuid;
use crate::web::{Error, Requester};
use serde::Deserialize;
use warp::http::HeaderValue;

#[derive(Deserialize, Debug, Clone)]
pub struct ProfileResponse {
    pub id: String,
    pub name: String,
    pub properties: Vec<Property>
}

#[derive(Deserialize, Debug, Clone)]
pub struct Property {
    pub name: String,
    pub value: String,
    pub signature: Option<String>
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct PlayerAttributes {
    pub privileges: Vec<Privilege>,
    ban_status: Option<BanStatus>
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Privilege {
    pub name: &'static str,
    pub enabled: bool
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct BanStatus {
    ban_id: String,
    expires: i64,
    reason: BanReason,
    reason_message: Option<String>
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum BanReason {
    FalseReporting,
    HateSpeech,
    TerrorismOrViolentExtremism,
    ChildSexualAbuse,
    ImminentHarm,
    NonConsensualIntimateImagery,
    HarassmentOrBullying,
    DefamationImpersonationFalseInformation,
    SelfHarmOrSuicide,
    AlcoholTobaccoDrugs
}

impl BanReason {
    pub fn from_string(string: String) -> BanReason {
        match string.as_str() {
            "false_reporting" => BanReason::FalseReporting,
            "hate_speech" => BanReason::HateSpeech,
            "terrorism_or_violent_extremism" => BanReason::TerrorismOrViolentExtremism,
            "child_sexual_exploitation_or_abuse" => BanReason::ChildSexualAbuse,
            "imminent_harm" => BanReason::ImminentHarm,
            "non_consensual_intimate_imagery" => BanReason::NonConsensualIntimateImagery,
            "harassment_or_bullying" => BanReason::HarassmentOrBullying,
            "defamation_impersonation_false_information" => BanReason::DefamationImpersonationFalseInformation,
            "self_harm_or_suicide" => BanReason::SelfHarmOrSuicide,
            "alcohol_tobacco_drugs" => BanReason::AlcoholTobaccoDrugs,
            _ => panic!("{}", format!("{} is not valid!", string))
        }
    }
}

pub async fn uuid_from_username(username: &'static str) -> Result<Uuid, Error> {
    let response = Requester::get(format!("https://api.mojang.com/users/profiles/minecraft/{}", username.clone())).execute().await;
    if response.is_err() {
        return Err(Error::new(format!("Unable to send uuid2username request => {}", response.err().unwrap()), 15));
    }
    let response = response.unwrap();
    if response.is_empty() {
        return Err(Error::new(format!("The user {} doesn't exists!", username), 15));
    }

    Ok(Uuid::from_str(serde_json::from_str::<Value>(&response).expect("Unable to parse response")["id"]
        .as_str().expect("Unable to find id object")).expect("Unable to create uuid"))
}

pub async fn profile_from_uuid(uuid: Uuid) -> Result<ProfileResponse, Error> {
    let response = Requester::get(format!("https://sessionserver.mojang.com/session/minecraft/profile/{}", uuid)).execute().await;
    if response.is_err() {
        return Err(Error::new(format!("Unable to send uuid to profile request => {}", response.err().unwrap()), 15));
    }
    let response = response.unwrap();

    let response = serde_json::from_str::<ProfileResponse>(&response);
    if response.is_err() {
        return Err(Error::new(format!("Unable to parse response"), 16));
    }

    Ok(response.unwrap())
}

pub async fn blocked_servers() -> Result<Vec<String>, Error> {
    let response = Requester::get_str("https://sessionserver.mojang.com/blockedservers").execute().await;
    if response.is_err() {
        return Err(Error::new(format!("Unable to send uuid to profile request => {}", response.err().unwrap()), 17));
    }
    let response = response.unwrap();

    let mut blocked_servers = Vec::new();
    for hash in response.split("\n") {
        blocked_servers.push(hash.to_string());
    }
    Ok(blocked_servers)
}

pub async fn player_attributes(access_token: String) -> Result<PlayerAttributes, Error> {
    let response = Requester::get_str("https://api.minecraftservices.com/player/attributes")
        .header(HeaderName::from_str("Authentication"), HeaderValue::from_str(&format!("Bearer {}", access_token)))
        .execute().await;
    if response.is_err() {
        return Err(Error::new(format!("Unable to send player attributes request => {}", response.err().unwrap()), 18));
    }
    let response = response.unwrap();

    println!("{}", response.clone());

    let response = serde_json::from_str::<Value>(&response);
    if response.is_err() {
        return Err(Error::new_str("Unable to parse response from player attributes endpoint", 19));
    }
    let response = response.unwrap();

    let online_chat: Privilege = Privilege { name: "onlineChat", enabled: response["privileges"]["onlineChat"]["enabled"].as_bool().unwrap() };
    let multiplayer_server: Privilege = Privilege { name: "multiplayerServer", enabled: response["privileges"]["multiplayerServer"]["enabled"].as_bool().unwrap() };
    let multiplayer_realms: Privilege = Privilege { name: "multiplayerRealms", enabled: response["privileges"]["multiplayerRealms"]["enabled"].as_bool().unwrap() };
    let telemetry: Privilege = Privilege { name: "telemetry", enabled: response["privileges"]["telemetry"]["enabled"].as_bool().unwrap() };

    let mut ban_status = None;
    if response.get("banStatus").is_some() && response.get("bannedScopes").is_some() && response.get("MULTIPLAYER").is_some() {
        let scope = &response["banStatus"]["bannedScopes"]["MULTIPLAYER"];
        ban_status = Some(BanStatus {
            reason: BanReason::from_string(scope["reason"].to_string()),
            ban_id: scope["banId"].to_string(),
            expires: scope["expires"].as_i64().unwrap(),
            reason_message: if scope.get("reasonMessage").is_some() {
                Some(scope.get("reasonMessage").unwrap().to_string())
            } else {
                None
            }
        });
    }

    Ok(PlayerAttributes {
        ban_status,
        privileges: vec![online_chat, multiplayer_server, multiplayer_realms, telemetry]
    })
}