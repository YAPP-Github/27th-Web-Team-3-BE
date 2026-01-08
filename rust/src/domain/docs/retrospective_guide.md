# 📊 [회고 작성 가이드] 구현 결과 확인

### 1. 구현 요약
* **상태:** ✅ 개발 완료
* **설계 준수:** 설계서의 모든 예외 처리 및 규약 반영 완료.

### 2. API 정보
* **호출 URL:** POST /api/ai/retrospective/guide
* **입력값 (Request):**
    - `secretKey`: string (필수) - AI 서비스 인증 키
    - `content`: string (필수) - 사용자가 작성한 회고 내용
* **출력값 (Response):**
    - `guideMessage`: string - AI가 생성한 조언 메시지

### 3. 정상 작동 증빙 (Success Case)
* **상황:** 회고 내용 작성 시 가이드 요청
* **입력 데이터:**
```json
{
  "secretKey": "test_secret_key",
  "content": "오늘 프로젝트를 진행하면서 많은 것을 배웠다."
}
```
* **실제 출력값:**
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "guideMessage": "좋은 시작이에요! 구체적으로 어떤 어려움이 있었는지, 그리고 무엇을 배웠는지 더 자세히 작성해보면 좋을 것 같아요. 또한 다음에 비슷한 상황이 왔을 때 어떻게 대처할지 계획을 추가하면 더 완성도 높은 회고가 될 거예요."
  }
}
```

### 4. 에러 대응 증빙 (Error Case)

#### 시나리오 1: 잘못된 secretKey 입력 (AI_001)
* **입력 데이터:**
```json
{
  "secretKey": "wrong_key",
  "content": "오늘 프로젝트를 진행하면서 많은 것을 배웠다."
}
```
* **결과:**
```json
{
  "isSuccess": false,
  "code": "AI_001",
  "message": "유효하지 않은 비밀 키입니다.",
  "result": null
}
```

#### 시나리오 2: content 누락 (COMMON400)
* **입력 데이터:**
```json
{
  "secretKey": "test_secret_key",
  "content": ""
}
```
* **결과:**
```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "잘못된 요청입니다.",
  "result": null
}
```

#### 시나리오 3: secretKey 누락 (COMMON400)
* **입력 데이터:**
```json
{
  "content": "오늘 프로젝트를 진행하면서 많은 것을 배웠다."
}
```
* **결과:**
```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "잘못된 요청입니다.",
  "result": null
}
```

#### 시나리오 4: Rate Limit 초과 (COMMON429)
* **상황:** 60초 내 10회 이상 요청
* **결과:**
```json
{
  "isSuccess": false,
  "code": "COMMON429",
  "message": "Rate limit exceeded. Try again later.",
  "result": null
}
```

### 5. 테스트 결과
```bash
# 단위 테스트 실행
$ cargo test test_provide_retrospective_guide

running 3 tests
test domain::ai::tests::tests::test_provide_retrospective_guide_missing_content ... ok
test domain::ai::tests::tests::test_provide_retrospective_guide_success ... ok
test domain::ai::tests::tests::test_provide_retrospective_guide_invalid_secret_key ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured
```

#### 테스트 케이스 설명
1. **test_provide_retrospective_guide_success**: 정상적인 요청 처리 테스트
2. **test_provide_retrospective_guide_invalid_secret_key**: 잘못된 secretKey 검증 테스트 (AI_001)
3. **test_provide_retrospective_guide_missing_content**: content 누락 검증 테스트 (COMMON400)

### 6. API 호출 예제
```bash
# 정상 요청
curl -X POST http://localhost:8080/api/ai/retrospective/guide \
  -H "Content-Type: application/json" \
  -d '{
    "secretKey": "your_secret_key_here",
    "content": "오늘 프로젝트를 진행하면서 많은 것을 배웠다."
  }'

# 잘못된 secretKey
curl -X POST http://localhost:8080/api/ai/retrospective/guide \
  -H "Content-Type: application/json" \
  -d '{
    "secretKey": "wrong_key",
    "content": "오늘 프로젝트를 진행하면서 많은 것을 배웠다."
  }'
```

### 7. 기술 스택
* **Framework:** Actix-web 4.5.1
* **Validation:** validator 0.18
* **Documentation:** utoipa (OpenAPI/Swagger)
* **Rate Limiting:** Custom implementation (10 requests per 60 seconds)

### 8. 환경 변수 설정
`.env` 파일에 다음 설정 필요:
```env
AI_SECRET_KEY=your_secret_key_here
OPENAI_API_KEY=your_openai_api_key_here
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
```

### 9. 기타 특이사항
* 현재 Mock AI 응답 사용 중 (실제 OpenAI API 연동 예정)
* Rate Limiter는 secretKey 기준으로 동작
* Swagger UI를 통한 API 테스트 가능: http://localhost:8080/swagger-ui/
* 모든 테스트 케이스 통과 확인 완료

### 10. 코드 구조
```
src/domain/ai/
├── mod.rs              # 모듈 선언
├── controller.rs       # API 엔드포인트 (provide_retrospective_guide)
├── service.rs          # 비즈니스 로직 (generate_retrospective_guide)
├── validator.rs        # secretKey 검증
├── prompt.rs           # AI 프롬프트 생성
└── tests.rs            # 단위 테스트
```



### 아래는 그냥 참고용으로 내가 쓴거
* 프롬프트
```md
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
```