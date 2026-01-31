# AI Agent 설계

## Agent 개요

```
┌─────────────────────────────────────────────────────────────────┐
│                        AI Agent Pipeline                         │
│                                                                   │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │ Log Watcher  │───▶│  Diagnostic  │───▶│  Auto-Fix    │       │
│  │    Agent     │    │    Agent     │    │    Agent     │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│        │                   │                   │                 │
│        ▼                   ▼                   ▼                 │
│   로그 감지/필터      컨텍스트 분석        코드 수정 시도        │
│   이벤트 트리거       진단 보고서          Draft PR 생성         │
└─────────────────────────────────────────────────────────────────┘
```

## 1. Log Watcher Agent

### 역할
- 로그 파일/스트림 실시간 모니터링
- 이상 패턴 감지 및 이벤트 생성
- 중복 알림 방지 (Deduplication)

### 구현 방식

#### Option A: Shell Script (MVP)
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

# 마지막 처리 위치 읽기
LAST_LINE=$(cat "$STATE_FILE" 2>/dev/null || echo 0)

# 새 로그 라인 처리
tail -n +$((LAST_LINE + 1)) "$LOG_FILE" | while read -r line; do
    # ERROR 레벨 감지
    if echo "$line" | jq -e '.level == "ERROR"' > /dev/null 2>&1; then
        ERROR_CODE=$(echo "$line" | jq -r '.fields.error_code // "UNKNOWN"')
        MESSAGE=$(echo "$line" | jq -r '.message')
        TARGET=$(echo "$line" | jq -r '.target')

        # Fingerprint 생성 (중복 체크용)
        FINGERPRINT="${ERROR_CODE}:${TARGET}"

        # 중복 체크
        NOW=$(date +%s)
        LAST_SEEN=$(grep "^${FINGERPRINT}:" "$DEDUP_FILE" 2>/dev/null | cut -d: -f3)

        if [ -n "$LAST_SEEN" ] && [ $((NOW - LAST_SEEN)) -lt $DEDUP_WINDOW ]; then
            continue  # 5분 내 중복, 스킵
        fi

        # 중복 기록 갱신
        grep -v "^${FINGERPRINT}:" "$DEDUP_FILE" > "${DEDUP_FILE}.tmp" 2>/dev/null || true
        echo "${FINGERPRINT}:${NOW}" >> "${DEDUP_FILE}.tmp"
        mv "${DEDUP_FILE}.tmp" "$DEDUP_FILE"

        # Discord 알림 발송
        ./scripts/discord-alert.sh "critical" "🚨 [$ERROR_CODE] Error Detected" "$MESSAGE" "$ERROR_CODE"

        # Diagnostic Agent 트리거
        ./scripts/trigger-diagnostic.sh "$line"
    fi
done

# 현재 위치 저장
wc -l < "$LOG_FILE" > "$STATE_FILE"
```

**로그 로테이션 대응**:
- 상태 파일은 날짜별로 분리 저장 (`logs/.state/log-watcher-state-YYYY-MM-DD`)
- 날짜가 변경되면 새 상태 파일 사용 (이전 상태 무시)
- 같은 날짜에 로그 파일이 교체되면 inode 변경 감지하여 상태 리셋
- 7일 이상 된 상태 파일은 자동 정리

#### Option B: Rust 프로그램 (Production)
```rust
// monitor/src/watcher.rs
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

pub struct LogWatcher {
    log_path: PathBuf,
    error_pattern: Regex,
    dedup_cache: LruCache<String, Instant>,
}

impl LogWatcher {
    pub async fn watch(&mut self) -> Result<(), Error> {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(1))?;

        watcher.watch(&self.log_path, RecursiveMode::NonRecursive)?;

