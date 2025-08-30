mod utils;

use crate::utils::{create_authenticated_user, testcontainers::get_test_containers};
use reqwest::Client;

#[tokio::test]
async fn test_get_current_user_authenticated() {
    let containers = get_test_containers().await;
    let authenticated_user = create_authenticated_user(&containers).await.unwrap();
    let uri = containers.gateway_uri().await;

    let resp = Client::new()
        .get(format!("{uri}/user/me"))
        .headers(authenticated_user.get_headers())
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_get_current_user_unauthenticated() {
    let containers = get_test_containers().await;
    let uri = containers.gateway_uri().await;

    let resp = Client::new()
        .get(format!("{uri}/user/me"))
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_logout_user() {
    let containers = get_test_containers().await;
    let authenticated_user = create_authenticated_user(&containers).await.unwrap();
    let uri = containers.gateway_uri().await;

    let resp = Client::new()
        .post(format!("{uri}/logout"))
        .headers(authenticated_user.get_headers())
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(resp.status(), 200);

    let resp = Client::new()
        .get(format!("{uri}/user/me"))
        .headers(authenticated_user.get_headers())
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(resp.status(), 401);
}
