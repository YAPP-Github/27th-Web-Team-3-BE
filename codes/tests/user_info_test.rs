use serde_json::json;

#[tokio::test]
async fn test_get_user_info_success() {
    // Given: 서버가 실행 중이라고 가정
    let client = reqwest::Client::new();

    // When: GET /api/user/1 요청
    let response = client
        .get("http://127.0.0.1:3000/api/user/1")
        .send()
        .await
        .expect("Failed to send request");

    // Then: 200 OK와 사용자 정보 응답
    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["id"], 1);
    assert_eq!(body["data"]["name"], "홍길동");
    assert_eq!(body["data"]["email"], "hong@example.com");
}

#[tokio::test]
async fn test_get_user_info_not_found() {
    // Given: 서버가 실행 중이라고 가정
    let client = reqwest::Client::new();

    // When: GET /api/user/999 요청 (존재하지 않는 사용자)
    let response = client
        .get("http://127.0.0.1:3000/api/user/999")
        .send()
        .await
        .expect("Failed to send request");

    // Then: 404 Not Found
    assert_eq!(response.status(), 404);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["success"], false);
    assert!(body["message"].as_str().unwrap().contains("User not found"));
}

