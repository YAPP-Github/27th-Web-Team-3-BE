use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt; // for collect
use serde_json::{json, Value};
use tower::ServiceExt; // for oneshot
use web3_server::app;

#[tokio::test]
async fn test_health_check() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"OK");
}

#[tokio::test]
async fn test_guide_api_validation_error() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ai/retrospective/guide")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "currentContent": "", // Empty content -> should fail
                        "secretKey": "mySecretKey123"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();

        let body_json: Value = serde_json::from_slice(&body).unwrap();

        

        assert_eq!(body_json["isSuccess"], false);

        assert_eq!(body_json["code"], "COMMON400");

    }

    

    #[tokio::test]

    async fn test_guide_api_unauthorized() {

        let app = app();

    

        let response = app

            .oneshot(

                Request::builder()

                    .method("POST")

                    .uri("/api/ai/retrospective/guide")

                    .header("Content-Type", "application/json")

                    .body(Body::from(

                        json!({

                            "currentContent": "Some content",

                            "secretKey": "WRONG_KEY"

                        })

                        .to_string(),

                    ))

                    .unwrap(),

            )

            .await

            .unwrap();

    

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    

        let body = response.into_body().collect().await.unwrap().to_bytes();

        let body_json: Value = serde_json::from_slice(&body).unwrap();

        

        assert_eq!(body_json["isSuccess"], false);

        assert_eq!(body_json["code"], "AI_001");

    }

    
