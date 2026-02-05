# Phase 1: 이벤트 수신 및 트리거 시스템

## 개요

| 항목 | 내용 |
|------|------|
| Phase | 1: Event Trigger System |
| 기간 | Week 1-2 |
| 목표 | 다양한 이벤트 소스를 통합하고 AI 자동화 파이프라인을 트리거하는 시스템 구축 |
| 의존성 | AI 모니터링 시스템 (Phase 2 MVP 완료) |

```
Phase 1 완료 상태
┌─────────────────────────────────────────────────────────────────────────────┐
│  ⬜ Discord Webhook 수신    ⬜ 모니터링 연동    ⬜ GitHub Issue 이벤트     │
│  ⬜ 이벤트 큐 시스템        ⬜ 트리거 엔진      ⬜ 필터링/우선순위         │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 1. 목표 및 범위

### 1.1 목표

이벤트 수신 및 트리거 시스템은 다양한 소스에서 발생하는 이벤트를 수집하고, 적절한 조건에 따라 AI 자동화 파이프라인을 활성화합니다.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Event Trigger System                                 │
│                                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                       │
│  │   Discord    │  │  Monitoring  │  │   GitHub     │                       │
│  │   Webhook    │  │    Alerts    │  │   Events     │                       │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                       │
│         │                 │                 │                                │
│         └────────────────┬┴─────────────────┘                                │
│                          ▼                                                   │
│                   ┌─────────────┐                                            │
│                   │   Event     │                                            │
│                   │   Router    │                                            │
│                   └──────┬──────┘                                            │
│                          │                                                   │
│         ┌────────────────┼────────────────┐                                  │
│         ▼                ▼                ▼                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                          │
│  │   Filter    │  │  Priority   │  │   Dedup     │                          │
│  │   Engine    │  │   Queue     │  │   Check     │                          │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                          │
│         │                │                │                                  │
│         └────────────────┴────────────────┘                                  │
│                          │                                                   │
│                          ▼                                                   │
│                   ┌─────────────┐                                            │
│                   │  AI Agent   │                                            │
│                   │  Trigger    │                                            │
│                   └─────────────┘                                            │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 범위

#### In Scope

- Discord 웹훅 수신 엔드포인트 구축
- 기존 모니터링 시스템 (`scripts/log-watcher.sh`) 연동
- GitHub Webhook 수신 (Issue, PR 이벤트)
- 이벤트 필터링 및 우선순위 로직
- 이벤트 큐 시스템 (Redis 또는 파일 기반)
- 중복 이벤트 제거 (Deduplication)

#### Out of Scope

- AI 기반 이슈 분석 (Phase 2: Issue Analysis)
- AI 코드 수정 및 테스트 (Phase 3: Code Fix)
- PR 생성 및 리뷰 요청 (Phase 4: PR Creation)
- 대시보드 UI (추후 검토)

> **Phase 역할 분담**
> - Phase 1: 이벤트 수신 및 트리거 (현재 문서)
> - Phase 2: 이슈 분석 및 브랜치 생성
> - Phase 3: AI 코드 수정 및 테스트 검증
> - Phase 4: PR 생성 및 리뷰 요청

---

## 2. 이벤트 소스 정의

### 2.1 Discord 웹훅 수신

Discord에서 특정 명령어나 멘션을 통해 AI 자동화를 트리거합니다.

#### 수신 엔드포인트

```
POST /api/webhooks/discord
Content-Type: application/json
```

#### 지원 명령어

| 명령어 | 설명 | 트리거 액션 |
|--------|------|------------|
| `@AI 분석해줘` | 특정 에러/이슈 분석 요청 | AI 진단 Agent 실행 |
| `@AI 수정해줘` | 특정 버그 자동 수정 요청 | Auto-Fix Agent 실행 |
| `@AI 리뷰해줘` | PR 코드 리뷰 요청 | Code Review Agent 실행 |
| `@AI 상태` | 시스템 상태 확인 | Health Check 응답 |

#### 페이로드 구조

```json
{
  "type": 1,
  "token": "webhook_token",
  "channel_id": "123456789",
  "guild_id": "987654321",
  "data": {
    "content": "@AI 분석해줘 AI5001 에러가 계속 발생해요",
    "author": {
      "id": "user_id",
      "username": "developer"
    },
    "timestamp": "2025-01-31T14:23:45Z"
  }
}
```

#### 구현

**파일**: `codes/server/src/domain/webhook/discord_handler.rs`

```rust
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordWebhookPayload {
    pub r#type: i32,
    pub token: String,
    pub channel_id: String,
    pub guild_id: Option<String>,
    pub data: Option<DiscordMessageData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordMessageData {
    pub content: String,
    pub author: DiscordAuthor,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct DiscordAuthor {
    pub id: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordWebhookResponse {
    pub r#type: i32,
    pub data: Option<DiscordResponseData>,
}

#[derive(Debug, Serialize)]
pub struct DiscordResponseData {
    pub content: String,
}

/// Discord 서명 검증
/// Discord는 Ed25519 서명을 사용하여 요청의 무결성을 검증합니다.
/// 참고: https://discord.com/developers/docs/interactions/receiving-and-responding#security-and-authorization
///
/// 필수 의존성 (Cargo.toml):
/// ```toml
/// ed25519-dalek = "2.0"
/// hex = "0.4"
/// ```
fn verify_discord_signature(
    public_key: &str,
    signature: &str,
    timestamp: &str,
    body: &[u8],
) -> Result<(), AppError> {
    use ed25519_dalek::{PublicKey, Signature, Verifier};

    // 1. Public Key 파싱 (hex -> bytes)
    let public_key_bytes = hex::decode(public_key)
        .map_err(|_| AppError::Internal("Invalid public key format"))?;
    let public_key = PublicKey::from_bytes(&public_key_bytes)
        .map_err(|_| AppError::Internal("Invalid public key"))?;

    // 2. Signature 파싱 (hex -> bytes)
    let signature_bytes = hex::decode(signature)
        .map_err(|_| AppError::Unauthorized("Invalid signature format"))?;
    let signature = Signature::from_bytes(&signature_bytes)
        .map_err(|_| AppError::Unauthorized("Invalid signature"))?;

    // 3. 메시지 구성: timestamp + body
    let mut message = timestamp.as_bytes().to_vec();
    message.extend_from_slice(body);

    // 4. 서명 검증
    public_key
        .verify(&message, &signature)
        .map_err(|_| AppError::Unauthorized("Signature verification failed"))?;

    Ok(())
}

pub async fn handle_discord_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<DiscordWebhookResponse>, AppError> {
    // 1. 헤더에서 서명 정보 추출
    let signature = headers
        .get("X-Signature-Ed25519")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized("Missing X-Signature-Ed25519 header"))?;

    let timestamp = headers
        .get("X-Signature-Timestamp")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized("Missing X-Signature-Timestamp header"))?;

    // 2. 서명 검증 (DISCORD_PUBLIC_KEY 환경 변수 필요)
    let public_key = std::env::var("DISCORD_PUBLIC_KEY")
        .map_err(|_| AppError::Internal("DISCORD_PUBLIC_KEY not configured"))?;

    verify_discord_signature(&public_key, signature, timestamp, &body)?;

    // 3. 페이로드 파싱
    let payload: DiscordWebhookPayload = serde_json::from_slice(&body)
        .map_err(|e| AppError::BadRequest(format!("Invalid payload: {}", e)))?;

    // 4. Discord ping 응답 (type 1)
    if payload.r#type == 1 {
        return Ok(Json(DiscordWebhookResponse {
            r#type: 1,
            data: None,
        }));
    }

    // 5. 메시지 처리
    if let Some(data) = payload.data {
        let event = parse_discord_command(&data.content)?;
        state.event_queue.push(event).await?;
    }

    Ok(Json(DiscordWebhookResponse {
        r#type: 4,
        data: Some(DiscordResponseData {
            content: "요청을 접수했습니다. 처리 중...".to_string(),
        }),
    }))
}
```

> **필수 환경 변수**: Discord 서명 검증을 위해 `DISCORD_PUBLIC_KEY` 환경 변수가 설정되어야 합니다.
> Discord Developer Portal > Application > General Information에서 PUBLIC KEY를 확인할 수 있습니다.

### 2.2 모니터링 알림 연동

기존 `scripts/log-watcher.sh` 시스템과 연동하여 에러 로그 감지 시 이벤트를 트리거합니다.

#### 기존 시스템 흐름

```
[Rust Server] → [JSON Log] → [log-watcher.sh] → [discord-alert.sh]
                                    │
                                    └─→ [diagnostic-agent.py]
                                              │
                                              └─→ [create-issue.sh / auto-fix.sh]
