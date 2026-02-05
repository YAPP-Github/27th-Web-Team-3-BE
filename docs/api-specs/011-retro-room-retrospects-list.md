# [API-011] GET /api/v1/retro-rooms/{retroRoomId}/retrospects

회고방 내 전체 회고 목록 조회 API

## 개요

특정 회고방에 속한 모든 회고 목록을 조회합니다.

- 과거, 오늘, 예정된 회고 데이터가 모두 포함됩니다.
- 클라이언트(프론트엔드)의 유연한 UI 대응을 위해 별도의 필터링 없이 전체 목록을 제공합니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |

## 엔드포인트

```
GET /api/v1/retro-rooms/{retroRoomId}/retrospects
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
| retroRoomId | long | Yes | 조회를 원하는 회고방의 고유 ID | 1 이상의 양수 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고방 내 전체 회고 목록 조회를 성공했습니다.",
  "result": [
    {
      "retrospectId": 100,
      "projectName": "지난 주 프로젝트 회고",
      "retrospectMethod": "PMI",
      "retrospectDate": "2026-01-20",
      "retrospectTime": "10:00"
    },
    {
      "retrospectId": 101,
      "projectName": "오늘 진행할 정기 회고",
      "retrospectMethod": "KPT",
      "retrospectDate": "2026-01-24",
      "retrospectTime": "16:00"
    }
  ]
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| retrospectId | long | 회고 고유 식별자 |
| projectName | string | 프로젝트 이름 |
| retrospectMethod | string (Enum) | 회고 방식 |
| retrospectDate | string | 회고 날짜 (yyyy-MM-dd) |
| retrospectTime | string | 회고 시간 (HH:mm) |

#### retrospectMethod Enum 값

| 값 | 설명 |
|----|------|
| KPT | Keep-Problem-Try 방식 |
| FOUR_L | 4L (Liked, Learned, Lacked, Longed For) 방식 |
| FIVE_F | 5F (Facts, Feelings, Findings, Future, Feedback) 방식 |
| PMI | Plus-Minus-Interesting 방식 |
| FREE | 자유 형식 |

> **정렬 순서**: 응답 배열은 `retrospectDate` + `retrospectTime` 기준 **최신순(내림차순)**으로 정렬됩니다.

### 빈 결과 응답

회고가 없는 경우 빈 배열을 반환합니다.

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고방 내 전체 회고 목록 조회를 성공했습니다.",
  "result": []
}
```

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

### 403 Forbidden - 접근 권한 없음

```json
{
  "isSuccess": false,
  "code": "RETRO4031",
  "message": "해당 회고방에 접근 권한이 없습니다.",
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
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | Authorization 헤더 누락, 토큰 만료, 잘못된 토큰 형식 |
| RETRO4031 | 403 | 해당 회고방에 접근 권한 없음 (멤버가 아님) | 요청자가 해당 회고방의 멤버가 아닌 경우 |
| RETRO4041 | 404 | 존재하지 않는 회고방 | 존재하지 않거나 삭제된 회고방의 retroRoomId로 요청 |
| COMMON500 | 500 | 회고 목록 조회 중 서버 에러 | 데이터베이스 연결 실패, 쿼리 실행 오류 |

## 사용 예시

### cURL

```bash
curl -X GET https://api.example.com/api/v1/retro-rooms/1/retrospects \
  -H "Authorization: Bearer {accessToken}"
```
