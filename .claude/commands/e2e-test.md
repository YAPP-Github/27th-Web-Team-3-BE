# /e2e-test - E2E API 테스트 명령어

개발 서버 또는 배포 서버에 대해 E2E API 테스트를 실행합니다.

**주의**: 이 명령어는 일반 개발 테스트가 아닌, 실제 서버를 대상으로 하는 통합 테스트용입니다.

## 사용법

```
/e2e-test [서버_URL] [Bearer_Token] [옵션]
```

### 기본값

| 파라미터 | 기본값 |
|----------|--------|
| 서버_URL | `https://api.moaofficial.kr` |
| Bearer_Token | 자동 획득 (`$E2E_TEST_EMAIL` 환경변수로 로그인) |

### 옵션

| 옵션 | 설명 |
|------|------|
| (없음) | 모든 API 테스트 (GET, POST, PUT, DELETE 포함) |
| `get` | GET API만 테스트 (데이터 손상 없음) |

### 예시

```bash
# 기본값으로 모든 API 테스트 (서버: api.moaofficial.kr, 토큰: 자동)
/e2e-test

# GET만 테스트
/e2e-test get

# 특정 서버 + 토큰 지정
/e2e-test https://api.example.com {TOKEN}

# 특정 서버 + GET만
/e2e-test https://api.example.com {TOKEN} get
```

## 실행 단계

### 1단계: Git 상태 저장
```bash
# 현재 브랜치 저장
ORIGINAL_BRANCH=$(git branch --show-current)

# 작업 중인 변경사항이 있다면 stash
if [ -n "$(git status --porcelain)" ]; then
    git stash push -m "e2e-test-auto-stash"
    STASHED=true
fi
```

### 2단계: dev 브랜치 최신화
```bash
git checkout dev
git pull origin dev
```

### 3단계: 토큰 획득 (토큰 미제공 시)

Bearer Token이 제공되지 않은 경우, 환경변수의 테스트 계정으로 자동 로그인합니다.

#### 환경변수 설정

`.env` 파일 또는 환경변수에 테스트 계정 설정:

```bash
E2E_TEST_EMAIL=your-test-account@example.com
```

#### 로그인 요청

```bash
# 환경변수에서 테스트 계정 읽기
curl -s -X POST {서버_URL}/api/auth/login/email \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$E2E_TEST_EMAIL\"}"
```

응답에서 accessToken 추출:
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "result": {
    "isNewMember": false
  }
}
```

**참고**: accessToken은 쿠키로 전달되므로, 이후 요청에서 쿠키를 유지하거나 응답 헤더에서 추출합니다.

```bash
# 쿠키 저장하여 사용
curl -s -X POST {서버_URL}/api/auth/login/email \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$E2E_TEST_EMAIL\"}" \
  -c cookies.txt

# 이후 요청에서 쿠키 사용
curl -s -b cookies.txt {서버_URL}/api/v1/retro-rooms
```

**환경변수가 없으면**: 사용자에게 테스트 계정 입력을 요청합니다.

### 4단계: Swagger 엔드포인트 목록 조회

**컨텍스트 절약을 위해 엔드포인트 목록만 조회합니다.**

기본 서버 URL: `https://api.moaofficial.kr`

```bash
# 엔드포인트 목록만 가져오기 (전체 Swagger JSON 가져오지 않음)
curl -s {서버_URL}/api-docs/openapi.json | jq '.paths | keys'
```

특정 API의 상세 스펙이 필요할 때만 선택적으로 조회:
```bash
# 특정 API의 파라미터/응답 스펙 조회
curl -s {서버_URL}/api-docs/openapi.json | jq '.paths["/api/v1/retro-rooms"]'
```

### 5단계: API 테스트 실행

`docs/api-specs/` 디렉토리의 각 API 문서를 **하나씩** 읽고 테스트합니다.

#### 5-1. 문서에서 테스트 케이스 추출

각 API 문서에서 다음 섹션을 확인:

1. **엔드포인트 정보**: 메서드, 경로
2. **Request 섹션**: 필수 파라미터, 헤더
3. **Response 섹션**: 성공 응답 구조
4. **에러 응답 섹션**: 에러 코드별 조건

예시 (`docs/api-specs/007-retro-room-list.md`):
```markdown
## 에러 응답
### 401 Unauthorized - 인증 실패
### 500 Internal Server Error - 서버 에러
```

#### 5-2. 성공 케이스 테스트

```bash
# API 문서의 "사용 예시" 기반으로 테스트
curl -s -H "Authorization: Bearer {TOKEN}" {서버_URL}/api/v1/retro-rooms
```

