# [API-024] GET /api/v1/retrospects/search

보관함 회고 검색 API

## 개요

내가 참여한 모든 팀의 회고 중, 프로젝트 이름(회고 이름)에 검색 키워드가 포함된 회고 목록을 조회합니다.

- 과거, 오늘, 예정된 회고가 모두 검색 대상에 포함됩니다.
- 보관함(아카이브) 성격상 특정 팀에 국한되지 않고, **내가 속한 모든 팀**에서 검색합니다.
- 검색 결과는 `retrospectDate` 기준 내림차순(최신순)으로 정렬됩니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |
| 1.1.0 | 2025-01-25 | 날짜 포맷을 ISO 8601(YYYY-MM-DD)로 명시 |

## 엔드포인트

```
GET /api/v1/retrospects/search
```

## 인증

- `Authorization` 헤더를 통한 Bearer 토큰 인증

## Request

### Headers

| Header | Value | Required |
|--------|-------|----------|
| Authorization | Bearer {accessToken} | Yes |

### Query Parameters

| Parameter | Type | Required | Description | 검증 규칙 |
|-----------|------|----------|-------------|----------|
| keyword | string | Yes | 검색할 키워드 | 최소 1자, 최대 100자 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고 검색 결과 조회 성공.",
  "result": [
    {
      "retrospectId": 201,
      "projectName": "AI 서비스 1차 릴리즈 회고",
      "teamName": "AI 사이드팀",
      "retrospectMethod": "KPT",
      "retrospectDate": "2026-01-10",
      "retrospectTime": "14:00"
    },
    {
      "retrospectId": 305,
      "projectName": "AI 챗봇 고도화 회고",
      "teamName": "코드 마스터즈",
      "retrospectMethod": "FOUR_L",
      "retrospectDate": "2026-02-05",
      "retrospectTime": "11:30"
    }
  ]
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| retrospectId | long | 회고 고유 ID |
| projectName | string | 회고 이름 (프로젝트명) |
| teamName | string | 해당 회고가 속한 팀 이름 |
| retrospectMethod | string | 회고 방식 |
| retrospectDate | string | 회고 날짜 (ISO 8601 형식: YYYY-MM-DD) |
| retrospectTime | string | 회고 시간 (HH:mm) |

### retrospectMethod Enum 설명

| Value | 한글명 | Description |
|-------|--------|-------------|
| KPT | Keep-Problem-Try | 유지할 점, 문제점, 시도할 점 정리 방식 |
| FOUR_L | 4L | Liked-Learned-Lacked-Longed for 방식 |
| FIVE_F | 5F | Facts-Feelings-Findings-Future-Feedback 방식 |
| PMI | Plus-Minus-Interesting | 긍정-부정-흥미로운 점 분류 방식 |
| FREE | 자유 형식 | 형식 제약 없이 자유롭게 작성 |

### 배열 정렬 순서

검색 결과 배열은 다음 기준으로 정렬됩니다:

| 기준 | 정렬 방향 | 우선순위 |
|------|---------|---------|
| retrospectDate | 내림차순 (최신순) | 1순위 |
| retrospectTime | 내림차순 | 2순위 (같은 날짜일 경우) |

### 빈 결과 응답

검색 결과가 없는 경우 빈 배열을 반환합니다.

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고 검색 결과 조회 성공.",
  "result": []
}
```

## 에러 응답

### 400 Bad Request - 검색어 누락

```json
{
  "isSuccess": false,
  "code": "SEARCH4001",
  "message": "검색어를 입력해주세요.",
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

### 500 Internal Server Error - 서버 내부 오류

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
|------|-------------|-------------|---------|
| SEARCH4001 | 400 | 검색어를 입력해주세요. | keyword 파라미터가 비어있거나 길이가 100자를 초과 |
| AUTH4001 | 401 | 인증 정보가 유효하지 않습니다. | Authorization 헤더가 없거나 토큰이 만료됨 |
| COMMON500 | 500 | 서버 내부 오류입니다. | 예상치 못한 서버 에러 발생 |

## 사용 예시

### cURL

```bash
curl -X GET "https://api.example.com/api/v1/retrospects/search?keyword=AI" \
  -H "Authorization: Bearer {accessToken}"
```

## 개발 참고사항

1. **검색 범위**: 현재는 `projectName`만 검색 대상입니다. 회고 내용(답변)까지 검색 시 서버 부하가 커질 수 있으므로 초기에는 제목 검색으로 시작합니다.
2. **팀 이름 포함**: 보관함은 여러 팀의 회고가 섞여 나오므로, 검색 결과에 해당 회고가 어느 팀의 회고인지 함께 표시합니다.
3. **날짜순 정렬**: 검색 결과는 기본적으로 `retrospectDate` 기준 내림차순(최신순)으로 정렬됩니다.
4. **페이징**: 회고 데이터가 많아지면 `page`, `size` 파라미터를 추가하여 페이징 처리를 고려할 수 있습니다.