```

#### 연동 포인트

| 파일 | 연동 방식 | 이벤트 타입 |
|------|----------|------------|
| `scripts/log-watcher.sh` | 직접 호출 | `monitoring.error_detected` |
| `scripts/discord-alert.sh` | 이벤트 발행 후 실행 | `notification.discord_sent` |
| `scripts/diagnostic-agent.py` | 이벤트로 대체 | `ai.diagnostic_requested` |

#### 에러 심각도 분류 (기존 시스템 연동)

기존 `log-watcher.sh`의 심각도 분류를 활용합니다:

```bash
# Critical: 즉시 알림 + AI 진단 + GitHub Issue
CRITICAL_CODES="COMMON500|AI5001|AI5002|AI5003|AI5031"

# Warning: 알림만 (AI 진단 없음)
WARNING_CODES="AUTH4001|AUTH4002|AUTH4003|AUTH4004|AUTH4005"

# Info: 로그만 (알림 없음)
# (CRITICAL, WARNING에 해당하지 않는 모든 에러)
```

#### 이벤트 페이로드

```json
{
  "event_type": "monitoring.error_detected",
  "source": "log-watcher",
  "timestamp": "2025-01-31T14:23:45Z",
  "data": {
    "error_code": "AI5001",
    "severity": "critical",
    "message": "Claude API timeout after 30000ms",
    "target": "server::domain::ai::service",
    "request_id": "req_abc123",
    "log_line": "{\"timestamp\":\"...\",\"level\":\"ERROR\",...}"
  },
  "metadata": {
    "fingerprint": "sha256_hash_here",
    "occurrence_count": 3,
    "first_seen": "2025-01-31T14:20:00Z"
  }
}
```

### 2.3 GitHub Webhook 이벤트

GitHub Webhook을 통해 Issue, PR, Comment 이벤트를 수신합니다.

#### 수신 엔드포인트

```
POST /api/webhooks/github
Content-Type: application/json
X-GitHub-Event: <event_type>
X-Hub-Signature-256: sha256=...
```

> **중요**: `X-GitHub-Event` 헤더 값은 이벤트 타입에 따라 달라집니다.
> - Issue 관련: `issues`
> - 코멘트 관련: `issue_comment`
> - PR 관련: `pull_request`

#### 지원 이벤트 타입 및 처리 방식

| X-GitHub-Event 헤더 | 액션 (action 필드) | 트리거 조건 | 처리 방식 | 우선순위 |
|---------------------|-------------------|------------|----------|----------|
| `issues` | `opened` | 라벨에 `ai-review` 포함 | AI 진단 Agent 실행 | P1 |
| `issues` | `labeled` | `ai-fix` 라벨 추가 시 | Auto-Fix Agent 실행 | P1 |
| `issue_comment` | `created` | `@ai-bot` 멘션 포함 | 멘션 명령어 파싱 후 해당 Agent 실행 | P1 |
| `pull_request` | `opened` | 라벨에 `ai-review` 포함 | Code Review Agent 실행 | P2 |
| `pull_request` | `labeled` | `ai-review` 라벨 추가 시 | Code Review Agent 실행 | P2 |

#### X-GitHub-Event 헤더 기반 분기 처리

```rust
use axum::{
    extract::{State, Json},
    http::HeaderMap,
};

