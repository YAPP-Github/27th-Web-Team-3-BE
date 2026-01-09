use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// 회원가입 요청 DTO
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct RegisterRequest {
    #[validate(email(message = "올바른 이메일 형식이 아닙니다"))]
    pub email: String,

    #[validate(length(min = 8, message = "비밀번호는 최소 8자 이상이어야 합니다"))]
    pub password: String,

    #[validate(length(min = 2, max = 50, message = "이름은 2-50자 사이여야 합니다"))]
    pub name: String,
}

/// 로그인 요청 DTO
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct LoginRequest {
    #[validate(email(message = "올바른 이메일 형식이 아닙니다"))]
    pub email: String,

    #[validate(length(min = 1, message = "비밀번호를 입력해주세요"))]
    pub password: String,
}

/// 인증 응답 DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserDto,
}

/// 사용자 DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDto {
    pub id: String,
    pub email: String,
    pub name: String,
}

/// 사용자 모델 (데이터베이스 모델 대신 간단한 메모리 저장용)
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub name: String,
}

impl User {
    pub fn new(email: String, password_hash: String, name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            email,
            password_hash,
            name,
        }
    }

    pub fn to_dto(&self) -> UserDto {
        UserDto {
            id: self.id.clone(),
            email: self.email.clone(),
            name: self.name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_valid_register_request() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_invalid_email() {
        let request = RegisterRequest {
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_short_password() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "short".to_string(),
            name: "Test User".to_string(),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "Test User".to_string(),
        );

        assert!(!user.id.is_empty());
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.name, "Test User");
    }

    #[test]
    fn test_user_to_dto() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "Test User".to_string(),
        );

        let dto = user.to_dto();
        assert_eq!(dto.id, user.id);
        assert_eq!(dto.email, user.email);
        assert_eq!(dto.name, user.name);
    }
}

