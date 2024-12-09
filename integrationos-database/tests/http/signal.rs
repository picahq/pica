use crate::context::TestServer;
use http::header::{ACCEPT, CONTENT_LENGTH, CONTENT_TYPE, HOST};
use integrationos_domain::{
    emitted_events::DatabaseConnectionLost, prefix::IdPrefix, Id, IntegrationOSError, Unit,
};
use mockito::Server as MockServer;
use std::collections::HashMap;

#[tokio::test]
async fn test_kill_signal() -> Result<Unit, IntegrationOSError> {
    let mut mock_server = MockServer::new_async().await;
    let mock_uri = mock_server.url();

    let connection_id = Id::now(IdPrefix::Connection);

    let path = "/v1/emit";
    let body = DatabaseConnectionLost {
        connection_id,
        reason: Some(
            "error returned from database: password authentication failed for user \"postgres\""
                .to_string(),
        ),
        schedule_on: None,
    }
    .as_event();

    let mock_server = mock_server
        .mock("POST", path)
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
        ("EMIT_URL".to_string(), mock_uri),
    ]))
    .await;

    mock_server.expect(1).assert_async().await;

    Ok(())
}