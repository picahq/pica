use crate::context::TestServer;
use http::{Method, StatusCode};
use osentities::{
    emitted_events::ConnectionLostReason, environment::Environment, prefix::IdPrefix, Connection,
    Id,
};
use serde_json::Value;

#[tokio::test]
async fn test_database_connection_lost_callback() {
    let mut server = TestServer::new(None).await;

    let (mut connection, _) = server.create_connection(Environment::Live).await;
    connection.group = server.live_access_key.data.group.clone();

    let connection_id = connection.id.to_string();

    let path = format!("v1/event-callbacks/database-connection-lost/{connection_id}");
    let reason = ConnectionLostReason {
        reason: "database-connection-lost".to_string(),
    };

    let request = server
        .send_request::<ConnectionLostReason, Connection>(&path, Method::POST, None, Some(&reason))
        .await
        .expect("Failed to send request");

    assert_eq!(request.code, StatusCode::OK);
    assert!(request.data.record_metadata.deprecated);
    assert!(!request.data.record_metadata.deleted);
    assert!(!request.data.record_metadata.active);
}

#[tokio::test]
async fn test_database_connection_lost_callback_404() {
    let server = TestServer::new(None).await;

    let connection_id = Id::now(IdPrefix::Connection).to_string();
    let path = format!("v1/event-callbacks/database-connection-lost/{connection_id}");
    let reason = ConnectionLostReason {
        reason: "database-connection-lost".to_string(),
    };

    let request = server
        .send_request::<ConnectionLostReason, Value>(&path, Method::POST, None, Some(&reason))
        .await
        .expect("Failed to send request");

    assert_eq!(request.code, StatusCode::NOT_FOUND);
}
