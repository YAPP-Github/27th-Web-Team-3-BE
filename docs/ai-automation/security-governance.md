# AI 자동화 파이프라인 보안 및 거버넌스

이 문서는 AI 자동화 파이프라인의 보안 정책, 권한 관리, 감사 절차 및 장애 대응 계획을 정의합니다.

## 1. 보안 고려사항

### 1.1 AI 접근 가능 범위 제한

| 범위 | 허용 | 제한 |
|------|------|------|
| 소스 코드 | 읽기/수정 | 프로덕션 브랜치 직접 푸시 금지 |
| 설정 파일 | `.claude/`, `docs/` | `.env`, 시크릿 파일 수정 금지 |
| 인프라 | 문서 조회만 | 인프라 변경 명령 실행 금지 |
| 데이터베이스 | 스키마 조회 | 프로덕션 데이터 접근 금지 |

#### 디렉토리 접근 정책

```yaml
# 허용된 경로
allowed_paths:
  - codes/server/src/**
  - codes/server/tests/**
  - docs/**
  - .claude/**
  - .github/workflows/**

# 제한된 경로
restricted_paths:
  - .env*                    # 환경 변수
  - **/credentials*          # 인증 정보
  - **/secrets*              # 시크릿
  - **/*.pem                 # 인증서
  - **/*.key                 # 키 파일
```

### 1.2 시크릿 및 민감정보 보호

#### 민감정보 분류

| 등급 | 예시 | 보호 방법 |
|------|------|----------|
| **Critical** | API 키, DB 비밀번호 | GitHub Secrets, 환경변수로만 주입 |
| **High** | 서비스 계정 토큰 | Vault 또는 Secrets Manager 사용 |
| **Medium** | 내부 서비스 URL | 환경별 설정 파일 분리 |
| **Low** | 포트 번호, 타임아웃 | 코드에 하드코딩 가능 |

#### 보호 메커니즘

```rust
// 로깅 시 민감정보 제외 예시
#[instrument(skip(api_key, secret_key))]
pub async fn call_external_api(
    api_key: &str,
    secret_key: &str,
    payload: &Request,
) -> Result<Response, AppError> {
    // ...
}
```

#### Git 커밋 전 체크

```bash
# pre-commit hook에서 민감정보 검사
patterns_to_block:
  - "password\\s*=\\s*['\"][^'\"]+['\"]"
  - "api_key\\s*=\\s*['\"][^'\"]+['\"]"
  - "secret\\s*=\\s*['\"][^'\"]+['\"]"
  - "-----BEGIN.*PRIVATE KEY-----"
```

### 1.3 악의적 입력 방지

#### 입력 검증 레이어

```
[사용자 입력]
    │
    ▼
[1. 형식 검증] ─── 잘못된 형식 → 거부
    │
    ▼
[2. 길이 제한] ─── 초과 → 거부
    │
    ▼
[3. 패턴 검사] ─── 악의적 패턴 → 거부
    │
    ▼
[4. 샌드박스 실행] ─── 비정상 동작 → 격리
    │
    ▼
[처리 완료]
```

#### 프롬프트 인젝션 방지

