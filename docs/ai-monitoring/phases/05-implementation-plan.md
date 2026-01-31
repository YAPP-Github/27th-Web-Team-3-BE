# 구현 계획

## 로드맵 개요

```
Phase 1: Foundation         Phase 2: MVP                Phase 3: AI                 Phase 4: Production
(Week 1-2)                  (Week 3-4)                  (Week 5-6)                  (Week 7-8)
┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐
│   로그 기반     │         │  모니터링 MVP   │         │   AI 진단       │         │   자동화 확장   │
│   구축          │────────▶│  구현           │────────▶│   연동          │────────▶│                 │
├─────────────────┤         ├─────────────────┤         ├─────────────────┤         ├─────────────────┤
│ - JSON 로깅     │         │ - Log Watcher   │         │ - Claude 연동   │         │ - Auto-Fix      │
│ - 에러 코드     │         │ - Discord 알림  │         │ - 진단 보고서   │         │ - GitHub 연동   │
│ - Request ID    │         │ - 기본 필터링   │         │ - 컨텍스트 수집 │         │ - 대시보드      │
└─────────────────┘         └─────────────────┘         └─────────────────┘         └─────────────────┘
```

## Phase 1 (Foundation): 로그 기반 구축

### 목표
- 구조화된 JSON 로그 포맷 적용
- 에러 코드 체계 수립
- Request ID 전파

### 태스크

#### 1.1 JSON 로그 포맷 적용
**파일**: `codes/server/src/main.rs`, `codes/server/src/utils/logging.rs`

```rust
// src/utils/logging.rs (신규)
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub fn init_logging() {
    // 중요: flatten_event(false)로 설정하여 fields 중첩 구조 유지
    // Log Watcher에서 .fields.error_code, .fields.request_id 등으로 접근 가능
    let fmt_layer = fmt::layer()
        .json()
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_current_span(true)
        .flatten_event(false);  // fields 중첩 구조 유지

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,server=debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}
```

**체크리스트**:
- [ ] `tracing-subscriber` JSON 포맷 설정
- [ ] 환경별 로그 레벨 설정 (RUST_LOG)
- [ ] 로그 파일 출력 추가 (옵션)

#### 1.2 에러 코드 체계 적용
**파일**: `codes/server/src/utils/error.rs`

```rust
// 기존 AppError에 error_code 필드 추가
#[derive(Debug)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub enum ErrorCode {
    // AI 관련
    AiAuthFailed,      // AI_001
    AiInvalidInput,    // AI_002
    AiTimeout,         // AI_003
    AiRateLimit,       // AI_004
    AiInternalError,   // AI_005

    // Auth 관련
    AuthTokenMissing,  // AUTH_001
    AuthTokenExpired,  // AUTH_002
    AuthTokenInvalid,  // AUTH_003
    AuthForbidden,     // AUTH_004

    // DB 관련
    DbConnectionFailed, // DB_001
    DbQueryTimeout,     // DB_002
    DbTransactionFailed, // DB_003
    DbNotFound,         // DB_004

    // 일반
    ValidationError,   // VAL_001
    InternalError,     // COMMON_500
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AiAuthFailed => "AI_001",
            Self::AiTimeout => "AI_003",
            // ...
        }
    }
}
```

**체크리스트**:
- [ ] ErrorCode enum 정의
- [ ] 기존 에러 타입 마이그레이션
- [ ] 에러 로깅 시 error_code 포함

#### 1.3 Request ID 미들웨어
**파일**: `codes/server/src/global/middleware.rs`

```rust
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct RequestId(pub String);

pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    request.extensions_mut().insert(RequestId(request_id.clone()));

    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri().path(),
    );

    let _guard = span.enter();

    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap(),
    );

    response
}
```

**체크리스트**:
- [ ] Request ID 미들웨어 구현
- [ ] 모든 로그에 request_id 포함
- [ ] 응답 헤더에 request_id 반환

### 산출물
- JSON 형식 로그 출력
- 에러 코드가 포함된 에러 로그
- Request ID로 추적 가능한 요청 로그

---

## Phase 2: 모니터링 MVP

