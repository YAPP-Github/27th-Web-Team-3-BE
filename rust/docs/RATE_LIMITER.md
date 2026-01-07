# RateLimiter Module

## 개요
사용자별 API 요청 횟수를 추적하고 제한하는 RateLimiter 모듈입니다. 
Kent Beck의 TDD 원칙에 따라 테스트 우선 개발(Test-First Development)로 구현되었습니다.

## 특징
- ✅ **사용자별 요청 추적**: 사용자 ID를 키로 하여 개별 요청 횟수 관리
- ✅ **시간 윈도우 기반**: 설정된 시간 윈도우 내에서 요청 제한
- ✅ **자동 리셋**: 시간 윈도우가 지나면 자동으로 카운터 초기화
- ✅ **스레드 안전**: Arc<Mutex>를 사용한 안전한 동시성 처리
- ✅ **상세한 에러 메시지**: 재시도 가능 시간을 포함한 친절한 에러 응답

## 설치 및 설정

### main.rs에 추가
```rust
use rate_limiter::RateLimiter;

// RateLimiter 초기화 (10 requests per 60 seconds)
let rate_limiter = web::Data::new(RateLimiter::new(10, 60));

HttpServer::new(move || {
    App::new()
        .app_data(rate_limiter.clone())
        // ... 기타 설정
})
```

## 사용법

### 컨트롤러에서 사용
```rust
use crate::rate_limiter::RateLimiter;

pub async fn sign_up(
    req: web::Json<SignUpRequest>,
    rate_limiter: web::Data<RateLimiter>,
) -> Result<HttpResponse, AppError> {
    // Rate limiting 체크
    rate_limiter.check_rate_limit(&req.email)?;
    
    // ... 비즈니스 로직
}
```

### 주요 메서드

#### `new(max_requests: u32, window_seconds: u64) -> Self`
새로운 RateLimiter 인스턴스를 생성합니다.

**파라미터:**
- `max_requests`: 시간 윈도우 내 최대 요청 수
- `window_seconds`: 시간 윈도우 (초 단위)

**예시:**
```rust
// 1분에 10번 요청 제한
let limiter = RateLimiter::new(10, 60);
```

#### `check_rate_limit(&self, user_id: &str) -> Result<(), AppError>`
사용자의 요청을 확인하고 허용 여부를 반환합니다.

**파라미터:**
- `user_id`: 요청한 사용자의 ID

**반환:**
- `Ok(())`: 요청 허용
- `Err(AppError::RateLimitExceeded)`: 요청 한도 초과

**예시:**
```rust
match limiter.check_rate_limit("user123") {
    Ok(_) => println!("요청 허용"),
    Err(e) => println!("요청 거부: {}", e),
}
```

#### `get_remaining_requests(&self, user_id: &str) -> u32`
사용자의 남은 요청 횟수를 조회합니다.

**예시:**
```rust
let remaining = limiter.get_remaining_requests("user123");
println!("남은 요청: {}", remaining);
```

#### `reset_user(&self, user_id: &str)`
특정 사용자의 요청 기록을 초기화합니다.

#### `reset_all(&self)`
모든 사용자의 요청 기록을 초기화합니다.

## TDD 개발 과정

이 모듈은 Kent Beck의 TDD 원칙을 따라 개발되었습니다:

### 1. Red (실패하는 테스트 작성)
```rust
#[test]
fn test_rate_limiter_blocks_requests_exceeding_limit() {
    let limiter = RateLimiter::new(2, 5);
    limiter.check_rate_limit("user").unwrap();
    limiter.check_rate_limit("user").unwrap();
    let result = limiter.check_rate_limit("user");
    assert!(result.is_err()); // 처음엔 실패
}
```

### 2. Green (테스트 통과시키기)
최소한의 코드로 테스트를 통과시킵니다.

### 3. Refactor (리팩토링)
중복 제거, 코드 개선, 문서화 작업을 진행합니다.

## 테스트

전체 7개의 단위 테스트가 포함되어 있습니다:

```bash
cargo test rate_limiter
```

### 테스트 케이스
1. ✅ 제한 내 요청 허용
2. ✅ 제한 초과 요청 차단
3. ✅ 윈도우 만료 후 자동 리셋
4. ✅ 사용자별 독립적 추적
5. ✅ 남은 요청 횟수 조회
6. ✅ 사용자별 리셋
7. ✅ 전체 리셋

## 에러 응답 예시

요청 한도 초과 시:
```json
{
  "isSuccess": false,
  "code": "COMMON429",
  "message": "요청 한도를 초과했습니다. 45초 후에 다시 시도해주세요.",
  "result": null
}
```

HTTP Status: `429 Too Many Requests`

## 성능 고려사항

- **메모리**: 각 사용자당 약 80 bytes (user_id + count + timestamp)
- **동시성**: Mutex를 사용하므로 높은 동시성 환경에서는 대안 고려 필요
- **확장성**: 현재는 인메모리 저장소 사용, 분산 환경에서는 Redis 등 고려

## 향후 개선 사항

- [ ] Redis 백엔드 지원
- [ ] 동적 제한 설정 (사용자 등급별 차등 적용)
- [ ] 요청 로그 기록
- [ ] 관리자 API (제한 조회/수정)
- [ ] 메트릭 수집 및 모니터링

## 라이센스
프로젝트 라이센스를 따릅니다.

