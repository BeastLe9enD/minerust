use crate::webapi::{blocked_servers, uuid_from_username};

#[tokio::test]
async fn test_username_to_uuid_valid() {
    let uuid = uuid_from_username("Cach30verfl0w").await.expect("Unable to get uuid from name");
    assert_eq!(uuid.to_string(), "abe18c25-73dc-4f18-8638-adb604cb1d03");
}

#[tokio::test]
async fn test_username_to_uuid_invalid() {
    let uuid = uuid_from_username("NotExistingPlayer").await;
    assert_eq!(uuid.err().unwrap().code, 15);
}

#[tokio::test]
async fn test_blocked_servers() {
    let blocked_servers = blocked_servers().await;
    assert_eq!(blocked_servers.unwrap().len(), 2295);
}