        loop {
            match rx.recv() {
                Ok(event) => self.handle_event(event).await?,
                Err(e) => error!("Watch error: {}", e),
            }
        }
    }

    async fn handle_event(&mut self, event: DebouncedEvent) -> Result<(), Error> {
        if let DebouncedEvent::Write(path) = event {
            let new_lines = self.read_new_lines(&path)?;

            for line in new_lines {
                if let Some(log) = self.parse_log_line(&line)? {
                    if self.should_alert(&log) {
                        self.trigger_diagnostic(log).await?;
                    }
                }
            }
        }
        Ok(())
    }

    fn should_alert(&mut self, log: &LogEntry) -> bool {
        // 중복 체크 (fingerprint 기반)
        let fingerprint = self.create_fingerprint(log);

        if let Some(last_seen) = self.dedup_cache.get(&fingerprint) {
            if last_seen.elapsed() < Duration::from_secs(300) {
                return false;  // 5분 내 중복
            }
        }

        self.dedup_cache.put(fingerprint, Instant::now());
        true
    }
}
```

### 감지 규칙

| 규칙 ID | 조건 | 심각도 | 액션 |
|---------|------|--------|------|
| `R001` | `level == "ERROR"` | Critical | 즉시 진단 |
| `R002` | `duration_ms > 5000` | Warning | 집계 후 알림 |
| `R003` | 5분 내 ERROR > 10건 | Critical | 즉시 진단 + 알림 |
| `R004` | `error_code starts with "DB_"` | Critical | 즉시 진단 |
| `R005` | `error_code == "AI_004"` (rate limit) | Warning | 집계 후 알림 |

### Fingerprint 생성

> **통일된 규칙**: 모든 문서에서 fingerprint는 `{error_code}:{target}` 형식을 사용합니다.
> 메시지 기반 변별이 필요한 경우, 선택적으로 sanitized_message를 추가할 수 있습니다.

```rust
fn create_fingerprint(log: &LogEntry) -> String {
    // 통일된 fingerprint 규칙: error_code + target
    // 이 규칙은 00-overview.md, log-watcher.sh 등 모든 문서에서 동일하게 적용
    format!(
        "{}:{}",
        log.error_code.as_deref().unwrap_or("UNKNOWN"),
        log.target
    )
}

// 선택적: 메시지 기반 추가 구분이 필요한 경우
fn create_detailed_fingerprint(log: &LogEntry) -> String {
    format!(
        "{}:{}:{}",
        log.error_code.as_deref().unwrap_or("UNKNOWN"),
        log.target,
        sanitize_message(&log.message)
    )
}

fn sanitize_message(msg: &str) -> String {
    // UUID, 숫자 등 변수 부분을 제거
    let re = Regex::new(r"[0-9a-f-]{36}|\d+").unwrap();
    re.replace_all(msg, "X").to_string()
}
```

## 2. Diagnostic Agent

### 역할
- 에러 컨텍스트 수집
- Claude API를 활용한 근본 원인 분석
- 구조화된 진단 보고서 생성

### 입력 데이터

```json
{
  "error_log": {
    "timestamp": "2025-01-31T14:23:45Z",
    "level": "ERROR",
    "error_code": "AI_003",
    "message": "Claude API timeout after 30000ms",
    "target": "server::domain::ai::service",
    "request_id": "req_abc123"
  },
  "context": {
    "recent_logs": ["...", "..."],  // 최근 50줄
    "source_file": "src/domain/ai/service.rs",
    "source_lines": "140-160",
    "source_content": "...",
    "recent_commits": [
      {
        "hash": "abc123",
        "message": "feat: add retry logic",
        "date": "2025-01-30"
      }
    ],
    "similar_errors": [
      {
        "date": "2025-01-28",
        "count": 5,
        "resolution": "Increased timeout"
      }
    ]
  }
}
```

### Diagnostic Prompt

```markdown
# 역할
당신은 Rust 백엔드 시스템의 에러 진단 전문가입니다.
제공된 로그와 코드를 분석하여 근본 원인을 파악하고 해결책을 제시합니다.

# 입력 데이터
## 에러 로그
```json
{error_log}
```

## 관련 소스 코드
파일: {source_file}
```rust
{source_content}
```

## 최근 관련 로그
```
{recent_logs}
```

## 최근 커밋
{recent_commits}

## 유사 에러 이력
{similar_errors}

# 분석 요청
다음 형식으로 진단 결과를 JSON으로 출력하세요:

```json
{
  "severity": "critical|warning|info",
  "root_cause": "근본 원인 설명",
  "impact": "영향 범위 설명",
  "recommendations": [
    {
      "priority": 1,
      "action": "권장 조치",
      "effort": "low|medium|high"
    }
  ],
  "auto_fixable": true/false,
  "fix_suggestion": "자동 수정이 가능한 경우 구체적인 코드 변경 제안"
}
```
```

### 출력 예시

```json
{
  "severity": "critical",
  "root_cause": "Claude API 호출 시 30초 타임아웃이 설정되어 있으나, 최근 트래픽 증가로 인해 API 응답 시간이 증가하여 타임아웃 발생. 재시도 로직이 있으나 3회 모두 실패.",
  "impact": "회고 어시스턴트 기능 전체 사용 불가. 현재 시점 기준 5분간 15건의 요청 실패.",
  "recommendations": [
    {
      "priority": 1,
      "action": "타임아웃 값을 30초에서 45초로 증가",
      "effort": "low"
    },
    {
      "priority": 2,
      "action": "지수 백오프 재시도 로직 개선 (현재 고정 1초 → 1s, 2s, 4s)",
      "effort": "medium"
    },
    {
      "priority": 3,
      "action": "비동기 처리로 전환하여 클라이언트 타임아웃과 서버 처리 분리",
      "effort": "high"
    }
  ],
  "auto_fixable": true,
  "fix_suggestion": "src/domain/ai/client.rs의 TIMEOUT_SECS 상수를 30에서 45로 변경"
}
```

### 구현

```rust
// monitor/src/diagnostic.rs
use anthropic::client::Client;

pub struct DiagnosticAgent {
    claude_client: Client,
    code_reader: CodeReader,
    git_client: GitClient,
}

impl DiagnosticAgent {
    pub async fn diagnose(&self, event: ErrorEvent) -> Result<DiagnosticReport, Error> {
        // 1. 컨텍스트 수집
        let source_context = self.collect_source_context(&event).await?;
        let git_context = self.collect_git_context(&event).await?;
        let history_context = self.collect_error_history(&event).await?;

        // 2. 프롬프트 구성
        let prompt = self.build_prompt(&event, &source_context, &git_context, &history_context);

        // 3. Claude API 호출
        // 모델명은 환경변수 DIAGNOSTIC_MODEL로 설정 (기본값: claude-sonnet-4-20250514)
        let model = std::env::var("DIAGNOSTIC_MODEL")
            .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string());
        let response = self.claude_client
            .messages()
            .create(MessagesRequest {
                model: model.into(),
                max_tokens: 2048,
                messages: vec![Message {
                    role: "user".into(),
                    content: prompt,
                }],
            })
            .await?;

        // 4. 응답 파싱
        let report: DiagnosticReport = serde_json::from_str(&response.content)?;

        Ok(report)
    }

    async fn collect_source_context(&self, event: &ErrorEvent) -> Result<SourceContext, Error> {
        // target에서 파일 경로 추출
        let file_path = self.target_to_path(&event.target)?;

        // 소스 코드 읽기
        let content = self.code_reader.read_file(&file_path)?;

        Ok(SourceContext {
            file_path,
            content,
            line_range: self.extract_line_range(event),
        })
    }
}
```

## 3. Auto-Fix Agent

### 역할
- 진단 결과 기반 자동 수정 시도
- Draft PR 생성
- 테스트 실행 및 검증

### 수정 가능 범위

#### 허용
| 유형 | 예시 |
|------|------|
| 설정 값 조정 | 타임아웃, 재시도 횟수, 버퍼 크기 |
| 로깅 개선 | 추가 컨텍스트 로깅 |
| 간단한 버그 | 오타, 누락된 null 체크 |
| 의존성 업데이트 | 패치 버전 업그레이드 |

#### 불허
| 유형 | 이유 |
|------|------|
| 아키텍처 변경 | 사람의 검토 필수 |
| 비즈니스 로직 | 요구사항 확인 필요 |
| 보안 코드 | 보안 검토 필수 |
| 대규모 리팩토링 | 영향 범위 불확실 |

### 워크플로우

```
Diagnostic Report (auto_fixable: true)
    │
    ▼
┌─────────────────────────────────────────┐
│ 1. 새 브랜치 생성                        │
│    fix/ai-timeout-{timestamp}           │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│ 2. Claude Code로 수정 적용               │
│    - fix_suggestion 기반 코드 수정       │
│    - 최소한의 변경만 적용                 │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│ 3. 테스트 실행                           │
│    cargo test                           │
│    cargo clippy                         │
└─────────────────────────────────────────┘
    │
    ├─── 실패 ───▶ 브랜치 삭제, 알림만 발송
    │
    ▼ 성공
