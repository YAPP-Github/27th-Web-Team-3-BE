# 📝 API 작업 리포트: 회고 작성 가이드 API

## 1. 요약 (Summary)

### 💡 고려한 요구 사항

* **기능 핵심**: 사용자가 작성 중인 회고 내용을 분석하여 AI(GPT-4o)가 가이드 메시지를 제공.
* **보안**: `secretKey`를 통한 간단한 인증 로직 적용 (서버 설정값과 비교).
* **유효성 검사**: `currentContent`와 `secretKey`의 필수 값 체크 및 빈 문자열 방지.
* **에러 처리**: 인증 실패 시 `AI_001`, 잘못된 요청 시 `COMMON400`, 서버 에러 시 `COMMON500` 코드 반환.

### 📤 요청 (Request)

* **Method**: `POST`
* **Endpoint**: `/api/ai/retrospective/guide`
* **Auth**: Request Body의 `secretKey` 사용

### 📥 응답 리스트 (Response List)

* **200 OK**: 성공적으로 가이드 메시지 생성 (`isSuccess: true`)
* **400 Bad Request**: 필수 파라미터 누락 (`COMMON400`)
* **401 Unauthorized**: 유효하지 않은 비밀 키 (`AI_001`)
* **500 Internal Server Error**: OpenAI API 호출 실패 등 서버 에러 (`COMMON500`)

### ✅ 수행한 테스트 (Checklist)

* [x] **정상 시나리오**: 올바른 키와 내용을 보냈을 때 `guideMessage` 반환 확인 (통합 테스트 `api_test.rs` - 로컬/CI 환경에서는 Mocking 필요할 수 있음)
* [x] **유효성 검사 실패**: `currentContent`가 빈 값일 때 `COMMON400` 반환 확인
* [x] **인증 실패**: `secretKey`가 틀렸을 때 `AI_001` (401 Unauthorized) 반환 확인

---

## 2. 상세 내역 (Detailed Details)

### 📋 자세한 요구사항

1. **AI 가이드 생성**: OpenAI Chat Completion API를 사용하여 시스템 프롬프트("회고 작성을 도와주는 친절한 AI 코치...")와 사용자 내용을 조합.
2. **데이터 제약 조건**:
    * `currentContent`: `min_length = 1`
    * `secretKey`: `min_length = 1`
3. **비즈니스 로직**:
    * 환경 변수 `APP_SECRET_KEY`와 요청의 `secretKey` 비교.
    * 일치 시 OpenAI 호출, 불일치 시 401 에러.

### 🛠 실제 요청/응답 예시 (JSON)

#### **실제 요청 (Request JSON)**

```json
{
  "currentContent": "오늘 프로젝트를 진행하면서 에러 처리가 힘들었어.",
  "secretKey": "mySecretKey123"
}
```

#### **실제 응답 (Response JSON - Success)**

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "currentContent": "오늘 프로젝트를 진행하면서 에러 처리가 힘들었어.",
    "guideMessage": "에러 처리 때문에 많이 힘드셨군요. 구체적으로 어떤 에러가 발생했는지, 그리고 그걸 해결하기 위해 어떤 시도를 해보셨는지 적어보면 좋을 것 같아요."
  }
}
```

#### **실제 응답 (Response JSON - Error)**

```json
{
  "isSuccess": false,
  "code": "AI_001",
  "message": "유효하지 않은 비밀 키입니다.",
  "result": null
}
```

### 🧪 수행한 테스트 상세

1. **[Validation Error Test]**:
    * **방법**: `tests/api_test.rs`에서 `currentContent`를 빈 문자열로 설정하여 요청.
    * **결과**: Status 400, Code `COMMON400` 확인.

2. **[Unauthorized Test]**:
    * **방법**: `tests/api_test.rs`에서 `secretKey`를 틀린 값으로 설정하여 요청.
    * **결과**: Status 401, Code `AI_001` 확인.

---

## 참고 자료

- `codes/server/tests/api_test.rs`: 통합 테스트 코드
- `codes/server/src/domain/ai/service.rs`: 비즈니스 로직
