# [API-029] POST /api/v1/retrospects/{retrospectId}/questions/{questionId}/assistant

> 회고 질문별 AI 어시스턴트 가이드 생성

## 개요

회고 작성 시 각 질문에 대해 AI 어시스턴트가 작성 가이드를 제공합니다.

**두 가지 모드:**
1. **초기 가이드 모드**: 사용자가 아직 답변을 입력하지 않은 상태에서 질문에 답하는 방법을 안내
2. **맞춤 가이드 모드**: 사용자가 이미 입력한 내용을 기반으로 더 풍부한 답변을 위한 맞춤형 가이드 제공

**주요 특징:**
- 질문별로 최대 3개의 가이드 제공
- 입력 내용 유무에 따라 다른 유형의 가이드 생성
- 사용자당 월 10회 사용 제한

---

## 버전 정보

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-30 | 최초 작성 |

---

## 엔드포인트

```
POST /api/v1/retrospects/{retrospectId}/questions/{questionId}/assistant
```

---

## 인증

Bearer 토큰 인증 필요

---

## Request

### Headers

| Header | Required | Description |
|--------|----------|-------------|
| Content-Type | Yes | `application/json` |
| Authorization | Yes | `Bearer {accessToken}` |

### Path Parameters

| Parameter | Type | Required | Description | Validation |
|-----------|------|----------|-------------|------------|
| retrospectId | integer | Yes | 회고 ID | > 0 |
| questionId | integer | Yes | 질문 ID | > 0, 1~5 범위 |

### Body

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| content | string | No | 현재 입력된 답변 내용 | 최대 1000자 |

**Body 예시 (입력 없는 경우):**

```json
{}
```

또는

```json
{
  "content": ""
}
```

**Body 예시 (입력 있는 경우):**

```json
{
  "content": "오래전부터 시작해서 끝까지 잘 참여했습니다."
}
```

---

## Response

### 성공 응답 (200 OK)

**초기 가이드 모드 (입력 없음):**

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고 어시스턴트 가이드가 생성되었습니다.",
  "result": {
    "questionId": 1,
    "questionContent": "이번 작업/프로젝트에서 잘했던 점은 무엇인가요?",
    "guideType": "INITIAL",
    "guides": [
      {
        "title": "구체적인 상황 떠올리기",
        "description": "특정 미팅이나 작업 순간 중 잘 진행됐다고 느꼈던 장면을 떠올려 보면 좋아요"
      },
      {
        "title": "나의 역할 중심으로 생각하기",
        "description": "팀 전체보다 내가 직접 기여한 부분에 초점을 맞춰 적어보면 좋아요"
      },
      {
        "title": "결과보다 과정 돌아보기",
        "description": "최종 결과물보다 그 과정에서 했던 시도나 노력을 적어보면 좋아요"
      }
    ],
    "remainingCount": 9
  }
}
```

**맞춤 가이드 모드 (입력 있음):**

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고 어시스턴트 가이드가 생성되었습니다.",
  "result": {
    "questionId": 1,
    "questionContent": "이번 작업/프로젝트에서 잘했던 점은 무엇인가요?",
    "guideType": "PERSONALIZED",
    "guides": [
      {
        "title": "시작 배경 구체화하기",
        "description": "마감 기한을 맞추기 위해 언제부터 어떤 방식으로 준비했고 중간에 무엇을 조정했는지 덧붙이면 좋아요"
      },
      {
        "title": "과정에서의 변화 담기",
        "description": "오래전부터 시작했지만 끝까지 참여하는 과정에서 힘들었던 순간과 그때 선택한 대응을 함께 적으면 좋아요"
      },
      {
        "title": "결과의 의미 연결하기",
        "description": "끝까지 잘 참여했다는 결과가 스스로에게 어떤 의미였는지 한 문장으로 정리해 보면 좋아요"
      }
    ],
    "remainingCount": 9
  }
}
```

### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| questionId | integer | 질문 ID |
| questionContent | string | 질문 내용 |
| guideType | string | 가이드 유형 (`INITIAL` 또는 `PERSONALIZED`) |
| guides | array[object] | 가이드 목록 (최대 3개) |
| guides[].title | string | 가이드 제목 (행동 지침) |
| guides[].description | string | 가이드 상세 설명 |
| remainingCount | integer | 이번 달 남은 사용 횟수 |

### 가이드 배열 규칙

| 규칙 | 설명 |
|------|------|
| 최소 개수 | 1개 |
| 최대 개수 | 3개 |
| 정렬 | 중요도순 (첫 번째가 가장 중요) |

