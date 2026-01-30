# [API-011] POST /api/v1/retrospects

회고 생성 API

## 개요

진행한 프로젝트에 대한 회고 세션을 생성합니다. 프로젝트 정보, 회고 방식, 참고 자료 등을 포함하며 생성된 회고의 고유 식별자를 반환합니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |
| 1.1.0 | 2025-01-25 | Enum 상세 설명, 검증 규칙, 에러 조건 추가 |
| 1.2.0 | 2025-01-25 | teamId 필드 추가, 날짜 포맷 ISO 8601(YYYY-MM-DD) 통일, 질문 생성 로직 추가 |
| 1.3.0 | 2026-01-30 | teamId → retroRoomId로 변경, retrospectTime 필드 추가 (실제 구현과 동기화) |

## 엔드포인트

```
POST /api/v1/retrospects
```

## 인증

- `Authorization` 헤더를 통한 Bearer 토큰 인증

## Request

### Headers

| Header | Value | Required |
|--------|-------|----------|
| Content-Type | application/json | Yes |
| Authorization | Bearer {accessToken} | Yes |

### Body

```json
{
  "retroRoomId": 789,
  "projectName": "나만의 회고 플랫폼",
  "retrospectDate": "2026-01-24",
  "retrospectTime": "14:00",
  "retrospectMethod": "KPT",
  "referenceUrls": [
    "https://github.com/jayson/project",
    "https://notion.so/retrospective-guide"
  ]
}
```

### 필드 설명

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| retroRoomId | long | Yes | 회고가 속한 회고방의 고유 ID | 1 이상의 양수 |
| projectName | string | Yes | 프로젝트 이름 | 최소 1자, 최대 20자 |
| retrospectDate | string | Yes | 회고 날짜 | ISO 8601 형식 (YYYY-MM-DD) |
| retrospectTime | string | Yes | 회고 시간 (한국 시간 기준) | HH:mm 형식 (예: 14:00) |
| retrospectMethod | string (Enum) | Yes | 회고 방식 | KPT, FOUR_L, FIVE_F, PMI, FREE 중 하나 |
| referenceUrls | array[string] | No | 참고 자료 URL 리스트 | 최대 10개, 각 URL은 유효한 형식이어야 함 (http/https) |

### referenceUrls 검증 규칙

