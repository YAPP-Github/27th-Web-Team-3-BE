# 알림 시스템

## 개요

```
┌─────────────────────────────────────────────────────────────────┐
│                        Alert Manager                             │
│                                                                   │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │   Discord    │    │    GitHub    │    │   Dashboard  │       │
│  │   Webhook    │    │    Issue     │    │   (Grafana)  │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
└─────────────────────────────────────────────────────────────────┘
```

## Discord 알림

### Webhook 설정

1. Discord 서버 → 채널 설정 → 연동 → 웹후크 생성
2. Webhook URL 복사
3. 환경 변수 설정:
```bash
# .env
DISCORD_WEBHOOK_URL=https://discord.com/api/webhooks/xxx/yyy
DISCORD_ALERT_CHANNEL_ID=123456789  # Critical 알림용
DISCORD_LOG_CHANNEL_ID=987654321    # 일반 로그용
```

### 알림 포맷

#### Critical Alert

> **타임존 규칙**: 모든 타임스탬프는 UTC 기준으로 저장/전송됩니다.
> 사용자에게 표시할 때만 KST로 변환합니다 (UTC + 9시간).

```json
{
  "embeds": [{
    "title": "🚨 [Critical] Claude API Timeout",
    "color": 15158332,
    "fields": [
      {
        "name": "📍 위치",
        "value": "`src/domain/ai/service.rs:142`",
        "inline": true
      },
      {
        "name": "⏰ 발생 시간 (UTC)",
        "value": "2025-01-31T05:23:45Z",
        "inline": true
      },
      {
        "name": "📊 발생 횟수",
        "value": "지난 5분간 15회",
        "inline": true
      },
      {
        "name": "🔍 진단 결과",
        "value": "Claude API의 응답 시간이 30초를 초과하여 타임아웃 발생.\n최근 트래픽 증가로 인한 API rate limit 도달 가능성."
      },
      {
        "name": "💡 권장 조치",
        "value": "1. CloudWatch에서 API 호출 패턴 확인\n2. 재시도 로직 backoff 시간 증가 검토"
      }
    ],
    "footer": {
      "text": "AI Monitor | Error Code: AI_003"
    },
    "timestamp": "2025-01-31T05:23:45.000Z"
  }],
  "content": "@here 긴급 확인이 필요합니다"
}
```

#### Warning Summary (집계)
```json
{
  "embeds": [{
    "title": "⚠️ 지난 1시간 경고 요약",
    "color": 16776960,
    "description": "집계 기간: 05:00 - 06:00 UTC (14:00 - 15:00 KST)",
    "fields": [
      {
        "name": "응답 지연 (> 5초)",
        "value": "23건",
        "inline": true
      },
      {
        "name": "Rate Limit 경고",
        "value": "5건",
        "inline": true
      },
      {
        "name": "인증 실패",
        "value": "12건",
        "inline": true
      }
    ],
    "footer": {
      "text": "AI Monitor | Hourly Summary"
    }
  }]
}
```

#### Auto-Fix PR 알림
```json
{
  "embeds": [{
    "title": "🤖 자동 수정 PR 생성",
    "color": 3066993,
    "fields": [
      {
        "name": "PR 제목",
        "value": "[Auto-Fix] AI_003 타임아웃 값 조정"
      },
      {
        "name": "수정 내용",
        "value": "TIMEOUT_SECS: 30 → 45"
      },
      {
        "name": "PR 링크",
        "value": "[#123](https://github.com/org/repo/pull/123)"
      }
    ],
    "footer": {
      "text": "검토 후 머지해주세요"
    }
  }]
}
```

### 구현

