use crate::context::TestServer;
use http::header::{ACCEPT, CONTENT_LENGTH, CONTENT_TYPE, HOST};
use mockito::Server as MockServer;
use osentities::{emitted_events::ConnectionLostReason, prefix::IdPrefix, Id, PicaError, Unit};
use std::collections::HashMap;

#[tokio::test]
async fn test_kill_signal() -> Result<Unit, PicaError> {
    let mut mock_server = MockServer::new_async().await;
    let mock_uri = mock_server.url();

    let connection_id = Id::now(IdPrefix::Connection);

    let path = format!("/v1/admin/connection/{connection_id}");
    let secret_req = mock_server
        .mock("GET", path.as_str())
        .with_status(200)
        .create_async()
        .await;

    let path = format!("/v1/event-callbacks/database-connection-lost/{connection_id}");
    let body = ConnectionLostReason {
        reason: "Deserialization error: Failed to deserialize secret: error decoding response body"
            .to_string(),
    };
    let body = serde_json::to_string(&body).expect("Failed to serialize body");

    let callback_req = mock_server
        .mock("POST", path.as_str())
        .match_header(CONTENT_TYPE, "application/json")
        .match_header(ACCEPT, "*/*")
        .match_header(HOST, mock_server.host_with_port().as_str())
        .match_header(CONTENT_LENGTH, body.to_string().len().to_string().as_str())
        .match_body(&*body.to_string())
        .with_status(200)
        .create_async()
        .await;

    let _ = TestServer::new(HashMap::from([
        ("CONNECTION_ID".to_string(), connection_id.to_string()),
        ("POSTGRES_PASSWORD".to_string(), "wrongpass".to_string()),
        ("CONNECTIONS_URL".to_string(), mock_uri),
    ]))
    .await;

    secret_req.expect(1).assert_async().await;
    callback_req.expect(1).assert_async().await;

    Ok(())
}
