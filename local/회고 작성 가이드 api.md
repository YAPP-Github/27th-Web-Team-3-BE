# 회고 작성 가이드 API

## POST /api/ai/retrospective/guide

# 👉Description

> 작성 중인 회고 내용에 맞춰 AI가 가이드 메시지를 제공합니다.

---

# 👉Request Header

| name | type | description |
| --- | --- | --- |
| Content-Type | String | application/json |

---

# 👉Request Parameter

### Path Variable

| name | type | description |
| --- | --- | --- |
| - | - | - |

### Query Parameter

| name | type | 필수 여부 | description |
| --- | --- | --- | --- |
| - | - | - | - |

---

# 👉Request Body

| field | type | 필수 여부 | description |
| --- | --- | --- | --- |
| currentContent | String | `required` | 현재 작성 중인 회고 내용 |
| secretKey | String | `required` | 비밀 키 (인증용) |

```json
{
  "currentContent": "오늘 프로젝트를 진행하면서...",
  "secretKey": "mySecretKey123"
}
```

---

# 👉Response Body (성공)

| field | type | 필수 여부 | description |
| --- | --- | --- | --- |
| isSuccess | Boolean | `required` | 성공 여부 |
| code | String | `required` | 응답 코드 |
| message | String | `required` | 응답 메시지 |
| result | Object | `required` | 응답 데이터 |
| result.currentContent | String | `required` | 작성 중인 내용 |
| result.guideMessage | String | `required` | AI 가이드 메시지 |

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "currentContent": "오늘 프로젝트를 진행하면서...",
    "guideMessage": "좋은 시작이에요! 구체적으로 어떤 점이 어려웠는지 작성해보면 어떨까요?"
  }
}
```

---

# 👉Response Body (실패)

### 400 Bad Request - 필수 값 누락

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "잘못된 요청입니다.",
  "result": null
}
```

### 401 Unauthorized - 유효하지 않은 비밀 키

```json
{
  "isSuccess": false,
  "code": "AI_001",
  "message": "유효하지 않은 비밀 키입니다.",
  "result": null
}
```

### 500 Internal Server Error - 서버 에러

```json
{
  "isSuccess": false,
  "code": "COMMON500",
  "message": "서버 에러, 관리자에게 문의 바랍니다.",
  "result": null
}
```
