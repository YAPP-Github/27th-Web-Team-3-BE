# [API-032] GET /api/v1/retrospects/{retrospectId}/analysis

회고 분석 결과 조회 API

## 개요

이미 생성된 회고 분석 결과를 조회합니다.

- **[API-023] POST /api/v1/retrospects/{retrospectId}/analysis**로 생성된 분석 결과를 조회합니다.
- 분석이 아직 생성되지 않은 경우 404 에러를 반환합니다.
- 회고방 멤버만 분석 결과를 조회할 수 있습니다.

### 분석 결과 저장 위치

| 데이터 | 저장 위치 | 설명 |
|--------|----------|------|
| AI 인사이트 | retrospects.insight | 회고방 전체를 위한 분석 메시지 |
| 감정 랭킹 | 분석 시점에만 생성 | 저장되지 않음 (조회 시 재계산 또는 별도 저장 필요) |
| 개인 미션 | member_retro.personal_insight | 사용자별 맞춤 미션 |

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-02-07 | 최초 작성 |

## 엔드포인트

```
GET /api/v1/retrospects/{retrospectId}/analysis
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
| retrospectId | long | Yes | 분석 결과를 조회할 회고 세션 고유 ID | 1 이상의 양수 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고 분석 결과 조회를 성공했습니다.",
  "result": {
    "insight": "이번 회고에서 팀은 목표 의식은 분명했지만, 에너지 관리 측면에서 아쉬움이 있었습니다.",
    "emotionRank": [
      {
        "rank": 1,
        "label": "피로",
        "description": "짧은 스프린트로 인해 팀 전반에 피로도가 높게 나타났습니다.",
        "count": 6
      },
      {
        "rank": 2,
        "label": "뿌듯",
        "description": "목표 달성에 대한 성취감을 느끼는 팀원이 많았습니다.",
        "count": 4
      },
      {
        "rank": 3,
        "label": "불안",
        "description": "다음 스프린트에 대한 부담감을 가진 팀원들이 있었습니다.",
        "count": 2
      }
    ],
    "personalMissions": [
      {
        "userId": 1,
        "userName": "소은",
        "missions": [
          {
            "missionTitle": "감정 표현 적극적으로 하기",
            "missionDesc": "활발한 협업은 좋았으나 감정 공유를 늘리면 팀 응집력이 더 높아질 것입니다."
          },
          {
            "missionTitle": "스프린트 분량 조절하기",
            "missionDesc": "작은 PR 단위로 나누어 업무를 분배하면 효율적인 리뷰가 가능합니다."
          },
          {
            "missionTitle": "피드백 즉각 공유하기",
            "missionDesc": "즉각적인 응답과 활발한 코드 리뷰로 협업 속도를 높여보세요."
          }
        ]
      },
      {
        "userId": 2,
        "userName": "민수",
        "missions": [
          {
            "missionTitle": "작업 분배 개선하기",
            "missionDesc": "업무 집중도가 높았지만 분산 작업을 통해 번아웃을 예방해보세요."
          },
          {
            "missionTitle": "휴식 시간 확보하기",
            "missionDesc": "집중 작업 후 충분한 휴식을 취하면 지속적인 성과를 낼 수 있습니다."
          },
          {
            "missionTitle": "팀원과 소통 늘리기",
            "missionDesc": "정기적인 체크인을 통해 진행 상황을 공유하면 협업이 원활해집니다."
          }
        ]
      }
    ]
  }
}
```

### 응답 필드

| Field | Type | Description | 용도 |
|-------|------|-------------|------|
| insight | string | 회고방 전체를 위한 AI 분석 메시지 | 회고방 레벨의 통찰력 및 개선점 제시 |
| emotionRank | array[object] | 감정 키워드 순위 리스트 (내림차순 정렬, 정확히 3개) | 팀원들이 느낀 주요 감정 상태 파악 |
| emotionRank[].rank | integer | 순위 (1부터 시작, 감정 빈도 기준 내림차순) | 감정 우선순위 표시 |
| emotionRank[].label | string | 감정 키워드 (예: "피로", "뿌듯") | 감정 카테고리 식별 |
| emotionRank[].description | string | 해당 감정에 대한 상세 설명 및 원인 분석 | 감정이 발생한 맥락 설명 |
| emotionRank[].count | integer | 해당 감정을 선택/언급한 횟수 | 감정의 대표성 정도 표시 |
| personalMissions | array[object] | 사용자별 개인 맞춤 미션 리스트 (userId 오름차순 정렬) | 팀원별 성장 기회 제시 |
| personalMissions[].userId | long | 사용자 고유 ID | 미션 대상자 식별 |
| personalMissions[].userName | string | 사용자 이름 | 미션 대상자 이름 표시 |
| personalMissions[].missions | array[object] | 해당 사용자의 개인 미션 리스트 (정확히 3개) | 사용자별 맞춤 미션 제공 |
| personalMissions[].missions[].missionTitle | string | 개인 미션 제목 (예: "감정 표현 적극적으로 하기") | 미션의 핵심 주제 |
| personalMissions[].missions[].missionDesc | string | 개인 미션 상세 설명 및 인사이트 | 미션 수행 이유 및 기대효과 설명 |

### 배열 검증 규칙

| 배열 필드 | 최소 요소 수 | 최대 요소 수 | 정렬 순서 | 설명 |
|-----------|------------|------------|---------|------|
| emotionRank | 3 | 3 | 감정 빈도 내림차순 (count 기준) | 정확히 3개의 감정 결과 필수 |
| personalMissions | 0 | 제한 없음 | userId 오름차순 | 분석된 팀원이 0명인 경우 빈 배열 |
| personalMissions[].missions | 3 | 3 | 순서 보장 없음 | 사용자당 정확히 3개의 미션 필수 |

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
  "code": "RETRO4031",
  "message": "해당 회고에 접근 권한이 없습니다.",
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

### 404 Not Found - 분석 결과 없음

```json
{
  "isSuccess": false,
  "code": "RETRO4044",
  "message": "회고 분석 결과가 존재하지 않습니다. 먼저 분석을 실행해주세요.",
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
| RETRO4031 | 403 | 접근 권한 없음 | 해당 회고가 속한 회고방의 멤버가 아닌 경우 |
| RETRO4041 | 404 | 존재하지 않는 회고 | 해당 retrospectId의 회고가 DB에 없음 |
| RETRO4044 | 404 | 분석 결과 없음 | 회고 분석이 아직 실행되지 않음 (insight가 null) |
| COMMON500 | 500 | 서버 내부 에러 | DB 연결 실패, 쿼리 오류 등 |

## 사용 예시

### cURL

```bash
curl -X GET https://api.example.com/api/v1/retrospects/100/analysis \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}"
```

## 관련 API

- [API-023] POST /api/v1/retrospects/{retrospectId}/analysis - 회고 분석 생성
- [API-013] GET /api/v1/retrospects/{retrospectId} - 회고 상세 정보 조회

## 구현 시 고려사항

### emotionRank 데이터 저장

현재 분석 생성 API(API-023)에서 emotionRank는 응답에만 포함되고 DB에 저장되지 않습니다.
분석 결과 조회 API를 구현하려면 다음 중 하나를 선택해야 합니다:

1. **별도 테이블 생성**: `retrospect_emotion_rank` 테이블 추가
2. **JSON 필드 저장**: `retrospects.emotion_rank_json` 컬럼 추가
3. **emotionRank 제외**: 조회 시 insight와 personalMissions만 반환

권장: 옵션 2 (JSON 필드)를 사용하여 분석 생성 시 함께 저장
