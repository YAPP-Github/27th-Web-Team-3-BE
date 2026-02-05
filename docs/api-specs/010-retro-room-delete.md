# [API-010] DELETE /api/v1/retro-rooms/{retroRoomId}

회고방 삭제 API

## 개요

생성된 회고방을 완전히 삭제합니다.

- **데이터 파기**: 회고방 삭제 시 해당 룸과 연결된 **모든 회고, 답변, 댓글, 초대 코드** 데이터가 영구적으로 삭제됩니다.
- **권한 검증**: 해당 회고방의 **관리자(Owner)** 권한을 가진 사용자만 요청할 수 있습니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |

## 엔드포인트

```
DELETE /api/v1/retro-rooms/{retroRoomId}
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
| retroRoomId | long | Yes | 삭제할 회고방의 고유 식별자 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고방 삭제에 성공하였습니다.",
  "result": {
    "retroRoomId": 123,
    "deletedAt": "2026-01-24T22:45:05"
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| retroRoomId | long | 삭제 처리된 회고방 고유 ID |
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
  "code": "RETRO4031",
  "message": "회고방을 삭제할 권한이 없습니다.",
  "result": null
}
```

### 404 Not Found - 회고방 없음

```json
{
  "isSuccess": false,
  "code": "RETRO4041",
  "message": "존재하지 않는 회고방입니다.",
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
|------|-------------|-------------|----------|
| AUTH4001 | 401 | 토큰 누락, 만료 또는 잘못된 형식 | Authorization 헤더 누락, 토큰 만료, 잘못된 토큰 형식 |
| RETRO4031 | 403 | 관리자(Owner)가 아닌 일반 멤버가 삭제 시도 | 회고방 관리자(Owner) 권한이 없는 사용자가 삭제 요청 |
| RETRO4041 | 404 | 유효하지 않은 retroRoomId이거나 이미 삭제된 경우 | 존재하지 않거나 이미 삭제된 회고방의 retroRoomId로 요청 |
| COMMON500 | 500 | 연관 데이터 삭제 트랜잭션 처리 중 오류 | 데이터베이스 연결 실패, 연관 데이터 삭제 중 트랜잭션 오류 |

## 사용 예시

### cURL

```bash
curl -X DELETE https://api.example.com/api/v1/retro-rooms/123 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}"
```

## 주의사항

- 삭제된 데이터는 복구할 수 없습니다.
- 삭제 전 사용자에게 확인 다이얼로그를 표시하는 것을 권장합니다.
- 회고방 관리자(Owner)만 삭제할 수 있습니다.