┌─────────────────────────────────────────┐
│ 4. Draft PR 생성                         │
│    - 진단 보고서 포함                     │
│    - auto-fix 라벨 추가                  │
│    - 리뷰어 자동 할당                    │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│ 5. Discord 알림                          │
│    "자동 수정 PR이 생성되었습니다"        │
└─────────────────────────────────────────┘
```

### 구현 (Claude Code CLI 활용)

```bash
#!/bin/bash
# scripts/auto-fix.sh

DIAGNOSTIC_REPORT="$1"
BRANCH_NAME="fix/auto-$(date +%s)"

# 1. 브랜치 생성
git checkout -b "$BRANCH_NAME"

# 2. Claude Code로 수정 적용
FIX_SUGGESTION=$(echo "$DIAGNOSTIC_REPORT" | jq -r '.fix_suggestion')

claude --print "
다음 수정 사항을 적용해주세요:

$FIX_SUGGESTION

수정 후 테스트를 실행하지 마세요. 수정만 적용해주세요.
" | claude

# 3. 테스트 실행
if ! cargo test; then
    echo "테스트 실패, 브랜치 삭제"
    git checkout main
    git branch -D "$BRANCH_NAME"
    exit 1
fi

if ! cargo clippy -- -D warnings; then
    echo "Clippy 실패, 브랜치 삭제"
    git checkout main
    git branch -D "$BRANCH_NAME"
    exit 1
fi

# 4. 커밋 및 PR 생성
git add -A
git commit -m "fix: $(echo "$DIAGNOSTIC_REPORT" | jq -r '.root_cause' | head -c 50)

Auto-generated fix based on AI diagnostic.

Co-Authored-By: AI Monitor <ai-monitor@example.com>"

git push -u origin "$BRANCH_NAME"

# 5. Draft PR 생성
ROOT_CAUSE=$(echo "$DIAGNOSTIC_REPORT" | jq -r '.root_cause')
IMPACT=$(echo "$DIAGNOSTIC_REPORT" | jq -r '.impact')

gh pr create --draft \
    --title "fix: Auto-fix for $(echo "$DIAGNOSTIC_REPORT" | jq -r '.error_code')" \
    --body "$(cat <<EOF
## AI 자동 생성 PR

### 진단 결과
**심각도**: $(echo "$DIAGNOSTIC_REPORT" | jq -r '.severity')

**근본 원인**
$ROOT_CAUSE

**영향 범위**
$IMPACT

### 적용된 수정
$(echo "$DIAGNOSTIC_REPORT" | jq -r '.fix_suggestion')

---
이 PR은 AI 모니터링 시스템에 의해 자동 생성되었습니다.
반드시 사람이 검토한 후 머지해주세요.

Labels: \`auto-fix\`, \`ai-generated\`
EOF
)" \
    --label "auto-fix" \
    --label "ai-generated"
```

## Agent 간 통신

### 이벤트 스키마

```json
{
  "event_type": "error_detected | diagnostic_complete | fix_applied",
  "timestamp": "2025-01-31T14:23:45Z",
  "source_agent": "log_watcher | diagnostic | auto_fix",
  "payload": {
    // 이벤트별 상이
  }
}
```

### 메시지 큐 (향후)

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ Log Watcher  │────▶│    Queue     │────▶│  Diagnostic  │
└──────────────┘     │   (Redis)    │     └──────────────┘
                     └──────────────┘
                            │
                            ▼
                     ┌──────────────┐
                     │   Auto-Fix   │
                     └──────────────┘
```

## 에러 핸들링

### Agent 실패 시 동작

| Agent | 실패 상황 | 대응 |
|-------|----------|------|
| Log Watcher | 로그 파일 접근 불가 | 재시도 3회 후 관리자 알림 |
| Diagnostic | Claude API 실패 | 원본 에러 로그만 Discord 전송 |
| Auto-Fix | 테스트 실패 | 브랜치 삭제, 수동 조치 권고 알림 |

### 재시도 정책

```rust
pub struct RetryPolicy {
    max_attempts: u32,
    initial_delay: Duration,
    max_delay: Duration,
    multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
        }
    }
}
```
