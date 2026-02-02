# /e2e-test - E2E API 테스트 명령어

개발 서버 또는 배포 서버에 대해 E2E API 테스트를 실행합니다.

**주의**: 이 명령어는 일반 개발 테스트가 아닌, 실제 서버를 대상으로 하는 통합 테스트용입니다.

## 사용법

```
/e2e-test <서버_URL>
```

예시:
- `/e2e-test https://dev-api.example.com`
- `/e2e-test https://api.example.com`

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

### 3단계: Swagger 스펙 가져오기
```bash
# {서버_URL}/swagger.json 에서 API 스펙 조회
curl -s {서버_URL}/swagger.json | jq .
```

- Swagger 스펙에서 사용 가능한 엔드포인트 목록 파악
- 각 엔드포인트의 요청/응답 스키마 확인

### 4단계: API 테스트 실행

`docs/api-specs/` 디렉토리의 각 API 문서를 읽고, 다음을 테스트:

#### 성공 케이스 테스트
- API 문서의 "사용 예시" 섹션의 cURL 명령어 기반
- 예상 응답 코드와 실제 응답 비교

#### 에러 케이스 테스트
- 필수 파라미터 누락 (400)
- 인증 실패 (401)
- 권한 없음 (403)
- 리소스 없음 (404)

### 5단계: 결과 출력

터미널에 다음 형식으로 출력:

```
========================================
E2E 테스트 결과 ({서버_URL})
========================================

[API-001] POST /api/v1/auth/social-login
  - 성공 케이스: PASS / FAIL (응답 코드: 200)
  - 에러 케이스 (400): PASS / FAIL
  - 에러 케이스 (401): PASS / FAIL

[API-002] POST /api/v1/auth/signup
  - 성공 케이스: SKIP (인증 필요)
  - 에러 케이스 (400): PASS / FAIL

... (이하 생략)

========================================
총 결과: X/Y 통과 (Z개 스킵)
========================================
```

### 6단계: Git 상태 복원
```bash
# 원래 브랜치로 복귀
git checkout $ORIGINAL_BRANCH

# stash 했다면 복원
if [ "$STASHED" = true ]; then
    git stash pop
fi
```

## 테스트 대상 API 목록

`docs/api-specs/` 디렉토리의 모든 API 문서를 대상으로 테스트합니다:

| 파일 | API | 인증 필요 |
|------|-----|----------|
| 001-auth-social-login.md | POST /api/v1/auth/social-login | No |
| 002-auth-signup.md | POST /api/v1/auth/signup | Yes (signupToken) |
| 003-auth-token-refresh.md | POST /api/v1/auth/token/refresh | Yes (refreshToken) |
| 004-auth-logout.md | POST /api/v1/auth/logout | Yes |
| ... | ... | ... |

## 테스트 전략

### 인증이 필요한 API
- 테스트용 계정으로 먼저 로그인하여 토큰 획득
- 획득한 토큰으로 인증 필요 API 테스트
- 테스트용 계정 정보는 환경변수 또는 사용자 입력으로 제공

### 인증이 필요 없는 API
- API 문서의 예시 요청을 그대로 실행
- 응답 코드와 응답 구조 검증

## 주의사항

- 프로덕션 서버 테스트 시 데이터 변경에 주의
- 테스트 후 생성된 데이터는 정리 필요
- 네트워크 오류 시 재시도 로직 포함
- 테스트 결과는 문서가 아닌 터미널에만 출력

## 실패 시 행동

1. 실패한 API 엔드포인트 식별
2. 예상 응답 vs 실제 응답 비교 출력
3. 가능한 원인 분석 (문서 불일치, 서버 버그 등)
4. 사용자에게 후속 조치 제안
