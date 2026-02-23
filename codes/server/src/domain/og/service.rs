use crate::domain::og::dto::OgMetadataResponse;
use crate::utils::error::AppError;
use reqwest::redirect::Policy;
use reqwest::Client;
use scraper::{Html, Selector};
use std::net::Ipv6Addr;
use std::sync::LazyLock;
use tracing::{info, warn};
use url::{Host, Url};

const REQUEST_TIMEOUT_SECS: u64 = 5;
const USER_AGENT: &str = "Mozilla/5.0 (compatible; MoalogBot/1.0)";
/// 응답 본문 최대 크기 (1MB)
const MAX_BODY_BYTES: usize = 1_024 * 1_024;

/// 공유 HTTP 클라이언트 (커넥션 풀링 활용, 리다이렉트 비활성화)
static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .redirect(Policy::none())
        .build()
        .expect("HTTP 클라이언트 초기화 실패")
});

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
/// SSRF 방지를 위해 내부 네트워크 주소를 차단합니다.
/// NOTE: DNS rebinding 공격에 대한 완전한 방어를 위해서는 DNS 해석 후 IP 재검증이 필요합니다.
fn is_valid_url(url: &str) -> bool {
    let parsed = match Url::parse(url) {
        Ok(u) => u,
        Err(_) => return false,
    };

    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return false;
    }

    match parsed.host() {
        Some(Host::Domain(domain)) => {
            let domain = domain.to_ascii_lowercase();
            domain != "localhost"
        }
        Some(Host::Ipv4(ipv4)) => {
            !(ipv4.is_loopback()          // 127.0.0.0/8
                || ipv4.is_private()      // 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
                || ipv4.is_link_local()   // 169.254.0.0/16
                || ipv4.is_unspecified()) // 0.0.0.0
        }
        Some(Host::Ipv6(ipv6)) => !is_blocked_ipv6(&ipv6),
        None => false,
    }
}

/// IPv6 주소가 차단 대상인지 확인합니다.
/// IPv4-mapped IPv6 주소(::ffff:x.x.x.x), 유니크 로컬(fc00::/7), 링크로컬(fe80::/10)도 차단합니다.
fn is_blocked_ipv6(ipv6: &Ipv6Addr) -> bool {
    if ipv6.is_loopback() || ipv6.is_unspecified() {
        return true;
    }

    // IPv4-mapped IPv6 (::ffff:x.x.x.x) → 내부 IPv4로 매핑된 주소 차단
    if let Some(ipv4) = ipv6.to_ipv4_mapped() {
        return ipv4.is_loopback()
            || ipv4.is_private()
            || ipv4.is_link_local()
            || ipv4.is_unspecified();
    }

    let segments = ipv6.segments();

    // 유니크 로컬 주소 fc00::/7 (첫 7비트가 1111110)
    if segments[0] & 0xfe00 == 0xfc00 {
        return true;
    }

    // 링크로컬 주소 fe80::/10 (첫 10비트가 1111111010)
    if segments[0] & 0xffc0 == 0xfe80 {
        return true;
    }

    false
}

/// 외부 URL에서 HTML을 가져옵니다.
/// 리다이렉트는 SSRF 방지를 위해 비활성화되어 있으며, 3xx 응답은 fallback 처리됩니다.
async fn fetch_html(url: &str) -> Result<String, AppError> {
    let response = HTTP_CLIENT
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await
        .map_err(|e| AppError::InternalError(format!("외부 URL 요청 실패: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::InternalError(format!("외부 URL 응답 오류: {}", e)))?;

    let content_length = response.content_length().unwrap_or(0) as usize;

    if content_length > MAX_BODY_BYTES {
        return Err(AppError::InternalError(
            "응답 본문이 너무 큽니다.".to_string(),
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| AppError::InternalError(format!("응답 본문 읽기 실패: {}", e)))?;

    if bytes.len() > MAX_BODY_BYTES {
        return Err(AppError::InternalError(
            "응답 본문이 너무 큽니다.".to_string(),
        ));
    }

    String::from_utf8(bytes.to_vec())
        .map_err(|e| AppError::InternalError(format!("응답 인코딩 오류: {}", e)))
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
    fn should_return_false_for_localhost() {
        // Arrange
        let url = "http://localhost:8080/admin";

        // Act
        let result = is_valid_url(url);

        // Assert
        assert!(!result);
    }

    #[test]
    fn should_return_false_for_loopback_ip() {
        // Arrange
        let url = "http://127.0.0.1/admin";

        // Act
        let result = is_valid_url(url);

        // Assert
        assert!(!result);
    }

    #[test]
    fn should_return_false_for_aws_metadata_endpoint() {
        // Arrange
        let url = "http://169.254.169.254/latest/meta-data/";

        // Act
        let result = is_valid_url(url);

        // Assert
        assert!(!result);
    }

    #[test]
    fn should_return_false_for_private_network_10() {
        // Arrange
        let url = "http://10.0.0.1/internal";

        // Act
        let result = is_valid_url(url);

        // Assert
        assert!(!result);
    }

    #[test]
    fn should_return_false_for_private_network_172() {
        // Arrange
        let url = "http://172.16.0.1/internal";

        // Act
        let result = is_valid_url(url);

        // Assert
        assert!(!result);
    }

    #[test]
    fn should_return_false_for_private_network_192() {
        // Arrange
        let url = "http://192.168.1.1/internal";

        // Act
        let result = is_valid_url(url);

        // Assert
        assert!(!result);
    }

    #[test]
    fn should_return_false_for_ipv6_loopback() {
        // Arrange
        let url = "http://[::1]/admin";

        // Act
        let result = is_valid_url(url);

        // Assert
        assert!(!result);
    }

    #[test]
    fn should_return_false_for_ipv4_mapped_ipv6_loopback() {
        // Arrange
        let url = "http://[::ffff:127.0.0.1]/admin";

        // Act
        let result = is_valid_url(url);

        // Assert
        assert!(!result);
    }

    #[test]
    fn should_return_false_for_ipv4_mapped_ipv6_private() {
        // Arrange
        let url = "http://[::ffff:10.0.0.1]/internal";

        // Act
        let result = is_valid_url(url);

        // Assert
        assert!(!result);
    }

    #[test]
    fn should_return_false_for_ipv6_unique_local() {
        // Arrange
        let url = "http://[fc00::1]/internal";

        // Act
        let result = is_valid_url(url);

        // Assert
        assert!(!result);
    }

    #[test]
    fn should_return_false_for_ipv6_link_local() {
        // Arrange
        let url = "http://[fe80::1]/internal";

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

    #[tokio::test]
    async fn should_return_error_for_localhost_ssrf() {
        // Arrange
        let url = "http://localhost:8080/internal";

        // Act
        let result = OgService::fetch_metadata(url).await;

        // Assert
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.error_code(), "COMMON400");
    }

    #[tokio::test]
    async fn should_return_error_for_private_ip_ssrf() {
        // Arrange
        let url = "http://169.254.169.254/latest/meta-data/";

        // Act
        let result = OgService::fetch_metadata(url).await;

        // Assert
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.error_code(), "COMMON400");
    }

    #[tokio::test]
    async fn should_return_error_for_ipv4_mapped_ipv6_ssrf() {
        // Arrange
        let url = "http://[::ffff:127.0.0.1]/admin";

        // Act
        let result = OgService::fetch_metadata(url).await;

        // Assert
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.error_code(), "COMMON400");
    }
}