/// X-GitHub-Event 헤더 값에 따른 이벤트 타입
#[derive(Debug, Clone, PartialEq)]
pub enum GitHubEventType {
    Issues,
    IssueComment,
    PullRequest,
    Unknown(String),
}

impl From<&str> for GitHubEventType {
    fn from(s: &str) -> Self {
        match s {
            "issues" => GitHubEventType::Issues,
            "issue_comment" => GitHubEventType::IssueComment,
            "pull_request" => GitHubEventType::PullRequest,
            other => GitHubEventType::Unknown(other.to_string()),
        }
    }
}

/// GitHub Webhook 핸들러 - 이벤트 타입별 분기 처리
pub async fn handle_github_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: bytes::Bytes,
) -> Result<Json<serde_json::Value>, AppError> {
    // 1. 서명 검증
    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized("Missing signature header"))?;

    verify_github_signature(&state.config.github_webhook_secret, signature, &body)?;

    // 2. 이벤트 타입 추출 (X-GitHub-Event 헤더)
    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .map(GitHubEventType::from)
        .ok_or(AppError::BadRequest("Missing X-GitHub-Event header"))?;

    // 3. 이벤트 타입별 분기 처리
    let event = match event_type {
        GitHubEventType::Issues => {
            let payload: IssuesPayload = serde_json::from_slice(&body)?;
            handle_issues_event(payload).await?
        }
        GitHubEventType::IssueComment => {
            let payload: IssueCommentPayload = serde_json::from_slice(&body)?;
            handle_issue_comment_event(payload).await?
        }
        GitHubEventType::PullRequest => {
            let payload: PullRequestPayload = serde_json::from_slice(&body)?;
            handle_pull_request_event(payload).await?
        }
        GitHubEventType::Unknown(event_name) => {
            tracing::warn!(event = %event_name, "Unsupported GitHub event type");
            return Ok(Json(serde_json::json!({
                "status": "ignored",
                "reason": format!("Unsupported event type: {}", event_name)
            })));
        }
    };

    // 4. 이벤트 큐에 추가
    if let Some(event) = event {
        state.event_queue.push(event).await?;
    }

    Ok(Json(serde_json::json!({"status": "accepted"})))
}
```

#### 이벤트 타입별 핸들러 구현

```rust
/// Issues 이벤트 처리 (opened, labeled)
async fn handle_issues_event(payload: IssuesPayload) -> Result<Option<Event>, AppError> {
    match payload.action.as_str() {
        "opened" => {
            // ai-review 라벨 확인
            if payload.issue.labels.iter().any(|l| l.name == "ai-review") {
                return Ok(Some(Event::new(
                    "github.issue_opened",
                    "github",
                    Priority::P1,
                    serde_json::to_value(&payload)?,
                )));
            }
        }
        "labeled" => {
            // ai-fix 라벨 추가 시
            if payload.label.as_ref().map(|l| l.name.as_str()) == Some("ai-fix") {
                return Ok(Some(Event::new(
                    "github.issue_labeled",
                    "github",
                    Priority::P1,
                    serde_json::to_value(&payload)?,
                )));
            }
        }
        _ => {}
    }
    Ok(None)
}

/// Issue Comment 이벤트 처리 (created)
async fn handle_issue_comment_event(payload: IssueCommentPayload) -> Result<Option<Event>, AppError> {
    if payload.action != "created" {
        return Ok(None);
    }

    // @ai-bot 멘션 확인
    if payload.comment.body.contains("@ai-bot") {
        return Ok(Some(Event::new(
            "github.issue_comment_created",
            "github",
            Priority::P1,
            serde_json::to_value(&payload)?,
        )));
    }

    Ok(None)
}

/// Pull Request 이벤트 처리 (opened, labeled)
async fn handle_pull_request_event(payload: PullRequestPayload) -> Result<Option<Event>, AppError> {
    match payload.action.as_str() {
        "opened" | "labeled" => {
            // ai-review 라벨 확인
            if payload.pull_request.labels.iter().any(|l| l.name == "ai-review") {
                return Ok(Some(Event::new(
                    "github.pr_opened",
                    "github",
                    Priority::P2,
                    serde_json::to_value(&payload)?,
                )));
            }
        }
        _ => {}
    }
    Ok(None)
}
```

#### 페이로드 DTO 정의

```rust
/// Issues 이벤트 페이로드 (X-GitHub-Event: issues)
#[derive(Debug, Deserialize)]
pub struct IssuesPayload {
    pub action: String,
    pub issue: Issue,
    pub label: Option<Label>,
    pub repository: Repository,
    pub sender: User,
}

