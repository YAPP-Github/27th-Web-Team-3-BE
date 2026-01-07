use crate::error::AppError;
use crate::models::request::SignUpRequest;
use std::collections::HashMap;
use std::sync::Mutex;
use chrono::Utc;

// 간단한 인메모리 저장소 (실제 프로덕션에서는 DB 사용)
lazy_static::lazy_static! {
    static ref USERS: Mutex<HashMap<String, User>> = Mutex::new(HashMap::new());
    static ref USER_ID_COUNTER: Mutex<u64> = Mutex::new(1);
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub created_at: String,
}

pub async fn create_user(req: &SignUpRequest) -> Result<User, AppError> {
    let mut users = USERS.lock().unwrap();

    // 이메일 중복 체크
    if users.contains_key(&req.email) {
        return Err(AppError::Conflict("이미 존재하는 이메일입니다.".to_string()));
    }

    // 비밀번호 해싱 (실제로는 bcrypt 등 사용)
    let password_hash = hash_password(&req.password);

    let mut counter = USER_ID_COUNTER.lock().unwrap();
    let user_id = *counter;
    *counter += 1;
    drop(counter);

    let user = User {
        id: user_id,
        email: req.email.clone(),
        username: req.username.clone(),
        password_hash,
        created_at: Utc::now().to_rfc3339(),
    };

    users.insert(req.email.clone(), user.clone());

    Ok(user)
}

fn hash_password(password: &str) -> String {
    // 실제로는 bcrypt, argon2 등 사용
    // 여기서는 간단히 구현
    format!("hashed_{}", password)
}