검증:
- `isSuccess: true` 확인
- `code: "COMMON200"` 확인
- `result` 구조가 문서와 일치하는지 확인

#### 5-3. 에러 케이스 테스트 (문서 기반)

**문서의 "에러 응답" 섹션에 정의된 모든 에러를 테스트합니다.**

| 에러 코드 | 테스트 방법 |
|----------|------------|
| 400 Bad Request | 필수 파라미터 누락 또는 잘못된 값 전송 |
| 401 Unauthorized | Authorization 헤더 제거 또는 잘못된 토큰 |
| 403 Forbidden | 권한 없는 리소스 접근 (다른 사용자의 데이터) |
| 404 Not Found | 존재하지 않는 ID 사용 (예: 99999) |
| 409 Conflict | 중복 데이터 생성 시도 |

```bash
# 401 테스트: 인증 없이 요청
curl -s {서버_URL}/api/v1/retro-rooms
# 예상: {"isSuccess":false,"code":"AUTH4001",...}

# 404 테스트: 없는 리소스
curl -s -H "Authorization: Bearer {TOKEN}" {서버_URL}/api/v1/retrospects/99999
# 예상: {"isSuccess":false,"code":"RETRO4041",...}

# 400 테스트: 잘못된 파라미터
curl -s -H "Authorization: Bearer {TOKEN}" "{서버_URL}/api/v1/retrospects/1/responses?category=INVALID"
# 예상: {"isSuccess":false,"code":"RETRO4004",...}
```

### 6단계: 결과 출력

터미널에 다음 형식으로 출력:

```
========================================
E2E 테스트 결과 ({서버_URL})
========================================

[Health] GET /health
  ✅ 성공 케이스: PASS (200)

[API-007] GET /api/v1/retro-rooms (회고방 목록)
  ✅ 성공 케이스: PASS (200)
  ✅ 에러 401 (인증없음): PASS - AUTH4001

[API-013] GET /api/v1/retrospects/{id} (회고 상세)
  ✅ 성공 케이스: PASS (200)
  ✅ 에러 401 (인증없음): PASS - AUTH4001
  ✅ 에러 404 (없는 ID): PASS - RETRO4041

[API-021] GET /api/v1/retrospects/{id}/responses (응답 목록)
  ✅ 성공 케이스: PASS (200)
  ✅ 에러 400 (잘못된 category): PASS - RETRO4004
  ✅ 에러 401 (인증없음): PASS - AUTH4001

========================================
총 결과: X/Y 통과 (Z개 스킵)
실패한 테스트: (있다면 나열)
========================================
```

### 7단계: Git 상태 복원
```bash
# 원래 브랜치로 복귀
git checkout $ORIGINAL_BRANCH

# stash 했다면 복원
if [ "$STASHED" = true ]; then
    git stash pop
fi
```

## 테스트 모드

### 기본 모드 (모든 API)

모든 HTTP 메서드를 테스트합니다:

1. **GET** - 조회 API
2. **POST** - 생성 API (데이터 생성됨)
3. **PUT/PATCH** - 수정 API (데이터 변경됨)
4. **DELETE** - 삭제 API (데이터 삭제됨)

테스트 순서:
1. GET API 먼저 실행
2. POST로 테스트 데이터 생성
3. PUT/PATCH로 수정 테스트
4. DELETE로 정리

### GET 모드 (`get` 옵션)

데이터 손상 없이 안전하게 테스트:

- **GET API만** 테스트
- POST, PUT, PATCH, DELETE 제외
- 프로덕션 서버에서 권장

## 테스트 제외 대상

다음 API는 항상 제외 (명시적 요청 시에만 테스트):

- `POST /api/v1/auth/*` - 인증 관련 (토큰 발급)
- `POST /api/v1/retrospects/{id}/questions/{questionId}/assistant` - AI 호출 (비용 발생)

## 실패 시 행동

1. 실패한 API 엔드포인트 식별
2. **예상 응답 vs 실제 응답** 비교 출력:
   ```
   ❌ FAIL: GET /api/v1/retrospects/1
   예상: {"isSuccess":true,"code":"COMMON200",...}
   실제: {"isSuccess":false,"code":"RETRO4041",...}
   원인: 테스트 데이터가 없거나 삭제됨
   ```
3. 가능한 원인 분석 (문서 불일치, 서버 버그, 테스트 데이터 문제)
4. 후속 조치 제안

## 주의사항

- **프로덕션 서버**: `get` 옵션 사용 권장
- **개발 서버**: 기본 모드 (모든 API) 사용 가능
- 기본 모드에서 생성된 테스트 데이터는 DELETE로 자동 정리 시도
- 결과는 문서가 아닌 **터미널에만** 출력
