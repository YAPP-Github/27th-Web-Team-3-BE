# API-128: Open Graph 메타데이터 조회

## 개요

외부 URL의 Open Graph 태그를 파싱하여 메타데이터(제목, 설명, 썸네일 등)를 반환합니다.
클라이언트에서 CORS로 인해 직접 외부 URL을 요청할 수 없으므로, 백엔드에서 프록시 역할을 합니다.

## 엔드포인트

```
GET /api/v1/og?url={url}
```

## 인증

Bearer Token 필수

## 요청

### Query Parameters

| 파라미터 | 타입 | 필수 | 설명 |
|----------|------|------|------|
| `url` | string | O | OG 태그를 파싱할 대상 URL (최대 2,048자) |

### 예시

```
GET /api/v1/og?url=https://github.com/example/project
```

## 응답

### 성공 (200 OK)

#### OG 태그 파싱 성공

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "url": "https://github.com/example/project",
    "title": "example/project",
    "description": "An example project repository",
    "image": "https://opengraph.githubassets.com/example.png"
  }
}
```

#### OG 태그 파싱 실패 (graceful fallback)

외부 사이트 접근 불가, OG 태그 미존재 등의 경우 URL만 반환합니다.

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "url": "https://example.com/no-og",
    "title": null,
    "description": null,
    "image": null
  }
}
```

### 에러 응답

| 코드 | HTTP | 설명 |
|------|------|------|
| `COMMON400` | 400 | URL 파라미터 누락 또는 잘못된 URL 형식 |
| `AUTH4001` | 401 | 인증 실패 |

## 응답 필드

| 필드 | 타입 | Nullable | 설명 |
|------|------|----------|------|
| `url` | string | N | 요청한 원본 URL |
| `title` | string | Y | `og:title` 값 |
| `description` | string | Y | `og:description` 값 |
| `image` | string | Y | `og:image` 값 |

## 참고 사항

- 외부 URL 요청 타임아웃: 5초
- User-Agent 헤더를 설정하여 봇 차단 우회
- 파싱 실패 시에도 200 OK로 URL만 반환 (graceful fallback)