```rust
// monitor/src/alerting/discord.rs
use reqwest::Client;
use serde_json::json;

pub struct DiscordAlerter {
    client: Client,
    webhook_url: String,
}

impl DiscordAlerter {
    pub async fn send_critical_alert(&self, report: &DiagnosticReport) -> Result<(), Error> {
        let embed = json!({
            "embeds": [{
                "title": format!("🚨 [Critical] {}", report.error_code),
                "color": 15158332,  // Red
                "fields": [
                    {
                        "name": "📍 위치",
                        "value": format!("`{}`", report.source_location),
                        "inline": true
                    },
                    {
                        "name": "⏰ 발생 시간 (UTC)",
                        // 내부 저장은 UTC, 표시는 UTC 사용 (필요시 클라이언트에서 KST 변환)
                        "value": report.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
                        "inline": true
                    },
                    {
                        "name": "🔍 근본 원인",
                        "value": &report.root_cause
                    },
                    {
                        "name": "💡 권장 조치",
                        "value": self.format_recommendations(&report.recommendations)
                    }
                ],
                "footer": {
                    "text": format!("AI Monitor | {}", report.error_code)
                },
                "timestamp": report.timestamp.to_rfc3339()
            }],
            "content": "@here 긴급 확인이 필요합니다"
        });

        self.client
            .post(&self.webhook_url)
            .json(&embed)
            .send()
            .await?;

        Ok(())
    }

    pub async fn send_warning_summary(&self, summary: &WarningSummary) -> Result<(), Error> {
        let embed = json!({
            "embeds": [{
                "title": "⚠️ 지난 1시간 경고 요약",
                "color": 16776960,  // Yellow
                "description": format!(
                    "집계 기간: {} - {}",
                    summary.start_time.format("%H:%M"),
                    summary.end_time.format("%H:%M")
                ),
                "fields": summary.categories.iter().map(|cat| {
                    json!({
                        "name": &cat.name,
                        "value": format!("{}건", cat.count),
                        "inline": true
                    })
                }).collect::<Vec<_>>()
            }]
        });

        self.client
            .post(&self.webhook_url)
            .json(&embed)
            .send()
            .await?;

        Ok(())
    }
}
```

## GitHub Issue 자동 생성

### 이슈 생성 조건
- Critical 에러 발생
- 새로운 에러 패턴 (이전에 없던 error_code)
- 동일 에러 반복 발생 (1시간 내 10회 이상)

### 이슈 템플릿

```markdown
## [AI 자동 생성] {error_code}: {short_description}

### 발생 정보
| 항목 | 값 |
|------|-----|
| 최초 발생 | {timestamp} |
| 발생 횟수 | {count}회 (최근 1시간) |
| 영향 API | `{affected_endpoint}` |
| 에러 코드 | `{error_code}` |

### 에러 로그
```json
{error_log_sample}
```

### AI 진단 결과

**심각도**: {severity}

**근본 원인**
{root_cause}

**영향 범위**
{impact}

### 권장 조치

{recommendations}

### 관련 파일
- `{source_file}:{line_number}`

### Labels
`bug`, `ai-generated`, `priority:{priority}`

---
이 이슈는 AI 모니터링 시스템에 의해 자동 생성되었습니다.
```

### 중복 체크

```rust
// monitor/src/alerting/github.rs
use octocrab::Octocrab;

pub struct GitHubIssueManager {
    client: Octocrab,
    owner: String,
    repo: String,
}

impl GitHubIssueManager {
    pub async fn create_issue_if_new(&self, report: &DiagnosticReport) -> Result<Option<Issue>, Error> {
        // 1. 기존 이슈 검색
        let search_query = format!(
            "repo:{}/{} is:issue label:ai-generated \"{}\" in:title",
            self.owner, self.repo, report.error_code
        );

        let existing = self.client
            .search()
            .issues_and_pull_requests(&search_query)
            .send()
            .await?;

        // 2. 열린 이슈가 있으면 코멘트만 추가
        if let Some(issue) = existing.items.iter().find(|i| i.state == "open") {
            self.add_occurrence_comment(issue.number, report).await?;
            return Ok(None);
        }

        // 3. 새 이슈 생성
        let issue = self.client
            .issues(&self.owner, &self.repo)
            .create(&self.build_issue_title(report))
            .body(&self.build_issue_body(report))
            .labels(&["bug", "ai-generated", &format!("priority:{}", report.severity)])
            .send()
            .await?;

        Ok(Some(issue))
    }

    async fn add_occurrence_comment(&self, issue_number: u64, report: &DiagnosticReport) -> Result<(), Error> {
        let comment = format!(
            "### 추가 발생 보고\n\n\
             - 시간: {}\n\
             - 발생 횟수: {}회 (이번 집계)\n\n\
             상세 로그는 모니터링 대시보드를 확인해주세요.",
            report.timestamp.format("%Y-%m-%d %H:%M:%S"),
            report.occurrence_count
        );

        self.client
            .issues(&self.owner, &self.repo)
            .create_comment(issue_number, &comment)
            .await?;

        Ok(())
    }
}
```

