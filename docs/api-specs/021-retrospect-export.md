# [API-021] GET /api/v1/retrospects/{retrospectId}/export

회고 내보내기 API (PDF 다운로드)

## 개요

특정 회고 세션의 전체 내용(팀 인사이트, 감정 통계, 팀원별 답변 등)을 요약하여 **PDF 파일**로 생성하고 다운로드합니다.

- 성공 시 브라우저를 통해 파일 다운로드가 즉시 시작됩니다.
- 실패 시에는 에러 코드와 메시지가 포함된 JSON 데이터가 반환됩니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |

## 엔드포인트

```
GET /api/v1/retrospects/{retrospectId}/export
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
| retrospectId | long | Yes | 내보낼 회고의 고유 식별자 | 1 이상의 양수 |

## Response

### 성공 시 Response Headers

| Header | Value | Description |
|--------|-------|-------------|
| Content-Type | application/pdf; charset=utf-8 | 응답 본문이 PDF 바이너리임을 명시 |
| Content-Disposition | attachment; filename="{dynamicFileName}.pdf" | 파일 다운로드 활성화 및 동적 파일명 지정 |
| Content-Length | {size} | PDF 파일의 바이트 크기 |
| Cache-Control | no-cache, no-store, must-revalidate | 브라우저 캐시 방지 |

### 성공 (200 OK)

**Binary Data (PDF)**

별도의 JSON 구조 없이 PDF 문서의 이진 데이터(Binary)가 직접 전송됩니다.

### 동적 파일명 규칙

PDF 파일명은 다음 형식을 따릅니다:

```
retrospect_report_{retrospectId}_{timestamp}.pdf
```

**예시:**
- `retrospect_report_100_20250125_143022.pdf`
- `retrospect_report_50_20250124_095015.pdf`

**규칙:**
- `{retrospectId}`: 회고 세션의 고유 식별자
- `{timestamp}`: 생성 시간 (YYYYMMdd_HHmmss 형식, UTC)
- 파일명은 한글을 포함하지 않음 (다운로드 호환성 보장)

## 에러 응답

### 400 Bad Request - 형식 오류

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "잘못된 요청입니다. retrospectId는 양의 정수여야 합니다.",
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

### 404 Not Found - 회고 없음 또는 접근 권한 없음

> 보안 정책: 비멤버에게 회고 존재 여부를 노출하지 않기 위해 "존재하지 않음"과 "접근 권한 없음"을 동일한 404로 처리합니다.

```json
{
  "isSuccess": false,
  "code": "RETRO4041",
  "message": "존재하지 않는 회고이거나 접근 권한이 없습니다.",
  "result": null
}
```

### 500 Internal Server Error - PDF 생성 실패

```json
{
  "isSuccess": false,
  "code": "COMMON500",
  "message": "PDF 생성 중 서버 에러가 발생했습니다.",
  "result": null
}
```

## 에러 코드 요약

| Code | HTTP Status | Description | 발생 조건 |
|------|-------------|-------------|---------|
| COMMON400 | 400 | 잘못된 요청 (형식 오류) | retrospectId가 숫자가 아니거나 음수인 경우 |
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | Authorization 헤더 누락 또는 토큰 만료/불유효 |
| RETRO4041 | 404 | 존재하지 않는 회고이거나 접근 권한 없음 | 해당 ID의 회고 세션이 없거나 팀 멤버가 아닌 경우 |
| COMMON500 | 500 | PDF 생성 중 서버 에러 | PDF 라이브러리 오류 또는 서버 파일 시스템 문제 |

## 사용 예시

### cURL

```bash
curl -X GET https://api.example.com/api/v1/retrospects/100/export \
  -H "Authorization: Bearer {accessToken}" \
  -o retrospect_report.pdf
```

### JavaScript (Fetch API)

```javascript
const response = await fetch('/api/v1/retrospects/100/export', {
  headers: {
    'Authorization': 'Bearer {accessToken}'
  }
});

if (response.ok) {
  const blob = await response.blob();
  const url = window.URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'retrospect_report.pdf';
  a.click();
}
```
