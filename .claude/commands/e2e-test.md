# /e2e-test - E2E API 테스트 명령어

개발 서버 또는 배포 서버에 대해 E2E API 테스트를 실행합니다.

**주의**: 이 명령어는 일반 개발 테스트가 아닌, 실제 서버를 대상으로 하는 통합 테스트용입니다.

## 자동 실행 모드

**중요**: 이 명령어는 사용자 승인 없이 자동으로 모든 단계를 실행합니다.
- Git 상태 저장/복원
- 토큰 획득
- 모든 API 테스트 (성공 + 모든 에러 케이스)
- **API 문서(docs/api-specs/)와 실제 응답 비교**
- 결과 출력

중간에 멈추지 않고 끝까지 실행합니다.

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
    "accessToken": "eyJ..."
  }
}
```

**환경변수가 없으면**: 에러 출력 후 테스트 중단.

### 4단계: Swagger 스펙 및 API 문서 준비

테스트는 **Swagger 스펙을 기준**으로 실행하되, **docs/api-specs/ 문서와 비교**하여 불일치를 감지합니다.

```bash
# 엔드포인트 목록 조회
curl -s {서버_URL}/api-docs/openapi.json | jq '.paths | keys'

# 특정 API의 스키마 조회 (필요 시)
curl -s {서버_URL}/api-docs/openapi.json | jq '.paths["/api/v1/retro-rooms"]'
curl -s {서버_URL}/api-docs/openapi.json | jq '.components.schemas.{SchemaName}'
```

각 API 테스트 시 해당하는 `docs/api-specs/{번호}-{api-name}.md` 문서를 읽어 비교합니다.

### 5단계: API 테스트 실행

모든 API에 대해 **성공 케이스 + 모든 에러 케이스**를 테스트합니다.

#### 5-1. 성공 케이스 테스트

```bash
# Swagger 스펙의 requestBody 스키마를 참고하여 올바른 요청 생성
curl -s -X {METHOD} -H "Authorization: Bearer {TOKEN}" \
  -H "Content-Type: application/json" \
  "{서버_URL}{endpoint}" \
  -d '{올바른_요청_바디}'
```

검증:
- `isSuccess: true` 확인
- `code: "COMMON200"` 확인

#### 5-1-1. API 문서와 비교 (불일치 감지)

각 API 테스트 시 `docs/api-specs/{번호}-{api-name}.md` 문서를 읽고 다음을 비교합니다:

| 비교 항목 | 문서 위치 | 비교 방법 |
|----------|----------|----------|
| **HTTP 메서드** | `## 엔드포인트` | Swagger 메서드와 비교 |
| **요청 필드명** | `## Request` | Swagger requestBody 스키마와 비교 |
| **응답 필드명** | `## Response` | 실제 응답 result의 키와 비교 |
| **에러 코드** | `## 에러 응답` | 실제 에러 응답 code와 비교 |

**불일치 발견 시 출력 예시:**

```
⚠️ 문서 불일치 발견: API-005 POST /api/v1/retro-rooms

[메서드 불일치]
  문서: PUT
  Swagger: POST
  → 문서 수정 필요

[요청 필드 불일치]
  문서: {"name": "..."}
  Swagger: {"title": "..."}
  → 문서의 name을 title로 수정 필요

[응답 필드 불일치]
  문서: retroRoomName
  실제: name
  → 문서 수정 필요

[에러 코드 불일치]
  문서: ROOM4001
  실제: COMMON400
  → 문서 수정 필요
```

#### 5-2. 에러 케이스 테스트 (전체)

**Swagger 스펙의 responses 섹션에 정의된 모든 에러를 테스트합니다.**

