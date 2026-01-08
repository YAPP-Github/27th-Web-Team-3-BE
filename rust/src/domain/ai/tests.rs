#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use crate::config::AppConfig;
    use crate::domain::ai::controller;
    use crate::models::request::RetrospectiveGuideRequest;
    use crate::rate_limiter::RateLimiter;

    fn get_test_config() -> AppConfig {
        AppConfig {
            openai_api_key: "test_openai_key".to_string(),
            ai_secret_key: "test_secret_key".to_string(),
        }
    }

    #[actix_web::test]
    async fn test_provide_retrospective_guide_success() {
        // Arrange
        let config = web::Data::new(get_test_config());
        let rate_limiter = web::Data::new(RateLimiter::new(10, 60));

        let app = test::init_service(
            App::new()
                .app_data(config.clone())
                .app_data(rate_limiter.clone())
                .service(
                    web::scope("/api/ai")
                        .configure(controller::configure)
                )
        ).await;

        let req_body = RetrospectiveGuideRequest {
            content: "오늘 프로젝트를 진행하면서 많은 것을 배웠다.".to_string(),
            secret_key: "test_secret_key".to_string(),
        };

        // Act
        let req = test::TestRequest::post()
            .uri("/api/ai/retrospective/guide")
            .set_json(&req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["isSuccess"], true);
        assert_eq!(body["code"], "COMMON200");
        assert!(body["result"]["guideMessage"].is_string());
    }

    #[actix_web::test]
    async fn test_provide_retrospective_guide_invalid_secret_key() {
        // Arrange
        let config = web::Data::new(get_test_config());
        let rate_limiter = web::Data::new(RateLimiter::new(10, 60));

        let app = test::init_service(
            App::new()
                .app_data(config.clone())
                .app_data(rate_limiter.clone())
                .service(
                    web::scope("/api/ai")
                        .configure(controller::configure)
                )
        ).await;

        let req_body = RetrospectiveGuideRequest {
            content: "오늘 프로젝트를 진행하면서 많은 것을 배웠다.".to_string(),
            secret_key: "wrong_secret_key".to_string(),
        };

        // Act
        let req = test::TestRequest::post()
            .uri("/api/ai/retrospective/guide")
            .set_json(&req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), 401);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["isSuccess"], false);
        assert_eq!(body["code"], "AI_001");
    }

    #[actix_web::test]
    async fn test_provide_retrospective_guide_missing_content() {
        // Arrange
        let config = web::Data::new(get_test_config());
        let rate_limiter = web::Data::new(RateLimiter::new(10, 60));

        let app = test::init_service(
            App::new()
                .app_data(config.clone())
                .app_data(rate_limiter.clone())
                .service(
                    web::scope("/api/ai")
                        .configure(controller::configure)
                )
        ).await;

        let req_body = serde_json::json!({
            "content": "",
            "secretKey": "test_secret_key"
        });

        // Act
        let req = test::TestRequest::post()
            .uri("/api/ai/retrospective/guide")
            .set_json(&req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), 400);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["isSuccess"], false);
        assert_eq!(body["code"], "COMMON400");
    }
}