### 목표
- Log Watcher 스크립트 구현
- Discord Webhook 연동
- 기본 알림 필터링

### 태스크

#### 2.1 Discord Webhook 연동
**파일**: `scripts/discord-alert.sh`

```bash
#!/bin/bash
# scripts/discord-alert.sh

WEBHOOK_URL="${DISCORD_WEBHOOK_URL}"
SEVERITY="$1"
TITLE="$2"
MESSAGE="$3"
ERROR_CODE="$4"

# 색상 설정
case "$SEVERITY" in
    critical) COLOR=15158332 ;;  # Red
    warning)  COLOR=16776960 ;;  # Yellow
    info)     COLOR=3066993 ;;   # Green
esac

curl -H "Content-Type: application/json" \
     -X POST \
     -d "{
       \"embeds\": [{
         \"title\": \"$TITLE\",
         \"description\": \"$MESSAGE\",
         \"color\": $COLOR,
         \"footer\": {
           \"text\": \"Error Code: $ERROR_CODE\"
         },
         \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
       }]
     }" \
     "$WEBHOOK_URL"
```

**체크리스트**:
- [ ] Discord Webhook 생성
- [ ] 환경 변수 설정 (.env)
- [ ] 알림 스크립트 작성

#### 2.2 Log Watcher 스크립트
**파일**: `scripts/log-watcher.sh`

```bash
#!/bin/bash
# scripts/log-watcher.sh

LOG_DIR="${LOG_DIR:-./logs}"
STATE_DIR="${STATE_DIR:-./logs/.state}"  # 상태 파일을 프로젝트 내에 저장
DEDUP_WINDOW=300  # 5분

# 상태 디렉토리 생성
mkdir -p "$STATE_DIR"

# 오늘 로그 파일
TODAY=$(date +%Y-%m-%d)
LOG_FILE="$LOG_DIR/server.${TODAY}.log"

# 날짜별 상태 파일 (로그 로테이션 대응)
STATE_FILE="$STATE_DIR/log-watcher-state-${TODAY}"
DEDUP_FILE="$STATE_DIR/log-watcher-dedup-${TODAY}"

# 오래된 상태 파일 정리 (7일 이상)
find "$STATE_DIR" -name "log-watcher-*" -mtime +7 -delete 2>/dev/null || true

# 상태 파일 초기화
touch "$STATE_FILE" "$DEDUP_FILE"

if [ ! -f "$LOG_FILE" ]; then
    echo "Log file not found: $LOG_FILE"
    exit 0
fi

# 현재 로그 파일의 inode 확인 (파일 교체 감지용)
CURRENT_INODE=$(stat -f%i "$LOG_FILE" 2>/dev/null || stat -c%i "$LOG_FILE" 2>/dev/null)
SAVED_INODE=$(cat "$STATE_FILE.inode" 2>/dev/null || echo "")

# inode가 변경되었으면 새 파일로 간주하고 처음부터 읽기
if [ -n "$SAVED_INODE" ] && [ "$CURRENT_INODE" != "$SAVED_INODE" ]; then
    echo "Log file rotated (inode changed), resetting state"
    echo "0" > "$STATE_FILE"
fi
echo "$CURRENT_INODE" > "$STATE_FILE.inode"

# 마지막 처리 라인
LAST_LINE=$(cat "$STATE_FILE" 2>/dev/null || echo 0)

# 새 라인 처리
tail -n +$((LAST_LINE + 1)) "$LOG_FILE" | while read -r line; do
    LEVEL=$(echo "$line" | jq -r '.level' 2>/dev/null)

    if [ "$LEVEL" = "ERROR" ]; then
        ERROR_CODE=$(echo "$line" | jq -r '.fields.error_code // "UNKNOWN"')
        MESSAGE=$(echo "$line" | jq -r '.message')
        TARGET=$(echo "$line" | jq -r '.target')

        # 중복 체크 (5분 내 동일 에러)
        FINGERPRINT="${ERROR_CODE}:${TARGET}"
        NOW=$(date +%s)
        LAST_SEEN=$(grep "^$FINGERPRINT:" "$DEDUP_FILE" 2>/dev/null | cut -d: -f3)

        if [ -n "$LAST_SEEN" ] && [ $((NOW - LAST_SEEN)) -lt $DEDUP_WINDOW ]; then
            continue  # 5분 내 중복, 스킵
        fi

        # 중복 기록 갱신
        grep -v "^$FINGERPRINT:" "$DEDUP_FILE" > "${DEDUP_FILE}.tmp" 2>/dev/null || true
        echo "$FINGERPRINT:$NOW" >> "${DEDUP_FILE}.tmp"
        mv "${DEDUP_FILE}.tmp" "$DEDUP_FILE"

        # Discord 알림
        ./scripts/discord-alert.sh "critical" \
            "🚨 [$ERROR_CODE] Error Detected" \
            "Location: $TARGET\n\nMessage: $MESSAGE" \
            "$ERROR_CODE"
    fi
done

# 현재 라인 수 저장
wc -l < "$LOG_FILE" > "$STATE_FILE"
```

