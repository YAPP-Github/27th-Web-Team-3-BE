use crate::domain::og::dto::OgMetadataResponse;
use crate::utils::error::AppError;
use scraper::{Html, Selector};
use tracing::{info, warn};

const REQUEST_TIMEOUT_SECS: u64 = 5;
const USER_AGENT: &str = "Mozilla/5.0 (compatible; MoalogBot/1.0)";

pub struct OgService;

impl OgService {
    /// 외부 URL에서 Open Graph 메타데이터를 파싱하여 반환합니다.
    /// 파싱 실패 시에도 에러가 아닌 URL만 포함한 fallback 응답을 반환합니다.
    pub async fn fetch_metadata(url: &str) -> Result<OgMetadataResponse, AppError> {
        if !is_valid_url(url) {
            return Err(AppError::BadRequest(
                "유효하지 않은 URL 형식입니다.".to_string(),
            ));
        }

        info!(url = %url, "OG 메타데이터 파싱 시작");

        let html = match fetch_html(url).await {
            Ok(html) => html,
            Err(e) => {
                warn!(url = %url, error = %e, "외부 URL 요청 실패, fallback 응답 반환");
                return Ok(fallback_response(url));
            }
        };

        let metadata = parse_og_tags(&html);

        Ok(OgMetadataResponse {
            url: url.to_string(),
            title: metadata.title,
            description: metadata.description,
            image: metadata.image,
        })
    }
}

/// URL 형식이 유효한지 검증합니다.
fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

/// 외부 URL에서 HTML을 가져옵니다.
async fn fetch_html(url: &str) -> Result<String, AppError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .map_err(|e| AppError::InternalError(format!("HTTP 클라이언트 생성 실패: {}", e)))?;

    let response = client
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await
        .map_err(|e| AppError::InternalError(format!("외부 URL 요청 실패: {}", e)))?;

    let text = response
        .text()
        .await
        .map_err(|e| AppError::InternalError(format!("응답 본문 읽기 실패: {}", e)))?;

    Ok(text)
}

struct OgTags {
    title: Option<String>,
    description: Option<String>,
    image: Option<String>,
}

/// HTML에서 Open Graph 태그를 파싱합니다.
fn parse_og_tags(html: &str) -> OgTags {
    let document = Html::parse_document(html);

    let title = extract_og_content(&document, "og:title");
    let description = extract_og_content(&document, "og:description");
    let image = extract_og_content(&document, "og:image");

    OgTags {
        title,
        description,
        image,
    }
}

/// 특정 OG 속성의 content 값을 추출합니다.
fn extract_og_content(document: &Html, property: &str) -> Option<String> {
    let selector = Selector::parse(&format!("meta[property=\"{}\"]", property)).ok()?;

    document
        .select(&selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string())
}

/// 파싱 실패 시 URL만 포함한 fallback 응답을 반환합니다.
fn fallback_response(url: &str) -> OgMetadataResponse {
    OgMetadataResponse {
        url: url.to_string(),
        title: None,
        description: None,
        image: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_true_for_valid_http_url() {
        // Arrange
        let url = "http://example.com";

        // Act
        let result = is_valid_url(url);

        // Assert
        assert!(result);
    }

    #[test]
    fn should_return_true_for_valid_https_url() {
        // Arrange
        let url = "https://example.com";

        // Act
        let result = is_valid_url(url);

        // Assert
        assert!(result);
    }

    #[test]
    fn should_return_false_for_invalid_url() {
        // Arrange
        let url = "not-a-url";

        // Act
        let result = is_valid_url(url);

        // Assert
        assert!(!result);
    }

    #[test]
    fn should_return_false_for_ftp_url() {
        // Arrange
        let url = "ftp://example.com";

        // Act
        let result = is_valid_url(url);

        // Assert
        assert!(!result);
    }

    #[test]
    fn should_parse_og_tags_from_html() {
        // Arrange
        let html = r#"
            <html>
            <head>
                <meta property="og:title" content="Test Title" />
                <meta property="og:description" content="Test Description" />
                <meta property="og:image" content="https://example.com/image.png" />
            </head>
            <body></body>
            </html>
        "#;

        // Act
        let result = parse_og_tags(html);

        // Assert
        assert_eq!(result.title, Some("Test Title".to_string()));
        assert_eq!(result.description, Some("Test Description".to_string()));
        assert_eq!(
            result.image,
            Some("https://example.com/image.png".to_string())
        );
    }

    #[test]
    fn should_return_none_for_missing_og_tags() {
        // Arrange
        let html = r#"
            <html>
            <head><title>No OG Tags</title></head>
            <body></body>
            </html>
        "#;

        // Act
        let result = parse_og_tags(html);

        // Assert
        assert!(result.title.is_none());
        assert!(result.description.is_none());
        assert!(result.image.is_none());
    }

    #[test]
    fn should_parse_partial_og_tags() {
        // Arrange
        let html = r#"
            <html>
            <head>
                <meta property="og:title" content="Only Title" />
            </head>
            <body></body>
            </html>
        "#;

        // Act
        let result = parse_og_tags(html);

        // Assert
        assert_eq!(result.title, Some("Only Title".to_string()));
        assert!(result.description.is_none());
        assert!(result.image.is_none());
    }

    #[test]
    fn should_handle_empty_content_attribute() {
        // Arrange
        let html = r#"
            <html>
            <head>
                <meta property="og:title" content="" />
            </head>
            <body></body>
            </html>
        "#;

        // Act
        let result = parse_og_tags(html);

        // Assert
        assert_eq!(result.title, Some("".to_string()));
    }

    #[test]
    fn should_return_fallback_response_with_url_only() {
        // Arrange
        let url = "https://example.com";

        // Act
        let result = fallback_response(url);

        // Assert
        assert_eq!(result.url, "https://example.com");
        assert!(result.title.is_none());
        assert!(result.description.is_none());
        assert!(result.image.is_none());
    }

    #[tokio::test]
    async fn should_return_error_for_invalid_url_format() {
        // Arrange
        let url = "not-a-valid-url";

        // Act
        let result = OgService::fetch_metadata(url).await;

        // Assert
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.error_code(), "COMMON400");
    }
}