/// Issue Comment 이벤트 페이로드 (X-GitHub-Event: issue_comment)
#[derive(Debug, Deserialize)]
pub struct IssueCommentPayload {
    pub action: String,
    pub issue: Issue,
    pub comment: Comment,
    pub repository: Repository,
    pub sender: User,
}

/// Pull Request 이벤트 페이로드 (X-GitHub-Event: pull_request)
#[derive(Debug, Deserialize)]
pub struct PullRequestPayload {
    pub action: String,
    pub pull_request: PullRequest,
    pub label: Option<Label>,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Debug, Deserialize)]
pub struct Issue {
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub labels: Vec<Label>,
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub labels: Vec<Label>,
    pub head: GitRef,
    pub base: GitRef,
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub body: String,
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub struct Label {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct GitRef {
    pub r#ref: String,
    pub sha: String,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub full_name: String,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub login: String,
}
```

#### 페이로드 구조 예시

##### Issues 이벤트 (X-GitHub-Event: issues)

```json
{
  "action": "opened",
  "issue": {
    "number": 123,
    "title": "[Bug] AI 분석 API 타임아웃",
    "body": "AI5001 에러가 계속 발생합니다...",
    "labels": [
      {"name": "bug"},
      {"name": "ai-review"}
    ],
    "user": {
      "login": "developer"
    }
  },
  "repository": {
    "full_name": "org/repo"
  },
  "sender": {
    "login": "developer"
  }
}
```

##### Issue Comment 이벤트 (X-GitHub-Event: issue_comment)

```json
{
  "action": "created",
  "issue": {
    "number": 123,
    "title": "[Bug] AI 분석 API 타임아웃"
  },
  "comment": {
    "id": 456,
    "body": "@ai-bot 이 이슈 분석해줘",
    "user": {
      "login": "developer"
    }
  },
  "repository": {
    "full_name": "org/repo"
  },
  "sender": {
    "login": "developer"
  }
}
```

##### Pull Request 이벤트 (X-GitHub-Event: pull_request)

```json
{
  "action": "opened",
  "pull_request": {
    "number": 456,
    "title": "Fix: AI 타임아웃 이슈 해결",
    "body": "AI5001 에러를 수정합니다.",
    "labels": [
      {"name": "ai-review"}
    ],
    "head": {
      "ref": "fix/ai-timeout",
      "sha": "abc123"
    },
    "base": {
      "ref": "main",
      "sha": "def456"
    },
    "user": {
      "login": "developer"
    }
  },
  "repository": {
    "full_name": "org/repo"
  },
  "sender": {
    "login": "developer"
  }
}
```

#### 서명 검증

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

fn verify_github_signature(
    secret: &str,
    signature: &str,
    body: &[u8],
) -> Result<(), AppError> {
    let signature = signature
        .strip_prefix("sha256=")
        .ok_or(AppError::Unauthorized("Invalid signature format"))?;

    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|_| AppError::Internal("HMAC error"))?;
    mac.update(body);

    let expected = hex::encode(mac.finalize().into_bytes());

    if signature != expected {
        return Err(AppError::Unauthorized("Signature mismatch"));
    }

    Ok(())
}
```

---

## 3. 트리거 조건 정의

### 3.1 이벤트별 트리거 매트릭스

| 이벤트 소스 | 조건 | 트리거 액션 | 우선순위 |
|-------------|------|------------|----------|
| 모니터링 | `severity == critical` | AI 진단 + Issue 생성 | P0 |
| 모니터링 | `severity == warning` | Discord 알림만 | P1 |
| 모니터링 | `severity == info` | 로그만 | P2 |
| Discord | `@AI 분석해줘` 명령 | AI 진단 Agent | P1 |
| Discord | `@AI 수정해줘` 명령 | Auto-Fix Agent | P1 |
| GitHub | Issue + `ai-fix` 라벨 | Auto-Fix Agent | P1 |
| GitHub | PR + `ai-review` 라벨 | Code Review Agent | P2 |

### 3.2 필터링 로직

```rust
use std::str::FromStr;

/// 에러 심각도 (트리거 필터링에 사용)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info = 0,
    Warning = 1,
    Critical = 2,
}

impl FromStr for Severity {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(Severity::Info),
            "warning" => Ok(Severity::Warning),
            "critical" => Ok(Severity::Critical),
            _ => Err(()),
        }
    }
}

pub struct TriggerFilter {
    /// 활성화된 이벤트 타입
    enabled_events: HashSet<String>,
    /// 화이트리스트 사용자
    allowed_users: HashSet<String>,
    /// 블랙리스트 에러 코드 (무시할 코드)
    ignored_error_codes: HashSet<String>,
    /// 최소 심각도
    min_severity: Severity,
}

impl TriggerFilter {
    pub fn should_trigger(&self, event: &Event) -> bool {
        // 1. 이벤트 타입 확인
        if !self.enabled_events.contains(&event.event_type) {
            return false;
        }

        // 2. 사용자 권한 확인 (Discord, GitHub)
        // metadata.user에서 사용자 정보 읽기
        if let Some(user) = &event.metadata.user {
            if !self.allowed_users.is_empty() && !self.allowed_users.contains(user) {
                return false;
            }
        }

        // 3. 에러 코드 필터링 (모니터링)
        // data.error_code에서 에러 코드 읽기
        if let Some(error_code) = event.data.get("error_code").and_then(|v| v.as_str()) {
            if self.ignored_error_codes.contains(error_code) {
                return false;
            }
        }

        // 4. 심각도 확인
        // data.severity에서 심각도 읽기
        if let Some(severity_str) = event.data.get("severity").and_then(|v| v.as_str()) {
            if let Ok(severity) = Severity::from_str(severity_str) {
                if severity < self.min_severity {
                    return false;
                }
            }
        }

        true
    }
}
```

### 3.3 우선순위 및 큐 관리

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    P0 = 0,  // Critical - 즉시 처리
    P1 = 1,  // High - 5분 내 처리
    P2 = 2,  // Medium - 15분 내 처리
    P3 = 3,  // Low - 1시간 내 처리
}

impl Priority {
    pub fn from_event(event: &Event) -> Self {
        match event.event_type.as_str() {
            "monitoring.error_detected" => {
                match event.data.get("severity").and_then(|s| s.as_str()) {
                    Some("critical") => Priority::P0,
                    Some("warning") => Priority::P1,
                    _ => Priority::P2,
                }
            }
            "discord.command" => Priority::P1,
            "github.issue_labeled" => Priority::P1,
            "github.pr_opened" => Priority::P2,
            _ => Priority::P3,
        }
    }
}
```

---

## 4. 이벤트 큐 시스템 설계

### 4.1 아키텍처

```
┌─────────────────────────────────────────────────────────────────┐
│                      Event Queue System                          │
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │   Producer   │───▶│    Queue     │───▶│   Consumer   │       │
│  │  (Webhooks)  │    │   (Redis)    │    │   (Worker)   │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│                             │                    │               │
│                             ▼                    ▼               │
│                      ┌─────────────┐      ┌─────────────┐       │
│                      │  Dead Letter│      │   AI Agent  │       │
│                      │    Queue    │      │   Router    │       │
│                      └─────────────┘      └─────────────┘       │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 이벤트 스키마

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    /// 고유 이벤트 ID
    pub id: Uuid,
    /// 이벤트 타입 (e.g., "monitoring.error_detected")
    pub event_type: String,
    /// 이벤트 소스 (e.g., "log-watcher", "discord", "github")
    pub source: String,
    /// 이벤트 발생 시간 (UTC)
    pub timestamp: DateTime<Utc>,
    /// 우선순위
    pub priority: Priority,
    /// 이벤트 데이터 (소스별로 다름)
    /// - monitoring: { error_code, severity, message, target, request_id, log_line }
    /// - discord: { command, args, channel_id }
    /// - github: { action, issue_number, labels, repository }
    pub data: serde_json::Value,
    /// 메타데이터
    pub metadata: EventMetadata,
    /// 재시도 횟수
    pub retry_count: u32,
    /// 상태
    pub status: EventStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventMetadata {
    /// 중복 제거용 지문
    pub fingerprint: String,
    /// 상관 관계 ID (연관 이벤트 추적)
    pub correlation_id: Option<Uuid>,
    /// 사용자 정보
    pub user: Option<String>,
    /// 추가 속성
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Retrying,
}
```

### 4.3 큐 구현 (Redis 기반)

> **주의: Processing 큐 완료 처리 문제 해결**
>
> 기존 구현의 문제점:
> - `rpoplpush`로 processing 큐에 **전체 JSON 문자열**이 저장됨
> - `complete()`에서 `event_id.to_string()`만으로 `lrem` 시도
> - JSON 문자열 != event_id 문자열이므로 **매칭 실패**
>
> 해결 방법: **Hash를 별도로 사용하여 event_id와 JSON을 매핑**

```rust
use redis::{AsyncCommands, Client};

pub struct RedisEventQueue {
    client: Client,
    queue_key: String,
    processing_key: String,      // List: processing 중인 event_id 목록
    processing_data_key: String, // Hash: event_id -> JSON 매핑
    dlq_key: String,  // Dead Letter Queue
}

impl RedisEventQueue {
    pub fn new(client: Client, prefix: &str) -> Self {
        Self {
            client,
            queue_key: format!("{}:queue", prefix),
            processing_key: format!("{}:processing", prefix),
            processing_data_key: format!("{}:processing_data", prefix),
            dlq_key: format!("{}:dlq", prefix),
        }
    }