**로그 로테이션 대응**:
- 상태 파일은 날짜별로 분리 저장 (`logs/.state/log-watcher-state-YYYY-MM-DD`)
- 날짜가 변경되면 새 상태 파일 사용 (이전 상태 무시)
- 같은 날짜에 로그 파일이 교체되면 inode 변경 감지하여 상태 리셋
- 7일 이상 된 상태 파일은 자동 정리

**체크리스트**:
- [ ] Log Watcher 스크립트 작성
- [ ] 중복 제거 로직 구현
- [ ] Cron 설정 (5분 간격)

#### 2.3 Cron 설정
**파일**: `scripts/setup-cron.sh`

```bash
#!/bin/bash
# scripts/setup-cron.sh

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
LOG_DIR="$PROJECT_DIR/logs"

# 로그 디렉토리 생성
mkdir -p "$LOG_DIR"

# 기존 log-watcher cron 제거
crontab -l 2>/dev/null | grep -v "log-watcher.sh" > /tmp/crontab.tmp || true

# 새 cron 추가 (5분 간격)
# - .env 파일을 source하여 환경변수 로드 (DISCORD_WEBHOOK_URL 등)
# - PATH 설정으로 jq, curl 등 명령어 사용 가능하게 함
# - 로그는 프로젝트 logs/ 디렉토리에 저장 (/var/log/ 권한 문제 방지)
echo "*/5 * * * * cd $PROJECT_DIR && export PATH=/usr/local/bin:/usr/bin:\$PATH && [ -f .env ] && export \$(grep -v '^#' .env | xargs) && ./scripts/log-watcher.sh >> $LOG_DIR/ai-monitor.log 2>&1" >> /tmp/crontab.tmp

# crontab 적용
crontab /tmp/crontab.tmp
rm /tmp/crontab.tmp

echo "Cron job installed. Running every 5 minutes."
echo "Log output: $LOG_DIR/ai-monitor.log"
```

**체크리스트**:
- [ ] Cron 스크립트 작성
- [ ] .env 파일에 DISCORD_WEBHOOK_URL 설정
- [ ] 서버 배포 시 자동 설정

### 산출물
- 동작하는 Discord 알림
- 5분 간격 로그 모니터링
- 중복 알림 방지

---

## Phase 3: AI 진단 연동

### 목표
- Claude API 기반 진단
- 컨텍스트 수집 (소스 코드, git 이력)
- 구조화된 진단 보고서

### 태스크

#### 3.1 Diagnostic Agent
**파일**: `scripts/diagnostic-agent.py`

