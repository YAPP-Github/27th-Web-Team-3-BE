# [API-022] POST /api/v1/retrospects/{retrospectId}/analysis

회고 종합 분석 API (AI 인사이트)

## 개요

특정 회고 세션에 쌓인 모든 회고 참여자의 답변을 종합 분석하여 AI 인사이트, 감정 통계, 맞춤형 미션을 생성합니다.

- 분석 데이터가 충분하지 않거나 월간 사용량을 초과한 경우 에러를 반환합니다.
- **Path Variable** 방식을 사용하여 특정 회고 리소스에 대한 분석임을 명확히 합니다.

### 최소 데이터 기준

AI 분석을 수행하기 위한 최소 데이터 요구사항:

| 기준 | 최소값 | 설명 |
|------|-------|------|
| 답변 수 | 3개 이상 | 모든 질문의 답변을 합산한 총 개수 |
| 참여자 수 | 1명 이상 | 회고에 답변을 작성한 고유 사용자 수 |

최소 기준을 충족하지 못하면 `RETRO4042` 에러를 반환합니다.

### 월간 한도 기준

| 항목 | 기준 | 설명 |
|------|------|------|
| 한도 단위 | 회고방당 월 10회 | 회고방별로 독립적으로 카운트 |
| 리셋 기준 | 매월 1일 00:00 KST | 한국 표준시(UTC+9) 기준 |
| 한도 초과 시 | AI4031 에러 반환 | 다음 달 1일 00:00 KST부터 사용 가능 |

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |
| 1.1.0 | 2025-01-25 | 최소 데이터 기준 명확화, 월간 한도 기준 상세화 |
| 1.2.0 | 2025-01-25 | 감정 랭킹 3개 고정, 개인 미션 구조 변경 (사용자당 3개 미션) |

## 엔드포인트

```
POST /api/v1/retrospects/{retrospectId}/analysis
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
| retrospectId | long | Yes | 분석을 진행할 회고 세션 고유 ID |

### Path Parameter Validation

| Parameter | Rule | Example |
|-----------|------|---------|
| retrospectId | 양의 정수, 1 이상 | `POST /api/v1/retrospects/100/analysis` |
| | 존재하는 회고 세션 ID | ID가 없으면 404 RETRO4041 반환 |

### Body

Request Body 없음

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고 분석이 성공적으로 완료되었습니다.",
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
| insight | string | 팀 전체를 위한 AI 분석 메시지 | 팀 레벨의 통찰력 및 개선점 제시 |
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
| emotionRank | 3 | 3 | 감정 빈도 내림차순 (count 기준) | 정확히 3개의 감정 결과 필수, count가 높을수록 먼저 정렬 |
| personalMissions | 0 | 제한 없음 | userId 오름차순 | 팀원이 0명인 경우 빈 배열 반환 가능 |
| personalMissions[].missions | 3 | 3 | 순서 보장 없음 | 사용자당 정확히 3개의 미션 필수 |

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

### 403 Forbidden - 월간 한도 초과

```json
{
  "isSuccess": false,
  "code": "AI4031",
  "message": "월간 분석 가능 횟수를 초과하였습니다.",
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

### 404 Not Found - 데이터 부족

```json
{
  "isSuccess": false,
  "code": "RETRO4042",
  "message": "분석할 회고 답변 데이터가 부족합니다.",
  "result": null
}
```

### 500 Internal Server Error - AI 분석 실패

```json
{
  "isSuccess": false,
  "code": "AI5001",
  "message": "데이터 종합 분석 중 오류가 발생했습니다.",
  "result": null
}
```

## 에러 코드 요약

| Code | HTTP Status | Description | 발생 조건 |
|------|-------------|-------------|----------|
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | Authorization 헤더 없음, 토큰 만료, 유효하지 않은 토큰 |
| AI4031 | 403 | 서비스 정책상 월간 인사이트 생성 한도 초과 | 현재 월(KST 기준) 해당 팀의 분석 횟수 ≥ 10회 |
| RETRO4041 | 404 | 존재하지 않는 회고 세션 | Path Parameter의 retrospectId가 DB에 없음 |
| RETRO4042 | 404 | 필수 답변 개수 미달로 분석 불가 | 회고 세션의 총 답변 수 < 3개 또는 참여자 수 < 1명 |
| AI5001 | 500 | AI 모델 통신 실패 또는 분석 서버 내부 에러 | OpenAI API 호출 실패, 타임아웃, 서버 내부 오류 |

## 사용 예시

### cURL

```bash
curl -X POST https://api.example.com/api/v1/retrospects/100/analysis \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}"
```