    pub async fn push(&self, event: Event) -> Result<(), AppError> {
        let mut conn = self.client.get_async_connection().await?;
        let event_json = serde_json::to_string(&event)?;

        // 우선순위에 따라 다른 큐에 추가
        let queue_key = format!("{}:p{}", self.queue_key, event.priority as u8);
        conn.lpush(&queue_key, event_json).await?;

        Ok(())
    }

    pub async fn pop(&self) -> Result<Option<Event>, AppError> {
        let mut conn = self.client.get_async_connection().await?;

        // 우선순위 순서대로 폴링 (P0 → P1 → P2 → P3)
        for priority in 0..=3 {
            let queue_key = format!("{}:p{}", self.queue_key, priority);

            // 큐에서 이벤트 가져오기 (rpop 사용 - rpoplpush 대신)
            if let Some(event_json) = conn
                .rpop::<_, Option<String>>(&queue_key, None)
                .await?
            {
                let event: Event = serde_json::from_str(&event_json)?;
                let event_id = event.id.to_string();

                // Processing 상태로 저장 (2단계):
                // 1. processing_key (List): event_id 추가 - 처리 순서 추적용
                // 2. processing_data_key (Hash): event_id -> JSON - complete/fail 시 조회용
                conn.lpush::<_, _, ()>(&self.processing_key, &event_id).await?;
                conn.hset::<_, _, _, ()>(&self.processing_data_key, &event_id, &event_json).await?;

                return Ok(Some(event));
            }
        }

        Ok(None)
    }