```python
#!/usr/bin/env python3
# scripts/diagnostic-agent.py

import os
import json
import subprocess
from anthropic import Anthropic

client = Anthropic()

def collect_source_context(target: str) -> str:
    """target에서 소스 파일 추출하고 읽기"""
    # server::domain::ai::service → src/domain/ai/service.rs
    path = target.replace("server::", "src/").replace("::", "/") + ".rs"

    if os.path.exists(f"codes/server/{path}"):
        with open(f"codes/server/{path}") as f:
            return f.read()
    return ""

def collect_git_context(path: str) -> str:
    """최근 커밋 이력"""
    result = subprocess.run(
        ["git", "log", "--oneline", "-5", "--", path],
        capture_output=True, text=True, cwd="codes/server"
    )
    return result.stdout

def diagnose(error_log: dict) -> dict:
    """Claude API로 진단"""
    target = error_log.get("target", "")
    source = collect_source_context(target)
    git_log = collect_git_context(target)

    prompt = f"""
    # 에러 로그 분석

    ## 에러 정보
    ```json
    {json.dumps(error_log, indent=2)}
    ```

    ## 관련 소스 코드
    ```rust
    {source[:2000]}  # 길이 제한
    ```

    ## 최근 커밋
    {git_log}

    ## 요청
    JSON 형식으로 진단 결과를 제공해주세요:
    {{
      "severity": "critical|warning|info",
      "root_cause": "근본 원인",
      "impact": "영향 범위",
      "recommendations": [
        {{"priority": 1, "action": "조치 내용", "effort": "low|medium|high"}}
      ],
      "auto_fixable": true/false,
      "fix_suggestion": "수정 제안 (auto_fixable이 true인 경우)"
    }}
    """

    response = client.messages.create(
        model="claude-sonnet-4-20250514",
        max_tokens=2048,
        messages=[{"role": "user", "content": prompt}]
    )

    # JSON 파싱
    content = response.content[0].text
    # JSON 블록 추출
    import re
    json_match = re.search(r'\{[\s\S]*\}', content)
    if json_match:
        return json.loads(json_match.group())
    return {"error": "Failed to parse response"}

if __name__ == "__main__":
    import sys
    error_log = json.loads(sys.argv[1])
    result = diagnose(error_log)
    print(json.dumps(result, ensure_ascii=False, indent=2))
```

**체크리스트**:
- [ ] Python 스크립트 작성
- [ ] Anthropic 패키지 설치
- [ ] 컨텍스트 수집 로직 구현
- [ ] 프롬프트 최적화

#### 3.2 Log Watcher 연동
**파일**: `scripts/log-watcher.sh` (수정)

```bash
# ERROR 감지 시 진단 Agent 호출 추가
if [ "$LEVEL" = "ERROR" ]; then
    # ... 기존 중복 체크 로직 ...

    # 진단 Agent 호출
    DIAGNOSTIC_RESULT=$(python3 ./scripts/diagnostic-agent.py "$line")

    # 진단 결과로 알림 보강
    SEVERITY=$(echo "$DIAGNOSTIC_RESULT" | jq -r '.severity')
    ROOT_CAUSE=$(echo "$DIAGNOSTIC_RESULT" | jq -r '.root_cause')

    ./scripts/discord-alert.sh "$SEVERITY" \
        "🚨 [$ERROR_CODE] $MESSAGE" \
        "**근본 원인**: $ROOT_CAUSE\n\n**위치**: $TARGET" \
        "$ERROR_CODE"
fi
```

**체크리스트**:
- [ ] 진단 Agent 호출 연동
- [ ] 진단 결과로 알림 개선
- [ ] 에러 처리 (진단 실패 시)

### 산출물
- Claude 기반 자동 진단
- 근본 원인이 포함된 알림
- 권장 조치 목록

---

## Phase 4: 자동화 확장

### 목표
- GitHub Issue 자동 생성
- Auto-Fix PR 생성
- 대시보드 연동 (선택)

### 태스크

#### 4.1 GitHub Issue 자동 생성
**파일**: `scripts/create-issue.sh`

