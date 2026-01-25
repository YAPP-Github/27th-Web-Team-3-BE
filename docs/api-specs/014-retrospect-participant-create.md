# [API-014] POST /api/retrospects/{retrospectId}/participants

회고 참석 API

## 개요

진행 예정인 회고에 참석자로 등록합니다.

- 별도의 Request Body 없이, 헤더의 JWT(Bearer)에서 유저 정보를 추출하여 등록을 처리합니다.
- 해당 회고가 속한 팀의 멤버만 참석이 가능합니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |
| 1.1.0 | 2025-01-25 | 500 에러 추가, 응답 필드 상세화, 에러 발생 조건 테이블 추가 |

## 엔드포인트

```
POST /api/retrospects/{retrospectId}/participants
```

## 인증

- `Authorization` 헤더를 통한 Bearer 토큰 인증

## Request

### Headers

| Header | Value | Required |
|--------|-------|----------|
| Authorization | Bearer {accessToken} | Yes |

### Path Parameters

| Parameter | Type | Required | Description | Validation |
|-----------|------|----------|-------------|------------|
| retrospectId | long | Yes | 참여하고자 하는 회고의 고유 ID | 1 이상의 양수 |

### Body

Request Body 없음 (JWT에서 유저 정보 추출)

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고 참석자로 성공적으로 등록되었습니다.",
  "result": {
    "participantId": 5001,
    "memberId": 123,
    "nickname": "제이슨"
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| participantId | long | 참석자 등록 고유 식별자 (참석 내역 조회/취소 시 사용) |
| memberId | long | 참석한 유저의 고유 ID (유저 프로필 식별자) |
| nickname | string | 참석한 유저의 닉네임 (화면 표시용) |

## 에러 응답

### 400 Bad Request - 잘못된 Path Parameter

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "retrospectId는 1 이상의 양수여야 합니다.",
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
  "message": "해당 회고가 속한 팀의 멤버가 아닙니다.",
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

### 409 Conflict - 중복 참석

```json
{
  "isSuccess": false,
  "code": "RETRO4091",
  "message": "이미 참석자로 등록되어 있습니다.",
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
| COMMON400 | 400 | 잘못된 요청 | retrospectId가 0 이하의 값 |
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | 토큰 누락, 만료 또는 잘못된 Bearer 토큰 |
| RETRO4031 | 403 | 팀 멤버가 아님 | JWT의 유저가 해당 회고가 속한 팀의 멤버가 아닌 경우 |
| RETRO4041 | 404 | 존재하지 않는 회고 | 해당 retrospectId의 회고가 DB에 없음 |
| RETRO4091 | 409 | 중복 참석 | 동일 유저가 동일 회고에 이미 참석 등록된 경우 |
| COMMON500 | 500 | 서버 내부 에러 | DB 연결 실패, 트랜잭션 오류 등 |

## 사용 예시

### cURL

```bash
curl -X POST https://api.example.com/api/retrospects/100/participants \
  -H "Authorization: Bearer {accessToken}"
```
