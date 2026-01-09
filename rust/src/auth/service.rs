use bcrypt::{hash, verify, DEFAULT_COST};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::common::error::AppError;
use super::models::{User, RegisterRequest, LoginRequest};

/// 사용자 저장소 (실제로는 DB를 사용해야 하지만 예제용으로 메모리 사용)
#[derive(Clone)]
pub struct UserRepository {
    users: Arc<Mutex<HashMap<String, User>>>,
}

impl UserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 이메일로 사용자 찾기
    pub fn find_by_email(&self, email: &str) -> Option<User> {
        let users = self.users.lock().unwrap();
        users.values().find(|u| u.email == email).cloned()
    }

    /// 사용자 저장
    pub fn save(&self, user: User) -> Result<User, AppError> {
        let mut users = self.users.lock().unwrap();
        let id = user.id.clone();
        users.insert(id, user.clone());
        Ok(user)
    }

    /// ID로 사용자 찾기
    pub fn find_by_id(&self, id: &str) -> Option<User> {
        let users = self.users.lock().unwrap();
        users.get(id).cloned()
    }
}

impl Default for UserRepository {
    fn default() -> Self {
        Self::new()
    }
}

/// 인증 서비스
#[derive(Clone)]
pub struct AuthService {
    repository: UserRepository,
}

impl AuthService {
    pub fn new(repository: UserRepository) -> Self {
        Self { repository }
    }

    /// 회원가입
    pub fn register(&self, request: RegisterRequest) -> Result<User, AppError> {
        // 이메일 중복 체크
        if self.repository.find_by_email(&request.email).is_some() {
            return Err(AppError::Conflict(format!(
                "이미 등록된 이메일입니다: {}",
                request.email
            )));
        }

        // 비밀번호 해싱
        let password_hash = hash(&request.password, DEFAULT_COST)
            .map_err(|e| AppError::InternalError(format!("비밀번호 해싱 실패: {}", e)))?;

        // 사용자 생성
        let user = User::new(request.email, password_hash, request.name);

        // 저장
        self.repository.save(user)
    }

    /// 로그인
    pub fn login(&self, request: LoginRequest) -> Result<User, AppError> {
        // 사용자 찾기
        let user = self
            .repository
            .find_by_email(&request.email)
            .ok_or_else(|| AppError::AuthError("이메일 또는 비밀번호가 일치하지 않습니다".to_string()))?;

        // 비밀번호 검증
        let valid = verify(&request.password, &user.password_hash)
            .map_err(|e| AppError::InternalError(format!("비밀번호 검증 실패: {}", e)))?;

        if !valid {
            return Err(AppError::AuthError(
                "이메일 또는 비밀번호가 일치하지 않습니다".to_string(),
            ));
        }

        Ok(user)
    }

    /// ID로 사용자 찾기
    pub fn find_user_by_id(&self, id: &str) -> Result<User, AppError> {
        self.repository
            .find_by_id(id)
            .ok_or_else(|| AppError::NotFound(format!("사용자를 찾을 수 없습니다: {}", id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_repository_save_and_find() {
        let repo = UserRepository::new();
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "Test User".to_string(),
        );

        let id = user.id.clone();
        repo.save(user.clone()).unwrap();

        let found = repo.find_by_id(&id).unwrap();
        assert_eq!(found.email, "test@example.com");
    }

    #[test]
    fn test_find_by_email() {
        let repo = UserRepository::new();
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "Test User".to_string(),
        );

        repo.save(user.clone()).unwrap();

        let found = repo.find_by_email("test@example.com").unwrap();
        assert_eq!(found.name, "Test User");
    }

    #[test]
    fn test_register_success() {
        let repo = UserRepository::new();
        let service = AuthService::new(repo);

        let request = RegisterRequest {
            email: "new@example.com".to_string(),
            password: "password123".to_string(),
            name: "New User".to_string(),
        };

        let result = service.register(request);
        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.email, "new@example.com");
        assert_ne!(user.password_hash, "password123"); // 해싱되어야 함
    }

    #[test]
    fn test_register_duplicate_email() {
        let repo = UserRepository::new();
        let service = AuthService::new(repo);

        let request = RegisterRequest {
            email: "duplicate@example.com".to_string(),
            password: "password123".to_string(),
            name: "User".to_string(),
        };

        service.register(request.clone()).unwrap();
        let result = service.register(request);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Conflict(_)));
    }

    #[test]
    fn test_login_success() {
        let repo = UserRepository::new();
        let service = AuthService::new(repo);

        // 먼저 회원가입
        let register_req = RegisterRequest {
            email: "login@example.com".to_string(),
            password: "password123".to_string(),
            name: "Login User".to_string(),
        };
        service.register(register_req).unwrap();

        // 로그인 시도
        let login_req = LoginRequest {
            email: "login@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = service.login(login_req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_login_wrong_password() {
        let repo = UserRepository::new();
        let service = AuthService::new(repo);

        // 먼저 회원가입
        let register_req = RegisterRequest {
            email: "user@example.com".to_string(),
            password: "correct_password".to_string(),
            name: "User".to_string(),
        };
        service.register(register_req).unwrap();

        // 잘못된 비밀번호로 로그인 시도
        let login_req = LoginRequest {
            email: "user@example.com".to_string(),
            password: "wrong_password".to_string(),
        };

        let result = service.login(login_req);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::AuthError(_)));
    }
}

