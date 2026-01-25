# [API-013] DELETE /api/v1/retrospects/{retrospectId}

회고 삭제 API

## 개요

진행된 특정 회고 세션을 완전히 삭제합니다.

- **데이터 파기**: 삭제 시 해당 회고와 연결된 **모든 답변, 댓글, 좋아요, AI 분석 결과**가 영구적으로 삭제됩니다.
- **권한 제한**: 해당 팀의 관리자(Owner) 또는 해당 회고를 생성한 유저만 삭제가 가능합니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |
| 1.1.0 | 2025-01-25 | Path Parameter 검증, 권한 조건 상세화, 응답 필드 설명 보완 |

## 엔드포인트

```
DELETE /api/v1/retrospects/{retrospectId}
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
| retrospectId | long | Yes | 삭제를 진행할 회고 세션 고유 ID | 1 이상의 양수 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고가 성공적으로 삭제되었습니다.",
  "result": null
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| isSuccess | boolean | API 호출 성공 여부 |
| code | string | 응답 코드 (성공 시 COMMON200) |
| message | string | 응답 메시지 |
| result | null | 삭제 요청 시 항상 null 반환 |

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
  "message": "해당 회고를 삭제할 권한이 없습니다.",
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

### 500 Internal Server Error - 삭제 실패

```json
{
  "isSuccess": false,
  "code": "COMMON500",
  "message": "데이터 삭제 중 서버 에러가 발생했습니다.",
  "result": null
}
```

## 에러 코드 요약

| Code | HTTP Status | Description | 발생 조건 |
|------|-------------|-------------|-----------|
| COMMON400 | 400 | 잘못된 요청 | retrospectId가 0 이하의 값 |
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | 토큰 누락, 만료 또는 잘못된 Bearer 토큰 |
| RETRO4031 | 403 | 삭제 권한 없음 | 아래 권한 조건 미충족 시 |
| RETRO4041 | 404 | 존재하지 않는 회고 | 해당 retrospectId의 회고가 DB에 없음 |
| COMMON500 | 500 | 서버 내부 에러 | 연관 데이터 삭제 트랜잭션 처리 중 오류 |

## 권한 조건

삭제 권한이 있는 사용자:

| 역할 | 조건 |
|------|------|
| 팀 관리자 (Owner) | 해당 회고가 속한 팀의 Owner 역할인 경우 |
| 회고 생성자 | 해당 회고를 직접 생성한 사용자인 경우 |

**참고**: 팀 멤버라도 위 조건을 충족하지 않으면 삭제 불가

## 사용 예시

### cURL

```bash
curl -X DELETE https://api.example.com/api/v1/retrospects/100 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}"
```

## 주의사항

- 삭제된 데이터는 복구할 수 없습니다.
- 삭제 전 사용자에게 확인 다이얼로그를 표시하는 것을 권장합니다.
- 팀 관리자 또는 회고 생성자만 삭제할 수 있습니다.
- 삭제 시 연관된 모든 데이터(답변, 댓글, 좋아요, AI 분석 결과)가 함께 삭제됩니다.
