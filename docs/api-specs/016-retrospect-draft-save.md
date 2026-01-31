# [API-016] PUT /api/v1/retrospects/{retrospectId}/drafts

회고 답변 임시 저장 API

## 개요

진행 중인 회고의 답변을 임시로 저장합니다.

- 기존에 저장된 내용이 있다면 전달받은 내용으로 **덮어쓰기(Overwrite)** 처리됩니다.
- 5개의 질문 중 일부만 선택하여 저장할 수 있습니다.
- 자동 저장(Auto-save) 로직 구현 시 활용하기에 적합합니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |
| 1.1.0 | 2025-01-25 | 500 에러 추가, drafts 배열 검증 규칙 상세화, content null 허용 여부 명시 |
| 1.2.0 | 2025-01-25 | 날짜 포맷 ISO 8601(YYYY-MM-DD) 통일, 에러 코드 RETRO4031에서 RETRO4031로 변경 |

## 엔드포인트

```
PUT /api/v1/retrospects/{retrospectId}/drafts
```

## 인증

- `Authorization` 헤더를 통한 Bearer 토큰 인증

## Request

### Headers

| Header | Value | Required |
|--------|-------|----------|
| Content-Type | application/json | Yes |
| Authorization | Bearer {accessToken} | Yes |

### Path Parameters

| Parameter | Type | Required | Description | Validation |
|-----------|------|----------|-------------|------------|
| retrospectId | long | Yes | 임시 저장할 회고의 고유 식별자 | 1 이상의 양수 |

### Body

```json
{
  "drafts": [
    {
      "questionNumber": 1,
      "content": "첫 번째 질문에 대한 임시 저장 내용입니다."
    },
    {
      "questionNumber": 3,
      "content": "세 번째 질문도 함께 저장합니다."
    }
  ]
}
```

### 필드 설명

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| drafts | array[object] | Yes | 임시 저장할 답변 데이터 리스트 | 최소 1개, 최대 5개 |
| drafts[].questionNumber | integer | Yes | 질문 번호 | 1 ~ 5 범위의 정수 |
| drafts[].content | string | No | 답변 내용 | 최대 1,000자, null 또는 빈 문자열 허용 |

### drafts 배열 검증 규칙

| 규칙 | 설명 |
|------|------|
| 최소 개수 | 1개 (빈 배열 불가) |
| 최대 개수 | 5개 (질문 개수 초과 불가) |
| 중복 허용 | 동일 questionNumber 중복 불가 |
| content null | 허용됨 (기존 임시 저장 내용 삭제 시 사용) |
| content 빈 문자열 | 허용됨 (빈 상태로 저장) |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "임시 저장이 완료되었습니다.",
  "result": {
    "retrospectId": 101,
    "updatedAt": "2026-01-24"
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| retrospectId | long | 해당 회고의 고유 ID |
| updatedAt | string | 최종 저장 날짜 (YYYY-MM-DD) |

## 에러 응답

### 400 Bad Request - 답변 길이 초과

```json
{
  "isSuccess": false,
  "code": "RETRO4003",
  "message": "답변은 1,000자를 초과할 수 없습니다.",
  "result": null
}
```

### 400 Bad Request - 잘못된 질문 번호

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "올바르지 않은 질문 번호입니다.",
  "result": null
}
```

### 400 Bad Request - 빈 drafts 배열

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "저장할 답변이 최소 1개 이상 필요합니다.",
  "result": null
}
```

### 400 Bad Request - 중복 질문 번호

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "중복된 질문 번호가 포함되어 있습니다.",
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

### 403 Forbidden - 권한 없음

```json
{
  "isSuccess": false,
  "code": "RETRO4031",
  "message": "해당 회고에 작성 권한이 없습니다.",
  "result": null
}
```

### 404 Not Found - 회고 없음

```json
{
  "isSuccess": false,
  "code": "RETRO4041",
  "message": "존재하지 않는 회고입니다.",
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
| RETRO4003 | 400 | 답변 글자 수 제한 초과 | content가 1,000자 초과 |
| COMMON400 | 400 | 잘못된 요청 | questionNumber가 1~5 범위 벗어남, 빈 배열, 중복 질문 번호 |
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | 토큰 누락, 만료 또는 잘못된 Bearer 토큰 |
| RETRO4031 | 403 | 작성 권한 없음 | 해당 회고에 참석자로 등록되지 않은 유저 |
| RETRO4041 | 404 | 존재하지 않는 회고 | 해당 retrospectId의 회고가 DB에 없음 |
| COMMON500 | 500 | 서버 내부 에러 | DB 연결 실패, 트랜잭션 오류 등 |

## 사용 예시

### cURL

```bash
curl -X PUT https://api.example.com/api/v1/retrospects/101/drafts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}" \
  -d '{
    "drafts": [
      {
        "questionNumber": 1,
        "content": "첫 번째 질문에 대한 임시 저장 내용입니다."
      },
      {
        "questionNumber": 3,
        "content": "세 번째 질문도 함께 저장합니다."
      }
    ]
  }'
```