```bash
#!/bin/bash
# scripts/create-issue.sh

DIAGNOSTIC="$1"

ERROR_CODE=$(echo "$DIAGNOSTIC" | jq -r '.error_code')
SEVERITY=$(echo "$DIAGNOSTIC" | jq -r '.severity')
ROOT_CAUSE=$(echo "$DIAGNOSTIC" | jq -r '.root_cause')
IMPACT=$(echo "$DIAGNOSTIC" | jq -r '.impact')

# 중복 체크
EXISTING=$(gh issue list --label "ai-generated" --search "$ERROR_CODE in:title" --state open --json number --jq '.[0].number')

if [ -n "$EXISTING" ]; then
    gh issue comment "$EXISTING" --body "### 추가 발생 ($(date '+%Y-%m-%d %H:%M'))"
    exit 0
fi

# 새 이슈 생성
gh issue create \
    --title "[AI Monitor] $ERROR_CODE: $(echo "$ROOT_CAUSE" | head -c 50)" \
    --body "## AI 자동 생성 이슈

### 심각도
$SEVERITY

### 근본 원인
$ROOT_CAUSE

### 영향 범위
$IMPACT

### 권장 조치
$(echo "$DIAGNOSTIC" | jq -r '.recommendations[] | "- [\(.effort)] \(.action)"')

---
_AI 모니터링 시스템 자동 생성_" \
    --label "bug,ai-generated,priority:$SEVERITY"
```

**체크리스트**:
- [ ] gh CLI 설치 및 인증
- [ ] 이슈 생성 스크립트
- [ ] 중복 이슈 방지 로직

#### 4.2 Auto-Fix Agent
**파일**: `scripts/auto-fix.sh`

```bash
#!/bin/bash
# scripts/auto-fix.sh

DIAGNOSTIC="$1"
AUTO_FIXABLE=$(echo "$DIAGNOSTIC" | jq -r '.auto_fixable')

if [ "$AUTO_FIXABLE" != "true" ]; then
    echo "Not auto-fixable"
    exit 0
fi

FIX_SUGGESTION=$(echo "$DIAGNOSTIC" | jq -r '.fix_suggestion')
ERROR_CODE=$(echo "$DIAGNOSTIC" | jq -r '.error_code')
BRANCH="fix/auto-${ERROR_CODE}-$(date +%s)"

# 1. 브랜치 생성
git checkout -b "$BRANCH"

# 2. Claude Code로 수정 적용
echo "$FIX_SUGGESTION" | claude --print "다음 수정을 적용해주세요: $FIX_SUGGESTION"

# 3. 테스트
cd codes/server
if ! cargo test; then
    git checkout dev
    git branch -D "$BRANCH"
    echo "Tests failed, aborting"
    exit 1
fi

# 4. 커밋 및 PR
git add -A
git commit -m "fix($ERROR_CODE): auto-fix based on AI diagnostic

Co-Authored-By: AI Monitor <ai@monitor.local>"

git push -u origin "$BRANCH"

gh pr create --draft \
    --title "fix($ERROR_CODE): Auto-fix" \
    --body "## AI 자동 수정 PR

$FIX_SUGGESTION

---
_검토 후 머지해주세요_" \
    --label "auto-fix,ai-generated"
```

**체크리스트**:
- [ ] Auto-Fix 스크립트 작성
- [ ] 테스트 통과 검증
- [ ] Draft PR 생성
- [ ] Discord에 PR 링크 알림

### 산출물
- 자동 GitHub Issue 생성
- 조건부 Auto-Fix PR
- 완전한 모니터링 파이프라인

---

## 환경 설정

### 필수 환경 변수

```bash
# .env.example에 추가
# AI Monitoring
DISCORD_WEBHOOK_URL=https://discord.com/api/webhooks/xxx/yyy
ANTHROPIC_API_KEY=sk-ant-xxx
GITHUB_TOKEN=ghp_xxx  # gh CLI 인증용
```

### 의존성

```bash
# Python (진단 Agent)
pip install anthropic

# Rust (로깅 개선)
cargo add tracing-subscriber --features json

# CLI
brew install gh jq
```

## 위험 요소 및 대응

| 위험 | 영향 | 대응 방안 |
|------|------|----------|
| Claude API 비용 증가 | 높음 | 진단 호출 제한 (시간당 10회) |
| 잘못된 Auto-Fix | 중간 | Draft PR만 생성, 테스트 필수 |
| 알림 피로 | 중간 | 집계 알림, 중복 제거 |
| 로그 저장소 부족 | 낮음 | 로테이션 설정 (7일) |

## 성공 지표

| 지표 | 목표 |
|------|------|
| 장애 감지 시간 | < 5분 |
| 진단 정확도 | > 70% |
| Auto-Fix 성공률 | > 50% (시도 대비) |
| 알림 응답 시간 | < 30분 |