### gh CLI 사용 (Shell Script)

```bash
#!/bin/bash
# scripts/create-github-issue.sh

REPORT="$1"

ERROR_CODE=$(echo "$REPORT" | jq -r '.error_code')
ROOT_CAUSE=$(echo "$REPORT" | jq -r '.root_cause')
SEVERITY=$(echo "$REPORT" | jq -r '.severity')
IMPACT=$(echo "$REPORT" | jq -r '.impact')

# 중복 이슈 체크
EXISTING=$(gh issue list --label "ai-generated" --search "$ERROR_CODE" --state open --json number --jq '.[0].number')

if [ -n "$EXISTING" ]; then
    # 기존 이슈에 코멘트
    gh issue comment "$EXISTING" --body "### 추가 발생
- 시간: $(date '+%Y-%m-%d %H:%M:%S')
- 에러 코드: $ERROR_CODE"
    echo "Commented on existing issue #$EXISTING"
    exit 0
fi

# 새 이슈 생성
gh issue create \
    --title "[AI Monitor] $ERROR_CODE: $(echo "$ROOT_CAUSE" | head -c 50)" \
    --body "$(cat <<EOF
## 발생 정보
- 시간: $(date '+%Y-%m-%d %H:%M:%S')
- 에러 코드: \`$ERROR_CODE\`
- 심각도: $SEVERITY

## 근본 원인
$ROOT_CAUSE

## 영향 범위
$IMPACT

## 권장 조치
$(echo "$REPORT" | jq -r '.recommendations[] | "- \(.action)"')

---
이 이슈는 AI 모니터링 시스템에 의해 자동 생성되었습니다.
EOF
)" \
    --label "bug" \
    --label "ai-generated" \
    --label "priority:$SEVERITY"
```

## 알림 정책

### 심각도별 동작

| 심각도 | Discord | GitHub Issue | Auto-Fix |
|--------|---------|--------------|----------|
| Critical | 즉시 + @here | 자동 생성 | 시도 |
| Warning | 1시간 집계 | 조건부 생성 | 안함 |
| Info | 로그만 | 안함 | 안함 |

### 알림 제한 (Rate Limiting)

```rust
pub struct AlertRateLimiter {
    // 동일 에러에 대해 최소 5분 간격
    min_interval: Duration,
    // 1시간 내 최대 10회
    max_per_hour: u32,
    // 하루 최대 50회
    max_per_day: u32,
}

impl AlertRateLimiter {
    pub fn should_alert(&mut self, fingerprint: &str) -> bool {
        let now = Instant::now();

        // 최근 알림 시간 체크
        if let Some(last) = self.last_alert.get(fingerprint) {
            if now.duration_since(*last) < self.min_interval {
                return false;
            }
        }

        // 시간당 제한 체크
        let hourly_count = self.count_recent_alerts(Duration::from_secs(3600));
        if hourly_count >= self.max_per_hour {
            return false;
        }

        // 일일 제한 체크
        let daily_count = self.count_recent_alerts(Duration::from_secs(86400));
        if daily_count >= self.max_per_day {
            return false;
        }

        self.record_alert(fingerprint);
        true
    }
}
```

### 에스컬레이션

```
1단계: Discord 알림 (자동)
    │
    ├─── 30분 내 해결 안됨
    │
    ▼
2단계: GitHub Issue 생성 + 담당자 할당
    │
    ├─── 2시간 내 진행 없음
    │
    ▼
3단계: Discord DM + 이메일 (관리자)
```

## 대시보드 연동 (향후)

### Grafana 연동
- Loki 로그 쿼리
- 알림 규칙 시각화
- 에러 트렌드 차트

### Prometheus 메트릭
```rust
// 노출할 메트릭
- monitor_errors_total{error_code, severity}
- monitor_alerts_sent_total{channel, severity}
- monitor_diagnostic_duration_seconds
- monitor_auto_fix_attempts_total{result}
```
