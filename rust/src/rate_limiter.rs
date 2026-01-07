use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use crate::error::AppError;

/// 사용자별 요청 횟수를 추적하고 제한하는 RateLimiter
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// 사용자별 요청 기록 (user_id -> (count, window_start))
    requests: Arc<Mutex<HashMap<String, (u32, SystemTime)>>>,
    /// 시간 윈도우 내 최대 요청 수
    max_requests: u32,
    /// 시간 윈도우 (초)
    window_duration: Duration,
}

impl RateLimiter {
    /// 새로운 RateLimiter를 생성합니다.
    ///
    /// # Arguments
    /// * `max_requests` - 시간 윈도우 내 최대 요청 수
    /// * `window_seconds` - 시간 윈도우 (초 단위)
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_duration: Duration::from_secs(window_seconds),
        }
    }

    /// 사용자의 요청을 확인하고 허용 여부를 반환합니다.
    ///
    /// # Arguments
    /// * `user_id` - 요청한 사용자의 ID
    ///
    /// # Returns
    /// * `Ok(())` - 요청이 허용됨
    /// * `Err(AppError::RateLimitExceeded)` - 요청 한도 초과
    pub fn check_rate_limit(&self, user_id: &str) -> Result<(), AppError> {
        let mut requests = self.requests.lock().unwrap();
        let now = SystemTime::now();

        let entry = requests.entry(user_id.to_string()).or_insert((0, now));

        // 시간 윈도우가 지났는지 확인
        let elapsed = now.duration_since(entry.1).unwrap_or(Duration::from_secs(0));

        if elapsed >= self.window_duration {
            // 새로운 윈도우 시작
            *entry = (1, now);
            Ok(())
        } else {
            // 현재 윈도우 내에서 요청 수 증가
            if entry.0 >= self.max_requests {
                let remaining_time = self.window_duration.as_secs() - elapsed.as_secs();
                Err(AppError::RateLimitExceeded(
                    format!("요청 한도를 초과했습니다. {}초 후에 다시 시도해주세요.", remaining_time)
                ))
            } else {
                entry.0 += 1;
                Ok(())
            }
        }
    }

    /// 사용자의 남은 요청 횟수를 조회합니다.
    ///
    /// # Arguments
    /// * `user_id` - 조회할 사용자의 ID
    ///
    /// # Returns
    /// * 남은 요청 횟수
    pub fn get_remaining_requests(&self, user_id: &str) -> u32 {
        let mut requests = self.requests.lock().unwrap();
        let now = SystemTime::now();

        if let Some(entry) = requests.get_mut(user_id) {
            let elapsed = now.duration_since(entry.1).unwrap_or(Duration::from_secs(0));

            if elapsed >= self.window_duration {
                // 윈도우가 만료되었으므로 리셋
                *entry = (0, now);
                self.max_requests
            } else {
                self.max_requests.saturating_sub(entry.0)
            }
        } else {
            self.max_requests
        }
    }

    /// 사용자의 요청 기록을 초기화합니다.
    ///
    /// # Arguments
    /// * `user_id` - 초기화할 사용자의 ID
    pub fn reset_user(&self, user_id: &str) {
        let mut requests = self.requests.lock().unwrap();
        requests.remove(user_id);
    }

    /// 모든 요청 기록을 초기화합니다.
    pub fn reset_all(&self) {
        let mut requests = self.requests.lock().unwrap();
        requests.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_rate_limiter_allows_requests_within_limit() {
        // Given: 3개 요청/5초 제한의 RateLimiter
        let limiter = RateLimiter::new(3, 5);
        let user_id = "user123";

        // When: 제한 내에서 요청
        let result1 = limiter.check_rate_limit(user_id);
        let result2 = limiter.check_rate_limit(user_id);
        let result3 = limiter.check_rate_limit(user_id);

        // Then: 모두 허용되어야 함
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
    }

    #[test]
    fn test_rate_limiter_blocks_requests_exceeding_limit() {
        // Given: 2개 요청/5초 제한의 RateLimiter
        let limiter = RateLimiter::new(2, 5);
        let user_id = "user456";

        // When: 제한을 초과하여 요청
        limiter.check_rate_limit(user_id).unwrap();
        limiter.check_rate_limit(user_id).unwrap();
        let result = limiter.check_rate_limit(user_id);

        // Then: 세 번째 요청은 차단되어야 함
        assert!(result.is_err());
        if let Err(AppError::RateLimitExceeded(msg)) = result {
            assert!(msg.contains("요청 한도를 초과했습니다"));
        } else {
            panic!("Expected RateLimitExceeded error");
        }
    }

    #[test]
    fn test_rate_limiter_resets_after_window() {
        // Given: 2개 요청/1초 제한의 RateLimiter
        let limiter = RateLimiter::new(2, 1);
        let user_id = "user789";

        // When: 제한까지 요청 후 윈도우가 지난 후 다시 요청
        limiter.check_rate_limit(user_id).unwrap();
        limiter.check_rate_limit(user_id).unwrap();

        // 1초 대기
        thread::sleep(Duration::from_secs(2));

        let result = limiter.check_rate_limit(user_id);

        // Then: 윈도우가 리셋되어 다시 허용되어야 함
        assert!(result.is_ok());
    }

    #[test]
    fn test_rate_limiter_tracks_different_users_separately() {
        // Given: 2개 요청/5초 제한의 RateLimiter
        let limiter = RateLimiter::new(2, 5);

        // When: 서로 다른 사용자가 요청
        limiter.check_rate_limit("user_a").unwrap();
        limiter.check_rate_limit("user_a").unwrap();

        let result_a = limiter.check_rate_limit("user_a");
        let result_b = limiter.check_rate_limit("user_b");

        // Then: user_a는 차단, user_b는 허용
        assert!(result_a.is_err());
        assert!(result_b.is_ok());
    }

    #[test]
    fn test_get_remaining_requests() {
        // Given: 5개 요청/10초 제한의 RateLimiter
        let limiter = RateLimiter::new(5, 10);
        let user_id = "user_remaining";

        // When: 2번 요청 후 남은 횟수 확인
        limiter.check_rate_limit(user_id).unwrap();
        limiter.check_rate_limit(user_id).unwrap();
        let remaining = limiter.get_remaining_requests(user_id);

        // Then: 3개가 남아있어야 함
        assert_eq!(remaining, 3);
    }

    #[test]
    fn test_reset_user() {
        // Given: 2개 요청/5초 제한의 RateLimiter
        let limiter = RateLimiter::new(2, 5);
        let user_id = "user_reset";

        // When: 제한까지 요청 후 리셋
        limiter.check_rate_limit(user_id).unwrap();
        limiter.check_rate_limit(user_id).unwrap();
        limiter.reset_user(user_id);
        let result = limiter.check_rate_limit(user_id);

        // Then: 리셋 후 다시 허용되어야 함
        assert!(result.is_ok());
    }

    #[test]
    fn test_reset_all() {
        // Given: 1개 요청/5초 제한의 RateLimiter
        let limiter = RateLimiter::new(1, 5);

        // When: 여러 사용자가 요청 후 전체 리셋
        limiter.check_rate_limit("user1").unwrap();
        limiter.check_rate_limit("user2").unwrap();
        limiter.reset_all();

        let result1 = limiter.check_rate_limit("user1");
        let result2 = limiter.check_rate_limit("user2");

        // Then: 모든 사용자가 다시 요청 가능해야 함
        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }
}