    pub async fn complete(&self, event_id: Uuid) -> Result<(), AppError> {
        let mut conn = self.client.get_async_connection().await?;
        let event_id_str = event_id.to_string();

        // Processing 상태에서 제거 (2단계):
        // 1. List에서 event_id 제거 (이제 event_id 문자열이므로 매칭 성공)
        // 2. Hash에서 event_id -> JSON 매핑 제거
        conn.lrem::<_, _, ()>(&self.processing_key, 1, &event_id_str).await?;
        conn.hdel::<_, _, ()>(&self.processing_data_key, &event_id_str).await?;

        Ok(())
    }

    pub async fn fail(&self, event: Event) -> Result<(), AppError> {
        let mut conn = self.client.get_async_connection().await?;
        let event_id_str = event.id.to_string();

        // 먼저 processing 상태에서 제거
        conn.lrem::<_, _, ()>(&self.processing_key, 1, &event_id_str).await?;
        conn.hdel::<_, _, ()>(&self.processing_data_key, &event_id_str).await?;

        if event.retry_count >= 3 {
            // Dead Letter Queue로 이동
            let event_json = serde_json::to_string(&event)?;
            conn.lpush(&self.dlq_key, event_json).await?;
        } else {
            // 재시도 큐에 추가
            let mut retry_event = event;
            retry_event.retry_count += 1;
            retry_event.status = EventStatus::Retrying;
            self.push(retry_event).await?;
        }

        Ok(())
    }

    /// Processing 중인 이벤트 복구 (서버 재시작 시 호출)
    /// 비정상 종료로 processing 상태에 남은 이벤트들을 다시 큐에 넣음
    pub async fn recover_processing(&self) -> Result<u32, AppError> {
        let mut conn = self.client.get_async_connection().await?;
        let mut recovered = 0;

        // Hash에서 모든 processing 이벤트 조회
        let processing_events: Vec<(String, String)> = conn
            .hgetall(&self.processing_data_key)
            .await?;

        for (event_id, event_json) in processing_events {
            // Processing 상태에서 제거
            conn.lrem::<_, _, ()>(&self.processing_key, 1, &event_id).await?;
            conn.hdel::<_, _, ()>(&self.processing_data_key, &event_id).await?;

            // 이벤트 파싱 후 재시도 처리
            if let Ok(mut event) = serde_json::from_str::<Event>(&event_json) {
                event.retry_count += 1;
                event.status = EventStatus::Retrying;
                self.push(event).await?;
                recovered += 1;
            }
        }

        Ok(recovered)
    }

    /// Processing 중인 이벤트 수 조회
    pub async fn processing_count(&self) -> Result<usize, AppError> {
        let mut conn = self.client.get_async_connection().await?;
        let count: usize = conn.hlen(&self.processing_data_key).await?;
        Ok(count)
    }
}
```

#### Redis 데이터 구조 요약

| Key | Type | 용도 |
|-----|------|------|
| `events:queue:p0` ~ `p3` | List | 우선순위별 대기 큐 (JSON 저장) |
| `events:processing` | List | Processing 중인 event_id 목록 |
| `events:processing_data` | Hash | event_id -> JSON 매핑 (완료 시 정확한 제거용) |
| `events:dlq` | List | Dead Letter Queue (JSON 저장) |

### 4.4 파일 기반 큐 (MVP)

Redis 없이 동작하는 경량 버전입니다.

```rust
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub struct FileEventQueue {
    queue_dir: PathBuf,
}

impl FileEventQueue {
    pub fn new(queue_dir: PathBuf) -> Result<Self, AppError> {
        fs::create_dir_all(&queue_dir)?;
        Ok(Self { queue_dir })
    }

    pub async fn push(&self, event: Event) -> Result<(), AppError> {
        let filename = format!("p{}_{}.json", event.priority as u8, event.id);
        let path = self.queue_dir.join("pending").join(&filename);

        fs::create_dir_all(path.parent().unwrap())?;

        let event_json = serde_json::to_string_pretty(&event)?;
        fs::write(&path, event_json)?;

        Ok(())
    }

    pub async fn pop(&self) -> Result<Option<Event>, AppError> {
        let pending_dir = self.queue_dir.join("pending");
        let processing_dir = self.queue_dir.join("processing");

        fs::create_dir_all(&processing_dir)?;

        // 우선순위 순서대로 파일 찾기
        for priority in 0..=3 {
            let prefix = format!("p{}_", priority);

            if let Ok(entries) = fs::read_dir(&pending_dir) {
                for entry in entries.flatten() {
                    let filename = entry.file_name().to_string_lossy().to_string();
                    if filename.starts_with(&prefix) {
                        // Processing 디렉토리로 이동
                        let new_path = processing_dir.join(&filename);
                        fs::rename(entry.path(), &new_path)?;

                        let content = fs::read_to_string(&new_path)?;
                        let event: Event = serde_json::from_str(&content)?;
                        return Ok(Some(event));
                    }
                }
            }
        }

        Ok(None)
    }

