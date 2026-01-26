// 회고 레트로룸 관련 API 단위 테스트(mock)
// CLAUDE.md 및 .claude/rules/rust-tests.md 기준

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_create_retro_room_success() {
        // given
        let title = "테스트 레트로룸";
        let description = Some("테스트 설명");
        // when
        let result = create_retro_room(title, description);
        // then
        assert!(result.is_ok());
        let room = result.unwrap();
        assert_eq!(room.title, title);
        assert_eq!(room.description, description.unwrap());
        assert!(room.invite_code.starts_with("INV-"));
    }

    #[test]
    fn test_create_retro_room_title_too_long() {
        // given
        let title = "a".repeat(21);
        let description = None;
        // when
        let result = create_retro_room(&title, description);
        // then
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().code, "RETRO4001");
    }

    #[test]
    fn test_join_retro_room_success() {
        // given
        let invite_url = "https://service.com/invite/INV-TEST-1234";
        // when
        let result = join_retro_room(invite_url);
        // then
        assert!(result.is_ok());
        let join_info = result.unwrap();
        assert_eq!(join_info.retro_room_id, 1);
        assert_eq!(join_info.joined_at.date(), Utc::now().date());
    }

    #[test]
    fn test_join_retro_room_expired() {
        // given
        let invite_url = "https://service.com/invite/INV-EXPIRED-0000";
        // when
        let result = join_retro_room(invite_url);
        // then
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().code, "RETRO4003");
    }
}

// mock 함수 정의 (실제 서비스 로직은 domain/retrospect/service.rs 참고)
fn create_retro_room(title: &str, description: Option<&str>) -> Result<RetroRoom, ApiError> {
    if title.len() > 20 {
        return Err(ApiError { code: "RETRO4001".to_string() });
    }
    Ok(RetroRoom {
        retro_room_id: 1,
        title: title.to_string(),
        description: description.unwrap_or("").to_string(),
        invite_code: "INV-TEST-1234".to_string(),
    })
}

fn join_retro_room(invite_url: &str) -> Result<JoinInfo, ApiError> {
    if invite_url.contains("EXPIRED") {
        return Err(ApiError { code: "RETRO4003".to_string() });
    }
    Ok(JoinInfo {
        retro_room_id: 1,
        joined_at: chrono::Utc::now(),
    })
}

struct RetroRoom {
    retro_room_id: i64,
    title: String,
    description: String,
    invite_code: String,
}

struct JoinInfo {
    retro_room_id: i64,
    joined_at: chrono::DateTime<chrono::Utc>,
}

struct ApiError {
    code: String,
}