```rust
// 사용자 입력을 AI 프롬프트에 포함할 때
fn sanitize_user_input(input: &str) -> String {
    // 1. 최대 길이 제한
    let truncated = input.chars().take(MAX_INPUT_LENGTH).collect::<String>();

    // 2. 제어 문자 제거
    let cleaned = truncated.replace(|c: char| c.is_control(), "");

    // 3. 잠재적 명령어 이스케이프
    cleaned.replace("```", "'''")
}
```

#### 명령어 실행 제한

| 명령어 카테고리 | 허용 | 제한 |
|----------------|------|------|
| 빌드 | `cargo build`, `cargo test` | - |
| Git | `git status`, `git diff` | `git push --force`, `git reset --hard` |
| 파일 시스템 | 프로젝트 내 읽기/쓰기 | 프로젝트 외부 접근 |
| 네트워크 | localhost 접근 | 외부 네트워크 요청 (예외 규정 참조) |

### 1.4 파이프라인 운영 예외 규정

일반 보안 정책과 AI 자동화 파이프라인 운영 간의 충돌을 해결하기 위한 예외 규정입니다.

#### 허용된 외부 API 호출

파이프라인 운영을 위해 다음 외부 API 호출이 허용됩니다:

| API 서비스 | 용도 | 호출 주체 | 승인 조건 |
|-----------|------|----------|----------|
| **OpenAI API** | AI 진단 및 코드 분석 | Phase 2-3 스크립트 | `OPENAI_API_KEY` 환경변수 설정 |
| **Anthropic API** | Claude 기반 이슈 분석 | `issue-analyzer.py` | `ANTHROPIC_API_KEY` 환경변수 설정 |
| **GitHub API** | PR 생성, 이슈 관리 | `gh` CLI | `GITHUB_TOKEN` 또는 PAT 설정 |
| **Discord Webhook** | 알림 발송 | `discord-alert.sh` | `DISCORD_WEBHOOK_URL` 설정 |
| **Slack Webhook** | 알림 발송 (선택) | 알림 스크립트 | `SLACK_WEBHOOK_URL` 설정 |

```yaml
# 허용된 외부 도메인 목록
allowed_external_domains:
  - api.openai.com
  - api.anthropic.com
  - api.github.com
  - discord.com/api/webhooks
  - hooks.slack.com

# 금지된 외부 접근 (변경 불가)
forbidden_external_access:
  - 프로덕션 데이터베이스
  - 내부 인프라 API
  - 사용자 개인정보 API
  - 결제/금융 서비스 API
```

#### 브랜치 푸시 권한 범위

자동화 파이프라인에서 허용되는 브랜치 푸시 범위입니다:

| 브랜치 패턴 | 허용 여부 | 용도 | 승인 필요 |
|------------|----------|------|----------|
| `fix/*` | **허용** | 버그 수정 자동화 | 자동 |
| `hotfix/*` | **허용** | 긴급 수정 자동화 | 자동 |
| `ai-fix/*` | **허용** | AI 자동 수정 | 자동 |
| `config/*` | **허용** | 설정 변경 | 자동 |
| `refactor/*` | **허용** | 리팩토링 | 자동 |
| `dev` | **제한** | 메인 개발 브랜치 | PR 머지만 허용 |
| `main`, `master` | **금지** | 프로덕션 브랜치 | 직접 푸시 금지 |
| `release/*` | **금지** | 릴리스 브랜치 | 수동 승인 필요 |

```yaml
# GitHub Actions 브랜치 보호 설정
branch_protection:
  # 자동 푸시 허용 브랜치
  auto_push_allowed:
    patterns:
      - "fix/**"
      - "hotfix/**"
      - "ai-fix/**"
      - "config/**"
      - "refactor/**"
    conditions:
      - ci_bot_only: true
      - max_files_changed: 10
      - no_sensitive_files: true

  # PR을 통한 머지만 허용
  pr_merge_only:
    - dev

  # 완전 보호 (수동 승인 필수)
  fully_protected:
    - main
    - master
    - "release/**"
```

#### 자동화 작업 승인 절차

| 작업 유형 | 자동 승인 조건 | 수동 승인 필요 |
|----------|--------------|---------------|
| **브랜치 생성** | `fix/*`, `hotfix/*`, `ai-fix/*` 패턴 | 다른 패턴 |
| **브랜치 푸시** | 허용된 패턴 + 10파일 이하 변경 | 11파일 이상 변경 |
| **PR 생성** | Draft PR + dev 브랜치 대상 | 다른 브랜치 대상 |
| **PR 머지** | 항상 수동 승인 필요 | - |
| **외부 API 호출** | 허용된 도메인 목록 내 | 목록 외 도메인 |

#### 예외 규정 적용 조건

예외 규정이 적용되기 위한 필수 조건:

1. **식별 가능한 자동화 주체**
   ```yaml
   automation_identity:
     git_author: "Claude Code <noreply@anthropic.com>"
     git_committer: "ci-bot <ci-bot@example.com>"
     commit_suffix: "Co-Authored-By: Claude"
   ```

2. **감사 로그 기록**
   - 모든 외부 API 호출 로깅
   - 모든 Git 작업 로깅
   - 요청 ID 및 세션 ID 포함

3. **Rate Limiting 준수**
   ```yaml
   rate_limits:
     api_calls_per_minute: 10
     branch_creations_per_hour: 20
     pr_creations_per_hour: 10
   ```

4. **자동 롤백 가능**
   - 모든 자동화 작업은 롤백 가능해야 함
   - 롤백 스크립트 사전 검증 필수

---

## 2. 권한 관리

### 2.1 GitHub 토큰 권한 범위

#### 최소 권한 원칙 (Principle of Least Privilege)

```yaml
# GitHub Actions 워크플로우 권한 - 기본 설정
permissions:
  contents: read          # 코드 읽기만
  pull-requests: write    # PR 생성/수정
  issues: write           # 이슈 코멘트
  checks: write           # 체크 결과 업데이트

# 금지된 권한 (일반 워크플로우)
# permissions:
#   contents: write       # 직접 푸시 금지
#   actions: write        # 워크플로우 수정 금지
#   secrets: write        # 시크릿 수정 금지
```

#### AI 자동화 파이프라인 전용 권한 (예외)

AI 자동화 파이프라인(`ai-fix.yml`, `analyze-and-branch.yml`)에 한하여 다음 권한이 허용됩니다:

```yaml
# AI 자동화 전용 워크플로우 권한
# 파일: .github/workflows/ai-automation.yml
name: AI Automation Pipeline
on:
  workflow_dispatch:
  schedule:
    - cron: '*/5 * * * *'

permissions:
  contents: write         # 브랜치 생성/푸시 허용 (조건부)
  pull-requests: write    # PR 생성/수정
  issues: write           # 이슈 코멘트
  checks: write           # 체크 결과 업데이트

# 조건부 제한 (반드시 준수)
# jobs.<job_id>.if: 조건을 통해 브랜치 보호
jobs:
  auto-fix:
    if: |
      startsWith(github.ref_name, 'fix/') ||
      startsWith(github.ref_name, 'hotfix/') ||
      startsWith(github.ref_name, 'ai-fix/')
    runs-on: ubuntu-latest
    steps:
      # ... 자동화 작업 ...
```

#### 권한 부여 조건

| 권한 | 조건 | 감사 요구사항 |
|------|------|-------------|
| `contents: write` | 허용된 브랜치 패턴만 | 모든 푸시 로깅 |
| `pull-requests: write` | Draft PR로만 생성 | PR 생성 로깅 |
| 외부 API 호출 | 허용된 도메인만 | API 호출 로깅 |

#### 권한 남용 방지

```yaml
# 자동 차단 조건
auto_block_conditions:
  - main_branch_push_attempt: true
  - release_branch_push_attempt: true
  - workflow_file_modification: true
  - secrets_access_attempt: true
  - rate_limit_exceeded: true

# 차단 시 조치
on_block:
  - notify: "#security-alerts"
  - disable_workflow: true
  - require_manual_review: true
```

#### Personal Access Token (PAT) 가이드

| 용도 | 필요 권한 | 만료 기간 |
|------|----------|----------|
| PR 자동화 | `repo:status`, `pull_request` | 90일 |
| 코드 읽기 | `repo:read` | 90일 |
| 릴리스 자동화 | `repo`, `workflow` | 30일 |

### 2.2 서비스 계정 관리

#### 계정 분류

| 계정 | 용도 | 권한 수준 |
|------|------|----------|
| `ci-bot` | CI/CD 파이프라인 | 읽기 + PR 생성 |
| `ai-automation-bot` | AI 자동화 파이프라인 | 읽기 + 제한적 쓰기 (아래 참조) |
| `release-bot` | 릴리스 자동화 | 태그 생성, 릴리스 발행 |
| `security-scanner` | 보안 스캔 | 읽기 전용 |

#### AI 자동화 전용 계정 (`ai-automation-bot`)

AI 자동화 파이프라인 전용 서비스 계정의 세부 권한입니다:

| 권한 항목 | 허용 범위 | 제한 사항 |
|----------|----------|----------|
| **브랜치 생성** | `fix/*`, `hotfix/*`, `ai-fix/*`, `config/*`, `refactor/*` | 다른 패턴 금지 |
| **브랜치 푸시** | 생성한 브랜치에만 | `dev`, `main` 금지 |
| **PR 생성** | Draft PR만, `dev` 브랜치 대상 | 다른 브랜치 대상 금지 |
| **외부 API 호출** | OpenAI, Anthropic, GitHub, Discord, Slack | 기타 도메인 금지 |
| **파일 수정** | `codes/server/src/**`, `codes/server/tests/**` | 설정 파일, 워크플로우 금지 |

```yaml
# ai-automation-bot PAT 스코프
ai_automation_bot:
  token_scopes:
    - repo:status
    - public_repo
    - read:org
    - write:discussion

  # Fine-grained permissions (권장)
  fine_grained:
    contents: write
    pull_requests: write
    issues: write
    metadata: read

  # 브랜치 규칙으로 추가 제한
  branch_restrictions:
    allow_patterns:
      - "fix/**"
      - "hotfix/**"
      - "ai-fix/**"
    deny_patterns:
      - "main"
      - "master"
      - "dev"
      - "release/**"
```

#### 계정 관리 정책

```markdown
1. **생성**: 팀 리드 승인 필요
2. **갱신**: 분기별 토큰 로테이션
3. **폐기**: 즉시 비활성화 후 30일 내 삭제
4. **감사**: 월별 권한 검토
```

#### 토큰 로테이션 절차

```bash
# 1. 새 토큰 생성
gh auth token --scopes repo,workflow

# 2. GitHub Secrets 업데이트
gh secret set CI_TOKEN < new_token.txt

# 3. 구 토큰 폐기
gh auth revoke --token OLD_TOKEN

# 4. 로그 기록
echo "$(date): Token rotated by $USER" >> token_rotation.log
```

---

## 3. 감사 및 로깅

### 3.1 모든 자동화 작업 기록

#### 로그 구조

```json
{
  "timestamp": "2025-01-15T10:30:00Z",
  "event_type": "ai_automation",
  "action": "code_modification",
  "actor": "claude-code",
  "target": {
    "file": "codes/server/src/domain/ai/handler.rs",
    "change_type": "edit"
  },
  "context": {
    "trigger": "user_request",
    "session_id": "abc123",
    "parent_task": "implement_api"
  },
  "result": {
    "status": "success",
    "lines_changed": 45
  }
}
```

#### 필수 로깅 이벤트

| 이벤트 | 중요도 | 보관 기간 |
|--------|--------|----------|
| 코드 변경 | High | 1년 |
| 명령어 실행 | High | 6개월 |
| 파일 읽기 | Medium | 30일 |
| 에러 발생 | Critical | 2년 |
| 권한 변경 | Critical | 영구 |

### 3.2 추적 가능성 확보

#### 변경 추적 체계

```
[커밋]
    │
    ├── commit_hash: abc123
    ├── author: Claude Code <claude@anthropic.com>
    ├── co_author: developer@example.com
    ├── timestamp: 2025-01-15T10:30:00Z
    │
    ├── linked_issue: #42
    ├── linked_pr: #100
    └── session_id: session_xyz
```

#### 감사 쿼리 예시

```sql
-- 특정 기간의 AI 자동화 작업 조회
SELECT * FROM audit_logs
WHERE actor = 'claude-code'
  AND timestamp BETWEEN '2025-01-01' AND '2025-01-31'
ORDER BY timestamp DESC;

-- 실패한 작업 조회
SELECT * FROM audit_logs
WHERE result_status = 'failed'
  AND severity IN ('HIGH', 'CRITICAL');
```

#### 로그 저장소

```yaml
log_destinations:
  - type: file
    path: logs/ai-automation.log
    rotation: daily
    retention: 90d

  - type: cloud
    service: cloudwatch  # 또는 stackdriver
    log_group: /app/ai-automation
    retention: 365d
```

---

## 4. 수동 개입 트리거

### 4.1 사람 승인이 필요한 경우

#### 필수 승인 시나리오

| 시나리오 | 승인 필요 이유 | 승인자 |
|----------|---------------|--------|
| 프로덕션 브랜치 변경 | 서비스 안정성 | 테크 리드 |
| 보안 관련 코드 수정 | 보안 위험 | 보안 담당자 |
| 의존성 업데이트 (major) | 호환성 문제 | 개발팀 리드 |
| 인프라 설정 변경 | 서비스 영향 | DevOps 담당자 |
| 민감 데이터 접근 로직 | 개인정보 보호 | 보안 담당자 |
| 대규모 리팩토링 (100+ 파일) | 검토 필요성 | 테크 리드 |

#### 자동 승인 요청 트리거

```yaml
# GitHub Actions 예시
approval_required_when:
  - path_changed: "Cargo.toml"
    condition: "version_bump >= major"

  - path_changed: ".github/workflows/**"
    condition: "always"

  - path_changed: "codes/server/src/config/**"
    condition: "always"

  - lines_changed: "> 500"
    condition: "always"
```

### 4.2 에스컬레이션 프로세스

#### 에스컬레이션 매트릭스

```
Level 1: 개발자
    │ 30분 내 미응답
    ▼
Level 2: 테크 리드
    │ 1시간 내 미응답
    ▼
Level 3: CTO / VP Engineering
    │ 2시간 내 미응답 (업무시간 외)
    ▼
Level 4: 온콜 엔지니어 (긴급 상황)
```

#### 에스컬레이션 기준

| 심각도 | 응답 시간 | 해결 시간 | 알림 채널 |
|--------|----------|----------|----------|
| Critical | 15분 | 1시간 | Slack + 전화 |
| High | 30분 | 4시간 | Slack |
| Medium | 2시간 | 24시간 | Slack |
| Low | 8시간 | 1주 | 이메일 |

#### 알림 템플릿

```markdown
## AI 자동화 승인 요청

**요청 유형**: [코드 변경 / 의존성 업데이트 / 설정 변경]
**심각도**: [Critical / High / Medium / Low]
**요청자**: Claude Code (session: abc123)

### 변경 사항
- 파일: `codes/server/src/...`
- 변경 라인 수: 45

### 필요한 조치
- [ ] 변경 사항 검토
- [ ] 테스트 결과 확인
- [ ] 승인 또는 거부

**승인 기한**: 2025-01-15 12:00 UTC
```

---

## 5. 장애 대응 계획

### 5.1 장애 분류

| 등급 | 정의 | 예시 |
|------|------|------|
| **P0** | 서비스 전체 중단 | AI 자동화로 인한 프로덕션 장애 |
| **P1** | 주요 기능 장애 | 자동 배포 실패로 롤백 필요 |
| **P2** | 부분 기능 장애 | 특정 자동화 작업 실패 |
| **P3** | 경미한 문제 | 비필수 자동화 지연 |

### 5.2 대응 절차

```
[장애 감지]
    │
    ├── 자동 감지: 모니터링 알림
    └── 수동 보고: 개발자 리포트
    │
    ▼
[초기 대응] (15분 이내)
    │
    ├── 1. 영향 범위 파악
    ├── 2. 자동화 일시 중지 여부 결정
    └── 3. 담당자 할당
    │
    ▼
[원인 분석]
    │
    ├── 로그 분석
    ├── 변경 이력 검토
    └── 재현 시도
    │
    ▼
[해결 및 복구]
    │
    ├── 긴급 수정 또는 롤백
    ├── 테스트 검증
    └── 정상화 확인
    │
    ▼
[사후 분석]
    │
    ├── RCA (Root Cause Analysis) 작성
    ├── 재발 방지책 수립
    └── 문서화
```

### 5.3 롤백 절차

```bash
# 1. 문제 커밋 식별
git log --oneline --author="Claude" -10

# 2. 롤백 커밋 생성
git revert <commit_hash> --no-edit

# 3. CI 통과 확인
gh pr checks <pr_number>

# 4. 긴급 배포 (승인 필요)
gh workflow run emergency-deploy.yml
```

### 5.4 비상 연락망

```yaml
on_call_rotation:
  - week: 1
    primary: developer_a@example.com
    secondary: developer_b@example.com

  - week: 2
    primary: developer_b@example.com
    secondary: developer_c@example.com

emergency_contacts:
  tech_lead: +82-10-xxxx-xxxx
  devops: +82-10-yyyy-yyyy
  security: security@example.com
```

---

## 6. 규정 준수 체크리스트

### 6.1 배포 전 체크리스트

```markdown
## 보안 검토
- [ ] 민감정보가 코드에 하드코딩되지 않았는가?
- [ ] 입력 검증이 모든 엔드포인트에 적용되었는가?
- [ ] 인증/인가 로직이 올바르게 구현되었는가?
- [ ] SQL 인젝션, XSS 등 취약점이 없는가?

## 권한 검토
- [ ] 최소 권한 원칙이 적용되었는가?
- [ ] 서비스 계정 권한이 적절한가?
- [ ] 토큰 만료 정책이 설정되었는가?

## 로깅 검토
- [ ] 모든 중요 작업이 로깅되는가?
- [ ] 민감정보가 로그에 기록되지 않는가?
- [ ] 로그 보관 정책이 준수되는가?

## 자동화 검토
- [ ] 수동 승인이 필요한 작업이 명확히 정의되었는가?
- [ ] 롤백 절차가 테스트되었는가?
- [ ] 장애 대응 연락망이 최신 상태인가?
```

### 6.2 정기 감사 체크리스트 (월별)

```markdown
## 접근 권한 감사
- [ ] 불필요한 서비스 계정 비활성화
- [ ] 토큰 만료일 확인 및 갱신
- [ ] 권한 변경 로그 검토

## 보안 감사
- [ ] 취약점 스캔 결과 검토
- [ ] 의존성 보안 업데이트 확인
- [ ] 침입 탐지 로그 검토

## 자동화 감사
- [ ] 자동화 작업 성공률 검토
- [ ] 에러 패턴 분석
- [ ] 성능 메트릭 검토
```

### 6.3 규정 준수 매트릭스

| 규정 | 요구사항 | 현재 상태 | 담당자 |
|------|----------|----------|--------|
| 데이터 보호 | 개인정보 암호화 | 준수 | 보안팀 |
| 접근 제어 | 역할 기반 접근 | 준수 | DevOps |
| 감사 로깅 | 1년 이상 보관 | 준수 | 인프라팀 |
| 인시던트 대응 | 24시간 내 보고 | 준수 | 온콜팀 |

---

## 7. 개정 이력

| 버전 | 날짜 | 변경 내용 | 작성자 |
|------|------|----------|--------|
| 1.0 | 2025-02-01 | 최초 작성 | AI Automation Team |
| 1.1 | 2026-02-01 | 파이프라인 운영 예외 규정 추가 (섹션 1.4, 2.1 보강) | AI Automation Team |

---

## 8. 참고 자료

### 외부 문서
- [GitHub Actions 보안 가이드](https://docs.github.com/en/actions/security-guides)
- [OWASP 보안 체크리스트](https://owasp.org/www-project-web-security-testing-guide/)
- [12 Factor App - Config](https://12factor.net/config)

### 프로젝트 내부 문서
- [Phase 1: Event Trigger](./phase-1-event-trigger.md) - 이벤트 수신 및 트리거
- [Phase 2: Issue Analysis](./phase-2-issue-analysis.md) - AI 기반 이슈 분석 및 자동 브랜치 생성
- [Phase 3: AI Diagnostic](./phase-3-ai-diagnostic.md) - Claude API 기반 에러 진단
- [Phase 4: Issue Automation](./phase-4-issue-automation.md) - GitHub Issue 자동 생성
- [Phase 5: Auto-Fix & PR](./phase-5-auto-fix-pr.md) - 자동 코드 수정 및 PR 생성
- [Overview](./overview.md) - 전체 개요
- [CLAUDE.md](../../CLAUDE.md) - 프로젝트 코딩 규칙
