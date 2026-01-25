# [API-018] GET /api/retrospects/{retrospectId}/references

회고 참고자료 목록 조회 API

## 개요

특정 회고에 등록된 모든 참고자료(URL) 목록을 조회합니다.

- 회고 생성 시 등록했던 외부 링크들을 확인할 수 있습니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |
| 1.1.0 | 2025-01-25 | 에러 발생 조건 명시, 정렬 순서 명시, URL 필드 제약 조건 추가 |

## 엔드포인트

```
GET /api/retrospects/{retrospectId}/references
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
| retrospectId | long | Yes | 조회를 원하는 회고의 고유 ID | 1 이상의 양수 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "참고자료 목록을 성공적으로 조회했습니다.",
  "result": [
    {
      "referenceId": 1,
      "urlName": "프로젝트 저장소",
      "url": "https://github.com/jayson/my-project"
    },
    {
      "referenceId": 2,
      "urlName": "기획 문서",
      "url": "https://notion.so/doc/123"
    }
  ]
}
```

### 응답 필드

| Field | Type | Description | 제약 조건 |
|-------|------|-------------|-----------|
| referenceId | long | 자료 고유 식별자 | - |
| urlName | string | 자료 별칭 (예: 깃허브 레포지토리) | 최대 50자 |
| url | string | 참고자료 주소 | http/https URL, 최대 2,048자 |

### 정렬 순서

| 기준 | 순서 |
|------|------|
| referenceId | 오름차순 (등록 순서대로) |

### 빈 결과 응답

참고자료가 없는 경우 빈 배열을 반환합니다.

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "참고자료 목록을 성공적으로 조회했습니다.",
  "result": []
}
```

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

### 403 Forbidden - 접근 권한 없음

```json
{
  "isSuccess": false,
  "code": "TEAM4031",
  "message": "해당 팀에 접근 권한이 없습니다.",
  "result": null
}
```

### 404 Not Found - 회고 없음

```json
{
  "isSuccess": false,
  "code": "RETRO4041",
  "message": "존재하지 않는 회고 세션입니다.",
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
| TEAM4031 | 403 | 접근 권한 없음 | JWT의 유저가 해당 회고가 속한 팀의 멤버가 아닌 경우 |
| RETRO4041 | 404 | 존재하지 않는 회고 세션 | 해당 retrospectId의 회고가 DB에 없음 |
| COMMON500 | 500 | 서버 내부 에러 | DB 연결 실패, 쿼리 오류 등 |

## 사용 예시

### cURL

```bash
curl -X GET https://api.example.com/api/retrospects/100/references \
  -H "Authorization: Bearer {accessToken}"
```