    pub async fn complete(&self, event_id: Uuid) -> Result<(), AppError> {
        let processing_dir = self.queue_dir.join("processing");
        let completed_dir = self.queue_dir.join("completed");

        fs::create_dir_all(&completed_dir)?;

        // 해당 이벤트 파일 찾기
        if let Ok(entries) = fs::read_dir(&processing_dir) {
            for entry in entries.flatten() {
                if entry.file_name().to_string_lossy().contains(&event_id.to_string()) {
                    let new_path = completed_dir.join(entry.file_name());
                    fs::rename(entry.path(), new_path)?;
                    break;
                }
            }
        }

        Ok(())
    }
}
```

---

## 5. 구현 체크리스트

### 5.1 Week 1: 이벤트 소스 구축

- [ ] **Discord Webhook 수신**
  - [ ] `/api/webhooks/discord` 엔드포인트 생성
  - [ ] Discord 인터랙션 검증 (Ping/Pong)
  - [ ] 명령어 파서 구현
  - [ ] Discord Bot 설정 및 권한 구성

- [ ] **모니터링 연동**
  - [ ] `log-watcher.sh` 수정: 이벤트 발행 추가
  - [ ] 이벤트 페이로드 표준화
  - [ ] 기존 Discord 알림과의 호환성 유지

- [ ] **GitHub Webhook 수신**
  - [ ] `/api/webhooks/github` 엔드포인트 생성
  - [ ] 서명 검증 구현
  - [ ] Issue/PR 이벤트 파싱

### 5.2 Week 2: 트리거 및 큐 시스템

- [ ] **트리거 엔진**
  - [ ] 필터링 로직 구현
  - [ ] 우선순위 분류 구현
  - [ ] 중복 제거 로직 (Fingerprint 기반)

- [ ] **이벤트 큐**
  - [ ] 파일 기반 큐 구현 (MVP)
  - [ ] Redis 큐 구현 (옵션)
  - [ ] Dead Letter Queue 처리
  - [ ] Worker 프로세스 구현

- [ ] **통합 테스트**
  - [ ] Discord 명령어 → 이벤트 생성 테스트
  - [ ] 모니터링 에러 → 이벤트 생성 테스트
  - [ ] GitHub Issue → 이벤트 생성 테스트
  - [ ] 우선순위 큐 동작 검증

---

## 6. 테스트 시나리오

### 6.1 Discord 웹훅 테스트

```bash
# 시나리오 1: AI 분석 명령어
curl -X POST http://localhost:8080/api/webhooks/discord \
  -H "Content-Type: application/json" \
  -d '{
    "type": 2,
    "token": "test_token",
    "channel_id": "123456789",
    "data": {
      "content": "@AI 분석해줘 AI5001 에러가 발생했어요",
      "author": {"id": "user1", "username": "developer"},
      "timestamp": "2025-01-31T14:23:45Z"
    }
  }'

# 예상 결과:
# - 이벤트 큐에 P1 우선순위로 추가
# - Discord 응답: "요청을 접수했습니다. 처리 중..."
```

### 6.2 모니터링 연동 테스트

```bash
# 시나리오 2: Critical 에러 발생
# logs/server.2025-01-31.log에 다음 로그 추가:
echo '{"timestamp":"2025-01-31T14:23:45Z","level":"ERROR","target":"server::domain::ai::service","fields":{"error_code":"AI5001","request_id":"req_abc123"},"message":"Claude API timeout"}' >> logs/server.$(date +%Y-%m-%d).log

# log-watcher.sh 실행
./scripts/log-watcher.sh

# 예상 결과:
# - 이벤트 큐에 P0 우선순위로 추가
# - Discord 알림 발송
# - AI 진단 Agent 트리거
```

### 6.3 GitHub 웹훅 테스트

> **중요**: 각 테스트에서 `X-GitHub-Event` 헤더 값이 이벤트 타입에 맞게 설정되어야 합니다.

#### 6.3.1 Issues 이벤트 테스트 (X-GitHub-Event: issues)

```bash
# 시나리오 3-1: Issue에 ai-fix 라벨 추가
curl -X POST http://localhost:8080/api/webhooks/github \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: issues" \
  -H "X-Hub-Signature-256: sha256=computed_signature" \
  -d '{
    "action": "labeled",
    "issue": {
      "number": 123,
      "title": "[Bug] API 타임아웃",
      "labels": [{"name": "bug"}, {"name": "ai-fix"}]
    },
    "label": {"name": "ai-fix"},
    "repository": {"full_name": "org/repo"}
  }'

# 예상 결과:
# - 이벤트 큐에 P1 우선순위로 추가
# - Auto-Fix Agent 트리거 대기
```

#### 6.3.2 Issue Comment 이벤트 테스트 (X-GitHub-Event: issue_comment)

```bash
# 시나리오 3-2: Issue에 @ai-bot 멘션 코멘트 추가
curl -X POST http://localhost:8080/api/webhooks/github \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: issue_comment" \
  -H "X-Hub-Signature-256: sha256=computed_signature" \
  -d '{
    "action": "created",
    "issue": {
      "number": 123,
      "title": "[Bug] API 타임아웃"
    },
    "comment": {
      "id": 456,
      "body": "@ai-bot 이 이슈 분석해줘",
      "user": {"login": "developer"}
    },
    "repository": {"full_name": "org/repo"},
    "sender": {"login": "developer"}
  }'

