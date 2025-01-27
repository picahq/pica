use crate::context::TestServer;
use api::logic::{common_enum::CreateRequest, ReadResponse};
use entities::common_model::CommonEnum;
use fake::{Fake, Faker};
use http::{Method, StatusCode};
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_pagination() {
    let server = TestServer::new(None).await;

    let mut enums = vec![];
    for _ in 0..10 {
        let req: CreateRequest = Faker.fake();
        let res = server
            .send_request::<Value, Value>(
                "v1/common-enums",
                Method::POST,
                Some(&server.live_key),
                Some(&serde_json::to_value(&req).unwrap()),
            )
            .await
            .unwrap();
        assert_eq!(res.code, StatusCode::OK);

        let r#enum: CommonEnum = serde_json::from_value(res.data).unwrap();
        let CreateRequest { name, options, .. } = req;

        assert_eq!(name, r#enum.name);
        assert_eq!(options, r#enum.options);

        enums.push(r#enum);
        sleep(Duration::from_millis(100)).await;
    }

    let pipelines: Vec<CommonEnum> = enums.into_iter().rev().collect();

    check_response(&server, 1, 0, &pipelines[..1]).await;
    check_response(&server, 10, 0, &pipelines).await;
    check_response(&server, 0, 10, &pipelines[10..]).await;
    check_response(&server, 5, 0, &pipelines[..5]).await;
    check_response(&server, 5, 5, &pipelines[5..]).await;
    check_response(&server, 5, 10, &pipelines[10..]).await;
}

async fn check_response(server: &TestServer, limit: u64, skip: u64, enums: &[CommonEnum]) {
    let res = server
        .send_request::<Value, Value>(
            &format!("v1/common-enums?limit={limit}&skip={skip}"),
            Method::GET,
            Some(&server.live_key),
            None,
        )
        .await
        .unwrap();

    assert_eq!(res.code, StatusCode::OK);

    let res: ReadResponse<CommonEnum> = serde_json::from_value(res.data).unwrap();
    assert_eq!(&res.rows, enums);
    assert_eq!(res.limit, limit);
    assert_eq!(res.skip, skip);
    assert_eq!(res.total, 10);
}
