# Web3 Server API Documentation

이 문서는 Rust로 마이그레이션된 Web3 Server의 API 명세를 설명합니다.
모든 AI 관련 API는 인증을 위해 `secretKey`를 요청 본문에 포함해야 합니다.

## 1. 공통 응답 형식 (Base Response)

대부분의 API(Health Check 제외)는 아래와 같은 공통 응답 형식을 따릅니다.

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": { ... } // 실제 응답 데이터
}
```

- **isSuccess**: 요청 성공 여부 (`true` / `false`)
- **code**: 응답 코드 (성공 시 `COMMON200`, 에러 시 별도 코드)
- **message**: 응답 메시지
- **result**: 성공 시 반환되는 실제 데이터 (에러 시 `null`)

---

## 2. API 상세 명세

### 2.1. 헬스 체크 (Health Check)

서버의 상태와 의존성 서비스(OpenAI API 등)의 연결 상태를 확인합니다.

- **URL**: `/health`
- **Method**: `GET`
- **Authentication**: 없음

#### 성공 응답 (200 OK)

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptimeSecs": 3600,
  "checks": {
    "openaiApi": {
      "status": true,
      "latencyMs": 150
    }
  }
}
```

- **status**: 서버 전체 상태 (`healthy`, `degraded`, `unhealthy`)
- **uptimeSecs**: 서버 가동 시간 (초)
- **checks**: 각 의존성 서비스별 상태
    - **openaiApi**: OpenAI 연결 상태 (`status`: 성공여부, `latencyMs`: 응답속도)

---

### 2.2. 회고 작성 가이드 (Provide Guide)

사용자가 작성 중인 회고 내용을 분석하여, 더 풍부한 회고를 작성할 수 있도록 가이드 질문이나 조언을 제공합니다.

- **URL**: `/api/ai/retrospective/guide`
- **Method**: `POST`
- **Authentication**: `secretKey` 필수

#### 요청 (Request)

```json
{
  "currentContent": "오늘 프로젝트를 진행하면서 새로운 라이브러리를 적용해봤는데 생각보다 어려웠다.",
  "secretKey": "your-secret-key"
}
```

- **currentContent**: 현재까지 작성한 회고 내용 (1~5000자)
- **secretKey**: API 사용 권한을 확인하기 위한 비밀 키

#### 성공 응답 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "currentContent": "오늘 프로젝트를 진행하면서 새로운 라이브러리를 적용해봤는데 생각보다 어려웠다.",
    "guideMessage": "어떤 부분이 가장 어려웠나요? 문제를 해결하기 위해 시도했던 방법들과 그 과정에서 배운 점을 구체적으로 적어보면 좋을 것 같아요."
  }
}
```

- **guideMessage**: AI가 생성한 코칭/가이드 메시지

---

### 2.3. 회고 말투 정제 (Refine Retrospective)

작성된 회고 내용을 선택한 어조(상냥체/정중체)로 다듬어줍니다.

- **URL**: `/api/ai/retrospective/refine`
- **Method**: `POST`
- **Authentication**: `secretKey` 필수

#### 요청 (Request)

```json
{
  "content": "오늘 배포하다가 에러나서 힘들었음. 근데 해결해서 다행임.",
  "toneStyle": "KIND",
  "secretKey": "your-secret-key"
}
```

- **content**: 정제할 원본 회고 내용 (1~5000자)
- **toneStyle**: 원하는 말투 스타일
    - `"KIND"`: 상냥체 (예: "~했어요")
    - `"POLITE"`: 정중체 (예: "~했습니다")
- **secretKey**: API 사용 권한을 확인하기 위한 비밀 키

#### 성공 응답 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "originalContent": "오늘 배포하다가 에러나서 힘들었음. 근데 해결해서 다행임.",
    "refinedContent": "오늘 배포 과정에서 에러가 발생해서 많이 힘들었어요. 하지만 무사히 해결해서 정말 다행이에요.",
    "toneStyle": "KIND"
  }
}
```

- **refinedContent**: 정제된 회고 내용
- **toneStyle**: 적용된 말투 스타일

---

## 3. 에러 처리

API 호출 중 문제가 발생하면 다음과 같은 에러 응답이 반환됩니다.

#### 예시: 잘못된 비밀 키 (401 Unauthorized)

```json
{
  "isSuccess": false,
  "code": "AI_001",
  "message": "유효하지 않은 비밀 키입니다.",
  "result": null
}
```

#### 주요 에러 코드
- **AI_001**: 유효하지 않은 Secret Key
- **AI_002**: OpenAI API 호출 실패
- **COMMON400**: 잘못된 요청 형식 (유효성 검증 실패 등)
- **COMMON500**: 내부 서버 오류

---

## 4. API 동작 검증 (테스트 실행)

실제 OpenAI API 키 없이도 로컬에서 API 동작 로직을 검증할 수 있습니다.
프로젝트에는 `MockAiClient`를 사용한 단위/통합 테스트가 포함되어 있어, 요청/응답 구조와 에러 처리가 의도대로 동작하는지 확인할 수 있습니다.

### 테스트 실행 방법

`rust` 디렉토리에서 다음 명령어를 실행하세요.

```bash
cd rust
cargo test
```

### 주요 테스트 파일
- **rust/tests/api_test.rs**: API 엔드포인트의 HTTP 상태 코드 및 응답 형식을 검증합니다.
- **rust/tests/handler_handler.rs**: Mock AI 클라이언트를 사용하여 성공/실패 시나리오를 시뮬레이션합니다.