### 가이드 유형 (GuideType)

| 값 | 설명 |
|------|------|
| INITIAL | 초기 가이드 - 답변 작성을 시작할 때 참고할 일반적인 가이드 |
| PERSONALIZED | 맞춤 가이드 - 입력된 내용을 분석하여 제공하는 개선 가이드 |

---

## 에러 응답

### 400 Bad Request - 유효하지 않은 요청

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "content는 1000자를 초과할 수 없습니다.",
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

### 403 Forbidden - 월간 사용 횟수 초과

```json
{
  "isSuccess": false,
  "code": "AI4032",
  "message": "이번 달 회고 어시스턴트 사용 횟수를 모두 사용했습니다.",
  "result": null
}
```

### 403 Forbidden - 회고 접근 권한 없음

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

### 404 Not Found - 질문 없음

```json
{
  "isSuccess": false,
  "code": "RETRO4043",
  "message": "존재하지 않는 질문입니다.",
  "result": null
}
```

### 500 Internal Server Error - AI 서비스 오류

```json
{
  "isSuccess": false,
  "code": "AI5001",
  "message": "AI 서비스 처리 중 오류가 발생했습니다.",
  "result": null
}
```

---

## 에러 코드 요약

| 에러 코드 | HTTP 상태 | 설명 |
|----------|----------|------|
| AUTH4001 | 401 | 인증 정보 유효하지 않음 |
| AI4032 | 403 | 월간 어시스턴트 사용 횟수 초과 |
| RETRO4031 | 403 | 회고 접근 권한 없음 |
| RETRO4041 | 404 | 존재하지 않는 회고 |
| RETRO4043 | 404 | 존재하지 않는 질문 |
| AI5001 | 500 | AI 서비스 처리 오류 |
| COMMON400 | 400 | 유효성 검사 실패 |

---

## 사용량 제한

### 월간 사용 제한

| 항목 | 값 |
|------|------|
| 사용자당 월간 제한 | 10회 |
| 리셋 시점 | 매월 1일 00:00 KST |
| 카운트 단위 | API 호출 횟수 (성공 응답만 카운트) |

**참고:** 회고 AI 분석(API-023)과 별도로 카운트됩니다.

---

## 사용 예시

### cURL - 초기 가이드 요청 (입력 없음)

```bash
curl -X POST 'https://api.example.com/api/v1/retrospects/1/questions/1/assistant' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...' \
  -d '{}'
```

### cURL - 맞춤 가이드 요청 (입력 있음)

```bash
curl -X POST 'https://api.example.com/api/v1/retrospects/1/questions/1/assistant' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...' \
  -d '{
    "content": "오래전부터 시작해서 끝까지 잘 참여했습니다."
  }'
```

---

## 비즈니스 로직

### 처리 흐름

1. **인증 확인**: Bearer 토큰 검증
2. **회고 존재 확인**: retrospectId로 회고 조회
3. **접근 권한 확인**: 해당 회고의 참여자인지 확인
4. **질문 유효성 확인**: questionId가 해당 회고의 질문인지 확인
5. **월간 사용량 확인**: 사용자의 이번 달 사용 횟수 확인
6. **가이드 유형 결정**: content 유무에 따라 INITIAL/PERSONALIZED 결정
7. **AI 가이드 생성**:
   - INITIAL: 질문 유형에 맞는 일반적인 가이드 생성
   - PERSONALIZED: 질문 + 입력 내용 분석하여 맞춤 가이드 생성
8. **사용량 증가**: 성공 시 사용 횟수 +1
9. **응답 반환**: 가이드 목록과 남은 횟수 반환

### 가이드 생성 규칙

**초기 가이드 (INITIAL):**
- 질문의 의도와 목적에 맞는 일반적인 작성 가이드
- 구체적인 사례나 관점을 제시
- "~하면 좋아요" 형태의 친근한 어투 사용

**맞춤 가이드 (PERSONALIZED):**
- 입력된 내용의 키워드와 맥락 분석
- 부족한 부분이나 확장 가능한 포인트 식별
- 입력 내용을 더 풍부하게 만들 수 있는 구체적 제안

---

## 관련 API

| API | 관계 |
|-----|------|
| [API-016] 회고 참여자 및 질문 조회 | 질문 목록 확인 |
| [API-017] 회고 답변 임시 저장 | 작성한 답변 저장 |
| [API-023] 회고 AI 분석 | 전체 회고 종합 분석 (별도 기능) |
