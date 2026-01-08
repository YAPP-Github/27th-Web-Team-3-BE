# 📋 [설계] AI 회고 작성 가이드 제공:

### 1. 기능 개요
* 사용자가 작성 중인 내용을 분석하여 적절한 코칭 메시지를 제공해야 함.\
* 인증을 위해 secretKey가 반드시 유효해야 함.

### 2. 인터페이스 약속 (Interface)
* **호출 URL:** POST /api/ai/retrospective/guide
* **입력값 (Request):**
    - `secretKey`: string (필수)
    - `content`: string (필수) - 사용자가 작성한 회고 내용
* **출력값 (Response):**
    - guideMessage: AI가 생성한 조언 메시지

### 3. 요구사항 및 예외 처리 (Must-Have & Exception)
* COMMON400: currentContent 또는 secretKey 누락 시
* AI_001: 잘못된 secretKey 입력 시
* COMMON500: AI 서버 통신 장애 등 내부 에러 발생 시

---

# 📊 [리포트] 구현 결과 확인

### 1. 구현 요약
* **상태:** ✅ 개발 완료
* **설계 준수:** 설계서의 모든 예외 처리 및 규약 반영 완료.
* **구현 위치:**
    - Controller: `src/domain/ai/controller.rs`
    - Service: `src/domain/ai/service.rs`
    - Models: `src/models/request.rs`, `src/models/response.rs`
    - Main: `src/main.rs`

### 2. 정상 작동 증빙 (Success Case)
* **상황:** 유효한 secretKey와 회고 내용을 전송했을 때
* **입력 데이터:**
```json
{
  "secretKey": "valid_secret_key_123",
  "content": "오늘 프로젝트를 진행하면서 어려움이 있었습니다."
}
```
* **실제 출력값:**
```json
{
  "guideMessage": "좋은 시작이에요! 구체적으로 어떤 점이 어려웠는지 작성해보면 어떨까요?"
}
```

### 3. 에러 대응 증빙 (Error Case)

#### 시나리오 1: content 누락
* **입력:**
```json
{
  "secretKey": "valid_secret_key_123",
  "content": ""
}
```
* **결과:**
```json
{
  "code": "COMMON400",
  "message": "content는 필수 입력값입니다."
}
```

#### 시나리오 2: secretKey 누락
* **입력:**
```json
{
  "secretKey": "",
  "content": "테스트 내용"
}
```
* **결과:**
```json
{
  "code": "COMMON400",
  "message": "secretKey는 필수 입력값입니다."
}
```

#### 시나리오 3: 잘못된 secretKey
* **입력:**
```json
{
  "secretKey": "invalid_key",
  "content": "테스트 내용"
}
```
* **결과:**
```json
{
  "code": "AI_001",
  "message": "잘못된 secretKey입니다."
}
```

#### 시나리오 4: AI 서버 통신 오류
* **상황:** AI 서버가 응답하지 않거나 에러를 반환할 때
* **결과:**
```json
{
  "code": "COMMON500",
  "message": "AI 서버 통신 중 오류가 발생했습니다."
}
```

### 4. 테스트 코드
테스트 파일 위치: 
- `src/domain/ai/controller.rs` (tests 모듈)
- `src/domain/ai/service.rs` (tests 모듈)

**구현된 테스트 케이스:**

**Controller 테스트:**
1. `test_provide_guide_success` - 정상 요청 테스트
2. `test_provide_guide_missing_content` - content 누락 테스트
3. `test_provide_guide_missing_secret_key` - secretKey 누락 테스트
4. `test_refine_retrospective_success` - 다듬기 정상 요청 테스트
5. `test_refine_retrospective_missing_content` - 다듬기 content 누락 테스트

**Service 테스트:**
1. `test_validate_secret_key_success` - secretKey 검증 성공
2. `test_validate_secret_key_failure` - secretKey 검증 실패
3. `test_generate_retrospective_guide` - 가이드 생성 테스트
4. `test_refine_retrospective_polite` - 정중체 다듬기 테스트
5. `test_refine_retrospective_kind` - 상냥체 다듬기 테스트

**테스트 실행 방법:**
```bash
# 전체 테스트
cargo test

# AI 도메인 테스트만 실행
cargo test --test domain::ai

# 특정 테스트만 실행
cargo test test_provide_guide
```

### 5. 기타 특이사항
* AI 서비스는 환경 변수 `AI_API_URL`과 `AI_API_KEY`를 통해 구성됩니다.
* secretKey 검증은 환경 변수 `SECRET_KEY`와 비교하여 수행됩니다.
* 모든 에러는 설계서에 명시된 에러 코드와 메시지를 정확히 따릅니다.
* AI 서버 응답 시간은 외부 API 상태에 따라 달라질 수 있습니다.

---

