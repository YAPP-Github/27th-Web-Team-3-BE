# [API-020] GET /api/v1/retrospects/storage

보관함 회고 리스트 조회 API

## 개요

완료된 회고 리스트를 연도별로 그룹화하여 조회합니다.

- 특정 기간 필터(`range`)를 사용하여 조회 범위를 제한할 수 있습니다.
- 결과가 없을 경우 `years` 리스트는 빈 배열(`[]`)로 반환됩니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |
| 1.1.0 | 2025-01-25 | range enum 검증 규칙, retroCategory enum 설명, 정렬 순서, 에러 조건 상세화 |
| 1.2.0 | 2025-01-25 | 날짜 포맷을 ISO 8601(YYYY-MM-DD)로 통일 |

## 엔드포인트

```
GET /api/v1/retrospects/storage
```

## 인증

- `Authorization` 헤더를 통한 Bearer 토큰 인증

## Request

### Headers

| Header | Value | Required |
|--------|-------|----------|
| Authorization | Bearer {accessToken} | Yes |

### Query Parameters

| Parameter | Type | Required | Description | Validation |
|-----------|------|----------|-------------|------------|
| range | string (Enum) | No | 조회 기간 필터 | ALL, 3_MONTHS, 6_MONTHS, 1_YEAR 중 하나 (기본값: ALL) |

### range Enum 설명

| Value | 한글명 | Description | 조회 범위 |
|-------|--------|-------------|-----------|
| ALL | 전체 | 기간 제한 없이 모든 회고 조회 | 무제한 |
| 3_MONTHS | 최근 3개월 | 오늘 기준 3개월 이내 회고 | 오늘 ~ 90일 전 |
| 6_MONTHS | 최근 6개월 | 오늘 기준 6개월 이내 회고 | 오늘 ~ 180일 전 |
| 1_YEAR | 최근 1년 | 오늘 기준 1년 이내 회고 | 오늘 ~ 365일 전 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "보관함 조회를 성공했습니다.",
  "result": {
    "years": [
      {
        "yearLabel": "2026년",
        "retrospects": [
          {
            "retrospectId": 124,
            "displayDate": "2026-01-24",
            "title": "API 명세 표준화 프로젝트",
            "retroCategory": "KPT",
            "memberCount": 5
          }
        ]
      }
    ]
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| years | array[object] | 연도별 그룹 데이터 (최신 연도순 내림차순 정렬) |
| years[].yearLabel | string | 연도 표시 (예: "2026년") |
| years[].retrospects | array[object] | 해당 연도의 회고 리스트 (최신순 내림차순 정렬) |
| years[].retrospects[].retrospectId | long | 회고 고유 식별자 |
| years[].retrospects[].displayDate | string | 화면 표시용 날짜 (ISO 8601 형식: YYYY-MM-DD, 예: "2026-01-24") |
| years[].retrospects[].title | string | 회고 제목 (프로젝트명) |
| years[].retrospects[].retroCategory | string (Enum) | 회고 유형 |
| years[].retrospects[].memberCount | integer | 참여 인원수 |

### retroCategory Enum 설명

| Value | 한글명 | Description |
|-------|--------|-------------|
| KPT | Keep-Problem-Try | 유지할 점, 문제점, 시도할 점 정리 방식 |
| FOUR_L | 4L | Liked-Learned-Lacked-Longed for 방식 |
| FIVE_F | 5F | Facts-Feelings-Findings-Future-Feedback 방식 |
| PMI | Plus-Minus-Interesting | 긍정-부정-흥미로운 점 분류 방식 |
| FREE | 자유 형식 | 형식 제약 없는 자유 작성 |

### 정렬 순서

| 필드 | 정렬 기준 | 순서 |
|------|----------|------|
| years | 연도 | 내림차순 (최신 연도가 상위) |
| retrospects | 회고 일시 | 내림차순 (최신 회고가 상위) |

### 빈 결과 응답

회고가 없는 경우 빈 배열을 반환합니다.

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "보관함 조회를 성공했습니다.",
  "result": {
    "years": []
  }
}
```

## 에러 응답

### 400 Bad Request - 잘못된 필터

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "유효하지 않은 기간 필터입니다.",
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
| COMMON400 | 400 | 잘못된 요청 | range 값이 정의된 Enum(ALL, 3_MONTHS, 6_MONTHS, 1_YEAR) 외의 값 |
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | 토큰 누락, 만료 또는 잘못된 Bearer 토큰 |
| COMMON500 | 500 | 서버 내부 에러 | DB 연결 실패, 쿼리 오류 등 |

## 사용 예시

### cURL

```bash
# 전체 조회
curl -X GET https://api.example.com/api/v1/retrospects/storage \
  -H "Authorization: Bearer {accessToken}"

# 최근 3개월 필터
curl -X GET "https://api.example.com/api/v1/retrospects/storage?range=3_MONTHS" \
  -H "Authorization: Bearer {accessToken}"
```