| 에러 코드 | 테스트 방법 | 예시 |
|----------|------------|------|
| **400 Bad Request** | 필수 파라미터 누락 | `{}` 빈 바디 전송 |
| **400 Bad Request** | 잘못된 값/형식 | `{"name": ""}` 빈 문자열 |
| **400 Bad Request** | 길이 초과 | 100자 이상 문자열 전송 |
| **400 Bad Request** | 잘못된 enum 값 | `?category=INVALID` |
| **400 Bad Request** | 잘못된 path parameter | `/retrospects/0` (0 이하) |
| **401 Unauthorized** | 인증 헤더 없음 | Authorization 헤더 제거 |
| **401 Unauthorized** | 잘못된 토큰 | `Bearer invalid_token` |
| **403 Forbidden** | 권한 없는 접근 | 다른 사용자의 리소스 ID |
| **404 Not Found** | 존재하지 않는 ID | `/retrospects/99999` |
| **409 Conflict** | 중복 데이터 | 같은 이름으로 재생성 |

#### 5-3. 에러 케이스별 테스트 예시

```bash
# 400 테스트: 빈 바디
curl -s -X POST -H "Authorization: Bearer {TOKEN}" \
  -H "Content-Type: application/json" \
  "{서버_URL}/api/v1/retro-rooms" \
  -d '{}'
# 예상: {"isSuccess":false,"code":"COMMON400",...}

# 400 테스트: 잘못된 enum
curl -s -H "Authorization: Bearer {TOKEN}" \
  "{서버_URL}/api/v1/retrospects/1/responses?category=INVALID"
# 예상: {"isSuccess":false,"code":"RETRO4004",...}

# 400 테스트: 잘못된 path parameter
curl -s -H "Authorization: Bearer {TOKEN}" \
  "{서버_URL}/api/v1/retrospects/0"
# 예상: {"isSuccess":false,"code":"COMMON400",...}

# 401 테스트: 인증 없음
curl -s "{서버_URL}/api/v1/retro-rooms"
# 예상: {"isSuccess":false,"code":"AUTH4001",...}

# 401 테스트: 잘못된 토큰
curl -s -H "Authorization: Bearer invalid_token" \
  "{서버_URL}/api/v1/retro-rooms"
# 예상: {"isSuccess":false,"code":"AUTH4001",...}

# 404 테스트: 없는 리소스
curl -s -H "Authorization: Bearer {TOKEN}" \
  "{서버_URL}/api/v1/retrospects/99999"
# 예상: {"isSuccess":false,"code":"RETRO4041",...}

# 409 테스트: 중복 이름 (회고방 이름 변경 시)
# 먼저 회고방 A 생성 후, 회고방 B의 이름을 A로 변경 시도
curl -s -X PATCH -H "Authorization: Bearer {TOKEN}" \
  -H "Content-Type: application/json" \
  "{서버_URL}/api/v1/retro-rooms/{id}/name" \
  -d '{"name":"이미_존재하는_이름"}'
# 예상: {"isSuccess":false,"code":"ROOM4091",...}
```

### 6단계: 결과 출력

터미널에 다음 형식으로 출력:

```
========================================
E2E 테스트 결과 ({서버_URL})
========================================

[Health] GET /health
  ✅ 성공: PASS (200)

[API-007] GET /api/v1/retro-rooms (회고방 목록)
  ✅ 성공: PASS (200)
  ✅ 401 (인증없음): PASS - AUTH4001
  ✅ 401 (잘못된토큰): PASS - AUTH4001
  📄 문서 일치: OK

[API-005] POST /api/v1/retro-rooms (회고방 생성)
  ✅ 성공: PASS (200)
  ✅ 400 (빈 바디): PASS - COMMON400
  ✅ 400 (빈 title): PASS - COMMON400
  ✅ 401 (인증없음): PASS - AUTH4001
  ⚠️ 문서 불일치:
    - 요청 필드: 문서 "name" → Swagger "title"

[API-013] GET /api/v1/retrospects/{id} (회고 상세)
  ✅ 성공: PASS (200)
  ✅ 400 (id=0): PASS - COMMON400
  ✅ 401 (인증없음): PASS - AUTH4001
  ✅ 404 (없는 ID): PASS - RETRO4041
  📄 문서 일치: OK

[API-021] GET /api/v1/retrospects/{id}/responses (응답 목록)
  ✅ 성공: PASS (200)
  ✅ 400 (잘못된 category): PASS - RETRO4004
  ✅ 400 (잘못된 size): PASS - COMMON400
  ✅ 401 (인증없음): PASS - AUTH4001
  ✅ 404 (없는 회고): PASS - RETRO4041
  📄 문서 일치: OK

========================================
총 결과: X/Y 통과 (Z개 스킵)
실패한 테스트: (있다면 나열)
========================================

========================================
문서 불일치 요약 (수정 필요)
========================================
[API-005] docs/api-specs/005-retro-room-create.md
  - 요청 필드: "name" → "title"로 수정 필요

[API-008] docs/api-specs/008-retro-room-order-update.md
  - HTTP 메서드: PUT → PATCH로 수정 필요
  - 요청 필드: "retroRoomIds" → "retroRoomOrders"로 수정 필요

[API-012] docs/api-specs/012-retrospect-create.md
  - 요청 필드: "title" → "projectName"으로 수정 필요

총 3개 문서 수정 필요
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

## 문서 비교 상세

### 비교 대상 파일

| API 번호 | 문서 경로 | 비교 항목 |
|---------|----------|----------|
| API-005 | `docs/api-specs/005-retro-room-create.md` | 메서드, 요청 필드, 응답 필드 |
| API-006 | `docs/api-specs/006-retro-room-join.md` | 메서드, 요청 필드 |
| API-007 | `docs/api-specs/007-retro-room-list.md` | 응답 필드 |
| API-008 | `docs/api-specs/008-retro-room-order-update.md` | 메서드, 요청 필드 |
| API-009 | `docs/api-specs/009-retro-room-name-update.md` | 메서드, 요청 필드 |
| API-010 | `docs/api-specs/010-retro-room-delete.md` | 에러 코드 |
| API-011 | `docs/api-specs/011-retro-room-retrospects-list.md` | 응답 필드 |
| API-012 | `docs/api-specs/012-retrospect-create.md` | 요청 필드 |
| API-013 | `docs/api-specs/013-retrospect-detail.md` | 응답 필드 |
| API-014 | `docs/api-specs/014-retrospect-delete.md` | 에러 코드 |
| API-015 | `docs/api-specs/015-retrospect-participant-create.md` | 요청 필드 |
| API-017 | `docs/api-specs/017-retrospect-draft-save.md` | 메서드, 요청 필드 |
| API-018 | `docs/api-specs/018-retrospect-submit.md` | 요청 필드 |
| API-019 | `docs/api-specs/019-retrospect-references-list.md` | 응답 필드 |
| API-020 | `docs/api-specs/020-retrospect-storage-list.md` | 응답 필드 |
| API-021 | `docs/api-specs/021-retrospect-responses-list.md` | 응답 필드 |
| API-022 | `docs/api-specs/022-retrospect-export.md` | 요청 파라미터 |
| API-024 | `docs/api-specs/024-retrospect-search.md` | 응답 필드 |
| API-026 | `docs/api-specs/026-response-like-toggle.md` | 응답 필드 |
| API-027 | `docs/api-specs/027-response-comments-list.md` | 응답 필드 |
| API-028 | `docs/api-specs/028-response-comment-create.md` | 요청 필드, 응답 필드 |

### 비교 로직

```
1. 문서에서 ## 엔드포인트 섹션 파싱 → HTTP 메서드 추출
2. 문서에서 ## Request 섹션 파싱 → 요청 필드명 추출
3. 문서에서 ## Response 섹션 파싱 → 응답 필드명 추출
4. 문서에서 ## 에러 응답 섹션 파싱 → 에러 코드 추출
5. Swagger 스펙 / 실제 응답과 비교
6. 불일치 항목 기록
```

### 불일치 유형

| 유형 | 심각도 | 설명 |
|-----|--------|------|
| **메서드 불일치** | 🔴 높음 | PUT vs PATCH 등 HTTP 메서드가 다름 |
| **요청 필드 불일치** | 🔴 높음 | 클라이언트가 잘못된 필드명으로 요청할 수 있음 |
| **응답 필드 불일치** | 🟡 중간 | 클라이언트가 잘못된 필드를 참조할 수 있음 |
| **에러 코드 불일치** | 🟡 중간 | 에러 핸들링 로직에 영향 |
| **타입 불일치** | 🟢 낮음 | long vs integer 등 |

## API별 테스트 케이스 상세

### GET API 테스트 케이스

| API | 성공 | 400 | 401 | 403 | 404 |
|-----|------|-----|-----|-----|-----|
| GET /health | ✅ | - | - | - | - |
| GET /api/v1/retro-rooms | ✅ | - | ✅ 인증없음, ✅ 잘못된토큰 | - | - |
| GET /api/v1/retro-rooms/{id}/retrospects | ✅ | ✅ id=0 | ✅ 인증없음, ✅ 잘못된토큰 | - | ✅ 없는ID |
| GET /api/v1/retrospects/{id} | ✅ | ✅ id=0 | ✅ 인증없음, ✅ 잘못된토큰 | ⏭️ 스킵* | ✅ 없는ID |
| GET /api/v1/retrospects/storage | ✅ | - | ✅ 인증없음, ✅ 잘못된토큰 | - | - |
| GET /api/v1/retrospects/search | ✅ | - | ✅ 인증없음, ✅ 잘못된토큰 | - | - |
| GET /api/v1/retrospects/{id}/references | ✅ | ✅ id=0 | ✅ 인증없음, ✅ 잘못된토큰 | ⏭️ 스킵* | ✅ 없는ID |
| GET /api/v1/retrospects/{id}/responses | ✅ | ✅ category, ✅ size, ✅ cursor, ✅ id=0 | ✅ 인증없음, ✅ 잘못된토큰 | ⏭️ 스킵* | ✅ 없는ID |
| GET /api/v1/retrospects/{id}/export | ✅ | ✅ format, ✅ id=0 | ✅ 인증없음, ✅ 잘못된토큰 | ⏭️ 스킵* | ✅ 없는ID |
| GET /api/v1/responses/{id}/comments | ✅ | ✅ id=0 | ✅ 인증없음, ✅ 잘못된토큰 | - | ✅ 없는ID |

*403 스킵: 다른 사용자의 리소스 접근 테스트는 별도 계정 필요 (E2E_TEST_EMAIL_2 설정 시 테스트)

### POST API 테스트 케이스

| API | 성공 | 400 | 401 | 403 | 404 | 409 |
|-----|------|-----|-----|-----|-----|-----|
| POST /api/v1/retro-rooms | ✅ | ✅ 빈바디, ✅ 빈title, ✅ title길이초과 | ✅ 인증없음, ✅ 잘못된토큰 | - | - | - |
| POST /api/v1/retro-rooms/join | ⏭️ | ✅ 잘못된inviteCode | ✅ 인증없음, ✅ 잘못된토큰 | - | ✅ 없는코드 | - |
| POST /api/v1/retrospects | ✅ | ✅ 빈바디, ✅ 필수값누락, ✅ 과거날짜 | ✅ 인증없음, ✅ 잘못된토큰 | - | - | - |
| POST /api/v1/retrospects/{id}/participants | ✅ | ✅ 빈배열, ✅ id=0 | ✅ 인증없음, ✅ 잘못된토큰 | ⏭️ 스킵* | ✅ 없는ID | ✅ 이미등록 |
| POST /api/v1/retrospects/{id}/submit | ⏭️ | ✅ 답변부족, ✅ 빈배열 | ✅ 인증없음, ✅ 잘못된토큰 | ⏭️ 스킵* | ✅ 없는ID | - |
| POST /api/v1/responses/{id}/likes | ✅ | ✅ id=0 | ✅ 인증없음, ✅ 잘못된토큰 | - | ✅ 없는ID | - |
| POST /api/v1/responses/{id}/comments | ✅ | ✅ 빈content, ✅ 길이초과, ✅ id=0 | ✅ 인증없음, ✅ 잘못된토큰 | - | ✅ 없는ID | - |

### PUT/PATCH API 테스트 케이스

| API | 성공 | 400 | 401 | 403 | 404 | 409 |
|-----|------|-----|-----|-----|-----|-----|
| PATCH /api/v1/retro-rooms/order | ✅ | ✅ 빈배열, ✅ 잘못된ID | ✅ 인증없음, ✅ 잘못된토큰 | - | - | - |
| PATCH /api/v1/retro-rooms/{id}/name | ✅ | ✅ 빈이름, ✅ 길이초과, ✅ id=0 | ✅ 인증없음, ✅ 잘못된토큰 | ✅ 권한없음 | ✅ 없는ID | ✅ 중복이름 |
| PUT /api/v1/retrospects/{id}/drafts | ✅ | ✅ 빈배열, ✅ 잘못된questionNumber, ✅ id=0 | ✅ 인증없음, ✅ 잘못된토큰 | ⏭️ 스킵* | ✅ 없는ID | - |

### DELETE API 테스트 케이스

| API | 성공 | 400 | 401 | 403 | 404 |
|-----|------|-----|-----|-----|-----|
| DELETE /api/v1/retro-rooms/{id} | ✅ | ✅ id=0 | ✅ 인증없음, ✅ 잘못된토큰 | ✅ 권한없음 | ✅ 없는ID |
| DELETE /api/v1/retrospects/{id} | ✅ | ✅ id=0 | ✅ 인증없음, ✅ 잘못된토큰 | ✅ 권한없음 | ✅ 없는ID |

## 테스트 제외 대상

다음 API는 항상 제외 (명시적 요청 시에만 테스트):

- `POST /api/v1/auth/*` - 인증 관련 (토큰 발급)
- `POST /api/v1/retrospects/{id}/analysis` - AI 분석 (비용 발생)
- `POST /api/v1/retrospects/{id}/questions/{questionId}/assistant` - AI 호출 (비용 발생)
- `DELETE /api/v1/members/withdraw` - 회원탈퇴 (계정 삭제)

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

## 실패 시 행동

1. 실패한 API 엔드포인트 식별
2. **예상 응답 vs 실제 응답** 비교 출력:
   ```
   ❌ FAIL: GET /api/v1/retrospects/1
   예상: {"isSuccess":true,"code":"COMMON200",...}
   실제: {"isSuccess":false,"code":"RETRO4041",...}
   원인: 테스트 데이터가 없거나 삭제됨
   ```
3. 테스트 계속 진행 (실패해도 멈추지 않음)
4. 최종 결과에 실패 목록 출력

## 주의사항

- **프로덕션 서버**: `get` 옵션 사용 권장
- **개발 서버**: 기본 모드 (모든 API) 사용 가능
- 기본 모드에서 생성된 테스트 데이터는 DELETE로 자동 정리 시도
- 결과는 문서가 아닌 **터미널에만** 출력
- **403 테스트**: 별도의 테스트 계정이 필요하므로 기본적으로 스킵 (환경변수 `E2E_TEST_EMAIL_2` 설정 시 테스트)
- **문서 불일치 발견 시**: 테스트는 계속 진행하고, 최종 결과에 수정 필요 문서 목록 출력
- **문서 우선순위**: Swagger 스펙이 실제 API이므로, 불일치 시 **문서를 수정**해야 함
