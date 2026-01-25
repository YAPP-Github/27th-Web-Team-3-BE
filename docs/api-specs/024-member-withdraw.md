# [API-024] DELETE /api/v1/members/me

서비스 탈퇴 API

## 개요

현재 로그인한 사용자의 계정을 삭제하고 서비스를 탈퇴 처리합니다.

- 탈퇴 시 해당 사용자와 연결된 모든 개인 정보 및 데이터는 즉시 파기되거나 익명화 처리되며, 이는 복구가 불가능합니다.
- 탈퇴 사유(`withdrawalReason`)는 서비스 품질 개선을 위한 통계 자료로 활용됩니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |
| 1.1.0 | 2025-01-25 | withdrawalReason 검증 규칙, 응답 필드 설명, 에러 발생 조건 상세화 |

## 엔드포인트

```
DELETE /api/v1/members/me
```

## 인증

- `Authorization` 헤더를 통한 Bearer 토큰 인증

## Request

### Headers

| Header | Value | Required |
|--------|-------|----------|
| Authorization | Bearer {accessToken} | Yes |

### Body

```json
{
  "withdrawalReason": "더 이상 회고를 작성할 필요가 없습니다."
}
```

### 필드 설명

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| withdrawalReason | string | No | 탈퇴 사유 |

### withdrawalReason 검증 규칙

| 규칙 | 조건 | 설명 |
|------|------|------|
| 최대 길이 | ≤ 200자 | 200자를 초과하는 경우 요청이 거부됩니다. |
| 형식 | UTF-8 문자열 | 특수 문자, 이모지 포함 허용 |
| 선택 사항 | 필수 아님 | 값이 없거나 빈 문자열(`""`) 전달 가능 |
| 공백 처리 | 그대로 저장 | 선행/후행 공백을 포함하여 저장됩니다. |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회원 탈퇴가 성공적으로 완료되었습니다.",
  "result": null
}
```

### 응답 필드 설명

| Field | Type | 설명 |
|-------|------|------|
| isSuccess | boolean | 요청 성공 여부 |
| code | string | 응답 코드 (성공 시 COMMON200) |
| message | string | 사용자 친화적 메시지 |
| result | null | 탈퇴 처리 완료 후 반환할 데이터가 없으므로 null입니다. |

## 에러 응답

### 401 Unauthorized - 인증 실패

```json
{
  "isSuccess": false,
  "code": "AUTH4001",
  "message": "인증 정보가 유효하지 않습니다.",
  "result": null
}
```

**발생 조건:**
- Authorization 헤더가 누락된 경우
- Bearer 토큰이 형식에 맞지 않는 경우
- 토큰이 만료된 경우
- 토큰이 유효하지 않거나 위조된 경우

### 404 Not Found - 사용자 없음

```json
{
  "isSuccess": false,
  "code": "MEMBER4042",
  "message": "존재하지 않는 사용자입니다.",
  "result": null
}
```

**발생 조건:**
- 토큰에 해당하는 사용자 정보가 데이터베이스에 존재하지 않는 경우
- 이미 탈퇴 처리가 완료된 계정으로 재요청하는 경우
- 데이터베이스에서 사용자 레코드가 손상되었거나 삭제된 경우

### 500 Internal Server Error - 서버 에러

```json
{
  "isSuccess": false,
  "code": "COMMON500",
  "message": "서버 내부 오류입니다.",
  "result": null
}
```

**발생 조건:**
- 사용자 데이터 삭제 중 데이터베이스 오류가 발생한 경우
- 사용자와 연결된 관계 데이터 해제 중 오류가 발생한 경우
- 익명화 처리 과정 중 예기치 않은 오류가 발생한 경우
- 서버 내부 로직 처리 중 예외 상황이 발생한 경우

## 에러 코드 요약

| Code | HTTP Status | Description | 발생 조건 |
|------|-------------|-------------|----------|
| AUTH4001 | 401 | 토큰 누락, 만료 또는 잘못된 형식 | Authorization 헤더 누락, 토큰 형식 오류, 토큰 만료/위조 |
| MEMBER4042 | 404 | 이미 탈퇴 처리가 완료된 계정 | 사용자 미존재, 이미 탈퇴 완료, DB 레코드 손상/삭제 |
| COMMON500 | 500 | 데이터 삭제 및 연관 관계 해제 중 DB 에러 | DB 삭제 오류, 관계 해제 오류, 익명화 처리 오류 |

## 사용 예시

### cURL

```bash
curl -X DELETE https://api.example.com/api/v1/members/me \
  -H "Authorization: Bearer {accessToken}" \
  -d '{
    "withdrawalReason": "더 이상 회고를 작성할 필요가 없습니다."
  }'
```

## 주의사항

- 탈퇴된 계정은 복구할 수 없습니다.
- 탈퇴 전 사용자에게 확인 다이얼로그를 표시하는 것을 권장합니다.
- 탈퇴 사유는 선택 사항이지만, 서비스 개선에 도움이 됩니다.
