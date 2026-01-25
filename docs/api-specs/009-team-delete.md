# [API-009] DELETE /api/teams/{teamId}

팀 삭제 API

## 개요

생성된 팀을 완전히 삭제합니다.

- **데이터 파기**: 팀 삭제 시 해당 팀과 연결된 **모든 회고, 답변, 댓글, 초대 코드** 데이터가 영구적으로 삭제됩니다.
- **권한 검증**: 해당 팀의 **관리자(Owner)** 권한을 가진 사용자만 요청할 수 있습니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |

## 엔드포인트

```
DELETE /api/teams/{teamId}
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

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| teamId | long | Yes | 삭제할 팀의 고유 식별자 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "팀 삭제에 성공하였습니다.",
  "result": {
    "teamId": 123,
    "deletedAt": "2026-01-24T22:45:05"
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| teamId | long | 삭제 처리된 팀 고유 ID |
| deletedAt | string | 삭제 완료 일시 (yyyy-MM-ddTHH:mm:ss) |

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

### 403 Forbidden - 권한 없음

```json
{
  "isSuccess": false,
  "code": "TEAM4031",
  "message": "팀을 삭제할 권한이 없습니다.",
  "result": null
}
```

### 404 Not Found - 팀 없음

```json
{
  "isSuccess": false,
  "code": "TEAM4041",
  "message": "존재하지 않는 팀입니다.",
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

| Code | HTTP Status | Description |
|------|-------------|-------------|
| AUTH4001 | 401 | 토큰 누락, 만료 또는 잘못된 형식 |
| TEAM4031 | 403 | 관리자(Owner)가 아닌 일반 멤버가 삭제 시도 |
| TEAM4041 | 404 | 유효하지 않은 teamId이거나 이미 삭제된 경우 |
| COMMON500 | 500 | 연관 데이터 삭제 트랜잭션 처리 중 오류 |

## 사용 예시

### cURL

```bash
curl -X DELETE https://api.example.com/api/teams/123 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}"
```

## 주의사항

- 삭제된 데이터는 복구할 수 없습니다.
- 삭제 전 사용자에게 확인 다이얼로그를 표시하는 것을 권장합니다.
- 팀 관리자(Owner)만 삭제할 수 있습니다.