| 규칙 | 설명 |
|------|------|
| 최대 개수 | 10개 |
| URL 형식 | http:// 또는 https://로 시작하는 유효한 URL |
| 최대 길이 | 각 URL 최대 2,048자 |
| 중복 허용 | 동일 URL 중복 등록 불가 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고가 성공적으로 생성되었습니다.",
  "result": {
    "retrospectId": 12345,
    "retroRoomId": 789,
    "projectName": "나만의 회고 플랫폼"
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| retrospectId | long | 생성된 회고 고유 ID |
| retroRoomId | long | 회고가 속한 회고방의 고유 ID |
| projectName | string | 저장된 프로젝트 이름 |

### retrospectMethod Enum 설명

| Value | 한글명 | Description | 사용 시나리오 |
|-------|--------|-------------|---------------|
| KPT | Keep-Problem-Try | 유지할 점, 문제점, 시도할 점을 정리하는 방식 | 팀 프로젝트 후 빠른 피드백이 필요할 때 |
| FOUR_L | 4L | Liked-Learned-Lacked-Longed for 방식 | 개인 성장과 감정 회고에 적합 |
| FIVE_F | 5F | Facts-Feelings-Findings-Future-Feedback 방식 | 종합적인 프로젝트 분석이 필요할 때 |
| PMI | Plus-Minus-Interesting | 긍정-부정-흥미로운 점을 분류하는 방식 | 빠른 의사결정 후 검토에 적합 |
| FREE | 자유 형식 | 형식 제약 없이 자유롭게 작성 | 유연한 회고가 필요할 때 |

### 회고 방식별 기본 질문 생성 로직

회고 생성 시 선택한 `retrospectMethod`에 따라 다음과 같은 기본 질문이 자동으로 생성됩니다.

#### KPT (Keep-Problem-Try)

| 질문 순서 | 질문 내용 |
|----------|----------|
| 1 | 계속 유지하고 싶은 좋은 점은 무엇인가요? |
| 2 | 개선이 필요한 문제점은 무엇인가요? |
| 3 | 다음에 시도해보고 싶은 것은 무엇인가요? |
| 4 | 전체 프로젝트를 돌아보며 느낀 점을 자유롭게 작성해주세요. |
| 5 | 추가로 공유하고 싶은 의견이 있나요? |

#### FOUR_L (Liked-Learned-Lacked-Longed for)

| 질문 순서 | 질문 내용 |
|----------|----------|
| 1 | 프로젝트에서 좋았던 점은 무엇인가요? |
| 2 | 새롭게 배운 것은 무엇인가요? |
| 3 | 부족했던 점은 무엇인가요? |
| 4 | 앞으로 바라는 것은 무엇인가요? |
| 5 | 추가로 공유하고 싶은 의견이 있나요? |

#### FIVE_F (Facts-Feelings-Findings-Future-Feedback)

| 질문 순서 | 질문 내용 |
|----------|----------|
| 1 | 프로젝트의 객관적 사실은 무엇인가요? |
| 2 | 프로젝트 진행 중 어떤 감정을 느꼈나요? |
| 3 | 프로젝트에서 발견한 것은 무엇인가요? |
| 4 | 앞으로 어떻게 적용할 수 있을까요? |
| 5 | 팀원에게 전하고 싶은 피드백이 있나요? |

#### PMI (Plus-Minus-Interesting)

| 질문 순서 | 질문 내용 |
|----------|----------|
| 1 | 긍정적이었던 점은 무엇인가요? |
| 2 | 부정적이었던 점은 무엇인가요? |
| 3 | 흥미로웠던 점은 무엇인가요? |
| 4 | 전체 프로젝트를 돌아보며 느낀 점을 자유롭게 작성해주세요. |
| 5 | 추가로 공유하고 싶은 의견이 있나요? |

#### FREE (자유 형식)

| 질문 순서 | 질문 내용 |
|----------|----------|
| 1 | 이번 프로젝트에서 가장 기억에 남는 것은 무엇인가요? |
| 2 | 프로젝트를 진행하며 어떤 생각이 들었나요? |
| 3 | 다음 프로젝트에서 개선하고 싶은 점은 무엇인가요? |
| 4 | 팀원들에게 전하고 싶은 말이 있나요? |
| 5 | 추가로 공유하고 싶은 의견이 있나요? |

## 에러 응답

### 400 Bad Request - 프로젝트 이름 길이 초과

```json
{
  "isSuccess": false,
  "code": "RETRO4001",
  "message": "프로젝트 이름은 20자를 초과할 수 없습니다.",
  "result": null
}
```

### 400 Bad Request - 날짜 형식 오류

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "날짜 형식이 올바르지 않습니다. (YYYY-MM-DD 형식 필요)",
  "result": null
}
```

### 400 Bad Request - 유효하지 않은 회고방 ID

```json
{
  "isSuccess": false,
  "code": "RETRO_ROOM4041",
  "message": "존재하지 않는 회고방입니다.",
  "result": null
}
```

### 403 Forbidden - 회고방 접근 권한 없음

```json
{
  "isSuccess": false,
  "code": "RETRO_ROOM4031",
  "message": "해당 회고방의 멤버가 아닙니다.",
  "result": null
}
```

### 400 Bad Request - 유효하지 않은 회고 방식

```json
{
  "isSuccess": false,
  "code": "RETRO4005",
  "message": "유효하지 않은 회고 방식입니다.",
  "result": null
}
```

### 400 Bad Request - 참고 URL 형식 오류

```json
{
  "isSuccess": false,
  "code": "RETRO4006",
  "message": "유효하지 않은 URL 형식입니다.",
  "result": null
}
```

### 401 Unauthorized - 인증 실패

```json
{
  "isSuccess": false,
  "code": "AUTH4001",
  "message": "인증 정보가 유효하지 않습니다.",
  "result": null
}
```

### 500 Internal Server Error - 서버 에러

```json
{
  "isSuccess": false,
  "code": "COMMON500",
  "message": "서버 내부 오류입니다.",
  "result": null
}
```

## 에러 코드 요약

| Code | HTTP Status | Description | 발생 조건 |
|------|-------------|-------------|-----------|
| RETRO4001 | 400 | 프로젝트 이름 길이 유효성 검사 실패 | projectName이 0자 또는 20자 초과 |
| RETRO4005 | 400 | 유효하지 않은 회고 방식 | retrospectMethod가 정의된 Enum 외의 값 |
| RETRO4006 | 400 | 유효하지 않은 URL 형식 | referenceUrls 중 http/https가 아닌 URL 포함 |
| COMMON400 | 400 | 잘못된 요청 | 날짜/시간 형식 오류(YYYY-MM-DD, HH:mm), 필수 필드 누락 등 |
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | 토큰 누락, 만료, 또는 잘못된 형식 |
| RETRO_ROOM4031 | 403 | 회고방 접근 권한 없음 | 해당 회고방의 멤버가 아닌 경우 |
| RETRO_ROOM4041 | 404 | 존재하지 않는 회고방 | 유효하지 않은 retroRoomId |
| COMMON500 | 500 | 서버 내부 에러 | DB 연결 실패, 트랜잭션 오류 등 |

## 사용 예시

### cURL

```bash
curl -X POST https://api.example.com/api/v1/retrospects \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}" \
  -d '{
    "retroRoomId": 789,
    "projectName": "나만의 회고 플랫폼",
    "retrospectDate": "2026-01-24",
    "retrospectTime": "14:00",
    "retrospectMethod": "KPT",
    "referenceUrls": [
      "https://github.com/jayson/project",
      "https://notion.so/retrospective-guide"
    ]
  }'
```