# 예상 결과:
# - 이벤트 큐에 P1 우선순위로 추가
# - 멘션 명령어 파싱 후 해당 Agent 실행
```

#### 6.3.3 Pull Request 이벤트 테스트 (X-GitHub-Event: pull_request)

```bash
# 시나리오 3-3: PR에 ai-review 라벨 추가
curl -X POST http://localhost:8080/api/webhooks/github \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: pull_request" \
  -H "X-Hub-Signature-256: sha256=computed_signature" \
  -d '{
    "action": "labeled",
    "pull_request": {
      "number": 456,
      "title": "Fix: AI 타임아웃 이슈 해결",
      "labels": [{"name": "ai-review"}],
      "head": {"ref": "fix/ai-timeout", "sha": "abc123"},
      "base": {"ref": "main", "sha": "def456"},
      "user": {"login": "developer"}
    },
    "label": {"name": "ai-review"},
    "repository": {"full_name": "org/repo"},
    "sender": {"login": "developer"}
  }'

# 예상 결과:
# - 이벤트 큐에 P2 우선순위로 추가
# - Code Review Agent 트리거 대기
```

#### 6.3.4 지원하지 않는 이벤트 테스트

```bash
# 시나리오 3-4: 지원하지 않는 이벤트 타입
curl -X POST http://localhost:8080/api/webhooks/github \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: push" \
  -H "X-Hub-Signature-256: sha256=computed_signature" \
  -d '{"ref": "refs/heads/main"}'

# 예상 결과:
# - 응답: {"status": "ignored", "reason": "Unsupported event type: push"}
# - 이벤트 큐에 추가되지 않음
```

### 6.4 우선순위 큐 테스트

```bash
# 시나리오 4: 다양한 우선순위 이벤트 동시 발생
# P2, P0, P1 순서로 이벤트 생성 후 처리 순서 확인

# 예상 결과: P0 → P1 → P2 순서로 처리
```

### 6.5 중복 제거 테스트

```bash
# 시나리오 5: 동일 에러 5분 내 재발생
# 같은 fingerprint의 에러 로그 2개 추가

# 예상 결과: 첫 번째 이벤트만 큐에 추가, 두 번째는 무시
```

---

## 7. 환경 변수

```bash
# .env 추가 항목

# Discord
DISCORD_WEBHOOK_URL=https://discord.com/api/webhooks/xxx/yyy
DISCORD_APPLICATION_ID=your_app_id
DISCORD_PUBLIC_KEY=your_public_key

# GitHub
GITHUB_WEBHOOK_SECRET=your_webhook_secret
GITHUB_TOKEN=ghp_xxx

# Event Queue
EVENT_QUEUE_TYPE=file  # or "redis"
EVENT_QUEUE_DIR=./data/events
REDIS_URL=redis://localhost:6379  # Redis 사용 시

# Trigger Settings
TRIGGER_MIN_SEVERITY=warning  # info, warning, critical
TRIGGER_DEDUP_WINDOW=300  # 5분
TRIGGER_RATE_LIMIT=10  # 시간당 최대 처리 수
```

---

## 8. 파일 구조 (구현 후)

```
27th-Web-Team-3-BE/
├── codes/server/src/
│   ├── domain/
│   │   └── webhook/                    # (신규)
│   │       ├── mod.rs
│   │       ├── discord_handler.rs      # Discord 웹훅 핸들러
│   │       ├── github_handler.rs       # GitHub 웹훅 핸들러
│   │       └── dto.rs                  # 페이로드 DTO
│   ├── event/                          # (신규)
│   │   ├── mod.rs
│   │   ├── event.rs                    # 이벤트 구조체
│   │   ├── queue.rs                    # 큐 인터페이스
│   │   ├── file_queue.rs               # 파일 기반 큐
│   │   ├── redis_queue.rs              # Redis 큐
│   │   ├── trigger.rs                  # 트리거 엔진
│   │   └── worker.rs                   # 이벤트 처리 워커
│   └── config/
│       └── event_config.rs             # 이벤트 설정
│
├── scripts/
│   ├── log-watcher.sh                  # (수정) 이벤트 발행 추가
│   └── event-worker.sh                 # (신규) 워커 프로세스
│
├── data/
│   └── events/                         # (신규) 파일 큐 저장소
│       ├── pending/
│       ├── processing/
│       ├── completed/
│       └── dlq/
│
└── docs/ai-automation/
    └── phase-1-event-trigger.md        # (현재 문서)
```

---

## 9. 다음 단계 (Phase 2)

Phase 1 완료 후, 다음 단계에서는 수신된 이벤트를 처리하는 AI Agent를 구현합니다:

- **Diagnostic Agent**: 에러 분석 및 근본 원인 진단
- **Auto-Fix Agent**: 자동 수정 PR 생성
- **Code Review Agent**: PR 코드 리뷰

```
Phase 1 (현재)          Phase 2               Phase 3
━━━━━━━━━━━━━━━         ━━━━━━━━━━━━━━━       ━━━━━━━━━━━━━━━
이벤트 수신/트리거  ───▶  AI Agent 구현   ───▶  통합 및 최적화
        │                      │                     │
        ▼                      ▼                     ▼
• 웹훅 수신             • Diagnostic Agent    • 성능 최적화
• 모니터링 연동         • Auto-Fix Agent      • 비용 최적화
• 이벤트 큐             • Code Review Agent   • 대시보드
```

---

## 관련 문서

- [AI 모니터링 개요](../ai-monitoring/README.md)
- [AI 모니터링 Phase 4 (자동화)](../ai-monitoring/phases/phase-4-automation.md)
- [알림 시스템 설계](../ai-monitoring/design/04-alerting.md)

---

#ai-automation #phase-1 #event-trigger #webhook
