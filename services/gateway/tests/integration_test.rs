use reqwest::Client;

use crate::utils::{
    create_authenticated_user,
    testcontainers::{TestContainers, get_test_containers},
};

mod utils;

#[tokio::test]
async fn test_main() {
    let containers = get_test_containers().await;

    test_get_current_user_authenticated(&containers).await;
    test_get_current_user_unauthenticated(&containers).await;
}

async fn test_get_current_user_authenticated(containers: &TestContainers) {
    let authenticated_user = create_authenticated_user(&containers).await.unwrap();
    let uri = containers.gateway_uri().await;

    let resp = Client::new()
        .get(format!("{uri}/user/me"))
        .headers(authenticated_user.get_headers())
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(
        resp.status(),
        200,
        "test_get_current_user_authenticated failed: expected 201, got {}",
        resp.status()
    );
}

async fn test_get_current_user_unauthenticated(containers: &TestContainers) {
    let uri = containers.gateway_uri().await;

    let resp = Client::new()
        .get(format!("{uri}/user/me"))
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(
        resp.status(),
        401,
        "test_get_current_user_unauthenticated failed: expected 401, got {}",
        resp.status()
    );
}
