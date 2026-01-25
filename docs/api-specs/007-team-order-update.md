# [API-007] PATCH /api/v1/teams/order

팀 순서 변경 API (드래그 앤 드롭)

## 개요

사이드바 또는 목록에서 드래그 앤 드롭으로 변경된 팀들의 정렬 순서를 서버에 일괄 저장합니다.

- 사용자가 참여 중인 팀들에 대해서만 순서 변경이 가능합니다.
- `orderIndex`는 낮은 숫자일수록 상단에 노출됨을 의미합니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |

## 엔드포인트

```
PATCH /api/v1/teams/order
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
  "teamOrders": [
    {
      "teamId": 456,
      "orderIndex": 1
    },
    {
      "teamId": 789,
      "orderIndex": 2
    }
  ]
}
```

### 필드 설명

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| teamOrders | array[object] | Yes | 변경된 순서 정보 리스트 | 최소 1개 이상 |
| teamOrders[].teamId | long | Yes | 순서를 변경할 팀 고유 ID | - |
| teamOrders[].orderIndex | integer | Yes | 새로 부여할 정렬 순서 | 1 이상의 정수, 중복 불가 |

> **orderIndex 규칙**:
> - 1부터 시작하는 연속된 정수를 사용합니다.
> - 배열 내 중복된 `orderIndex` 값은 허용되지 않습니다.
> - 사용자가 참여 중인 모든 팀의 순서를 포함할 필요 없이, 변경이 필요한 팀들만 전달합니다.

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "팀 순서가 성공적으로 변경되었습니다.",
  "result": null
}
```

## 에러 응답

### 400 Bad Request - 잘못된 순서 데이터

```json
{
  "isSuccess": false,
  "code": "TEAM4004",
  "message": "잘못된 순서 데이터입니다.",
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
  "code": "TEAM4031",
  "message": "순서를 변경할 권한이 없습니다.",
  "result": null
}
```

### 404 Not Found - 팀 없음

```json
{
  "isSuccess": false,
  "code": "TEAM4041",
  "message": "존재하지 않는 팀 정보가 포함되어 있습니다.",
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
| TEAM4004 | 400 | 중복된 orderIndex 또는 유효하지 않은 형식 | teamOrders 배열 내 orderIndex 중복, 0 이하의 orderIndex 값 |
| AUTH4001 | 401 | 토큰 누락, 만료 또는 잘못된 형식 | Authorization 헤더 누락, 토큰 만료, 잘못된 토큰 형식 |
| TEAM4031 | 403 | 참여하지 않은 팀의 ID 포함 | 요청자가 멤버로 참여하지 않은 팀의 teamId를 포함하여 요청 |
| TEAM4041 | 404 | DB에 존재하지 않는 teamId | 존재하지 않거나 삭제된 팀의 teamId를 포함하여 요청 |
| COMMON500 | 500 | 순서 변경 트랜잭션 처리 중 서버 에러 | 데이터베이스 연결 실패, 트랜잭션 커밋 오류 |

## 사용 예시

### cURL

```bash
curl -X PATCH https://api.example.com/api/v1/teams/order \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}" \
  -d '{
    "teamOrders": [
      { "teamId": 456, "orderIndex": 1 },
      { "teamId": 789, "orderIndex": 2 }
    ]
  }'
```
