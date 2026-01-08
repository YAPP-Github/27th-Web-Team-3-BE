# 회고 말투 정제 API

## Endpoint
```
POST /api/ai/retrospective/refine
```

## Description
작성된 회고를 AI가 선택한 말투(상냥체/정중체)로 정제합니다.

---

## Request

### Headers
```yaml
Content-Type: application/json
```

### Body Schema
```yaml
content:
  type: String
  required: true
  description: 정제할 회고 내용

toneStyle:
  type: Enum
  required: true
  description: 말투 스타일
  values:
    - KIND: 상냥체 (친근하고 상냥한 표현)
    - POLITE: 정중체 (존댓말 등 정중한 표현)

secretKey:
  type: String
  required: true
  description: 비밀 키 (인증용)
```

### Example Request
```json
{
  "content": "오늘 일 힘들었음 근데 배운게 많았어",
  "toneStyle": "KIND",
  "secretKey": "mySecretKey123"
}
```

---

## Response

### Success (200)
```yaml
Schema:
  isSuccess: Boolean
  code: String
  message: String
  result:
    originalContent: String  # 원본 내용
    refinedContent: String   # 정제된 내용
    toneStyle: String        # 적용된 말투 스타일
```

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "originalContent": "오늘 일 힘들었음 근데 배운게 많았어",
    "refinedContent": "오늘 업무가 힘들었지만, 그만큼 많은 것을 배울 수 있었어요.",
    "toneStyle": "KIND"
  }
}
```

### Error Responses
```yaml
400_INVALID_REQUEST:
  code: COMMON400
  message: 잘못된 요청입니다.
  cause: 필수 값 누락

400_INVALID_TONE_STYLE:
  code: AI_002
  message: 유효하지 않은 말투 스타일입니다. KIND 또는 POLITE만 가능합니다.
  cause: toneStyle 값이 KIND 또는 POLITE가 아님

401_INVALID_SECRET_KEY:
  code: AI_001
  message: 유효하지 않은 비밀 키입니다.
  cause: secretKey가 유효하지 않음

500_SERVER_ERROR:
  code: COMMON500
  message: 서버 에러, 관리자에게 문의 바랍니다.
  cause: 서버 내부 오류
```

---

## Implementation Details

### Files Structure
```
src/domain/ai/
├── controller.rs    # API endpoint handlers
├── service.rs       # OpenAI integration logic
├── validator.rs     # Secret key validation
└── prompt.rs        # AI prompt templates
```

### Key Functions
- **Controller**: `refine_retrospective()` - Handles POST /api/ai/retrospective/refine
- **Service**: `refine_content()` - Calls OpenAI API with appropriate prompts
- **Validator**: `validate_secret_key()` - Validates the secret key
- **Prompt**: `get_refine_system_prompt()`, `get_refine_user_prompt()` - Generate AI prompts

### OpenAI Model
- Model: `gpt-4o-mini`
- Temperature: `0.7`

### Testing
```bash
curl -X POST http://localhost:8080/api/ai/retrospective/refine \
  -H "Content-Type: application/json" \
  -d '{
    "content": "오늘 일 힘들었음 근데 배운게 많았어",
    "toneStyle": "KIND",
    "secretKey": "your-secret-key"
  }'
```
