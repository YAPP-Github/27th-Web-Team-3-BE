# Phase 2: 이슈 분석 및 브랜치 생성

## 개요

| 항목 | 내용 |
|------|------|
| Phase | 2: Issue Analysis & Branch Creation |
| 기간 | Week 3-4 |
| 목표 | 에러 로그 파싱, 컨텍스트 수집, AI 기반 이슈 분석, 자동 브랜치 생성 |
| 의존성 | Phase 1 (로그 기반) 완료 |

```
Phase 2 완료 상태
+-----------------------------------------------------------------+
|  [ ] 에러 파싱    [ ] 컨텍스트 수집    [ ] AI 분석    [ ] 브랜치 생성  |
+-----------------------------------------------------------------+
```

---

## 1. 목표 및 범위

### 1.1 목표

이 Phase에서는 다음을 구현합니다:

1. **에러 로그 파싱**: JSON 형식의 로그에서 에러 정보 추출
2. **컨텍스트 수집**: 관련 코드, 최근 변경사항, 연관 파일 수집
3. **AI 기반 이슈 분류**: Claude API를 활용한 에러 원인 분석 및 심각도 분류
4. **자동 브랜치 생성**: 분석 결과 기반 수정 브랜치 자동 생성

### 1.2 범위

**포함**:
- ERROR 레벨 로그 분석
- Rust 소스 코드 컨텍스트 수집
- Git 이력 기반 변경 추적
- 자동 브랜치 생성 및 푸시

**제외**:
- 자동 코드 수정 (Phase 4에서 구현)
- PR 생성 (Phase 4에서 구현)
- DEBUG/INFO 레벨 로그 분석

---

## 2. 이슈 분석 시스템

### 2.1 에러 로그 파싱

#### 로그 구조 (JSON)

프로젝트는 `tracing` 크레이트를 사용하여 JSON 형식 로그를 생성합니다.

```json
{
  "timestamp": "2025-01-31T14:23:45.123456Z",
  "level": "ERROR",
  "target": "server::domain::ai::service",
  "message": "Claude API request failed",
  "fields": {
    "request_id": "req_abc123",
    "error_code": "AI5003",
    "duration_ms": 30500,
    "retry_count": 3
  },
  "span": {
    "name": "process_retrospect_assistant",
    "request_id": "req_abc123",
    "user_id": "user_456"
  }
}
```

#### 파싱 구현

**파일**: `scripts/parse-error-log.sh`

```bash
#!/bin/bash
# scripts/parse-error-log.sh - 에러 로그 파싱

LOG_LINE="$1"

# 필수 필드 추출
TIMESTAMP=$(echo "$LOG_LINE" | jq -r '.timestamp')
LEVEL=$(echo "$LOG_LINE" | jq -r '.level')
TARGET=$(echo "$LOG_LINE" | jq -r '.target')
MESSAGE=$(echo "$LOG_LINE" | jq -r '.message')

# 구조화된 필드 추출
ERROR_CODE=$(echo "$LOG_LINE" | jq -r '.fields.error_code // "UNKNOWN"')
REQUEST_ID=$(echo "$LOG_LINE" | jq -r '.fields.request_id // .span.request_id // "N/A"')
DURATION_MS=$(echo "$LOG_LINE" | jq -r '.fields.duration_ms // "N/A"')
RETRY_COUNT=$(echo "$LOG_LINE" | jq -r '.fields.retry_count // 0')

# 결과 출력
cat << EOF
{
  "timestamp": "$TIMESTAMP",
  "level": "$LEVEL",
  "target": "$TARGET",
  "message": "$MESSAGE",
  "error_code": "$ERROR_CODE",
  "request_id": "$REQUEST_ID",
  "duration_ms": "$DURATION_MS",
  "retry_count": $RETRY_COUNT
}
EOF
```

#### 에러 코드 체계

> 에러 코드 표준에 대한 상세 내용은 [overview.md의 에러 코드 표준](./overview.md#8-에러-코드-표준)을 참조하세요.

| 도메인 | 접두어 | 범위 | 설명 |
|--------|--------|------|------|
| AI | `AI5xxx` | AI5001-AI5099 | Claude/OpenAI API 관련 |
| Auth | `AUTH4xxx` | AUTH4001-AUTH4099 | 인증/인가 관련 |
| Database | `DB5xxx` | DB5001-DB5099 | 데이터베이스 관련 |
| Validation | `VAL4xxx` | VAL4001-VAL4099 | 입력 검증 관련 |
| External | `EXT5xxx` | EXT5001-EXT5099 | 외부 서비스 관련 |

### 2.2 컨텍스트 수집

#### 관련 코드 수집

**파일**: `scripts/collect-context.py`

```python
#!/usr/bin/env python3
"""
Context Collector - 에러 관련 컨텍스트 수집
"""

import os
import subprocess
import json
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent
SERVER_DIR = PROJECT_ROOT / "codes" / "server"


def target_to_path(target: str) -> Path:
    """
    target 경로를 파일 경로로 변환
    server::domain::ai::service -> codes/server/src/domain/ai/service.rs
    """
    # server:: 제거 후 :: -> /로 변환
    relative = target.replace("server::", "").replace("::", "/")
    return SERVER_DIR / "src" / f"{relative}.rs"


def collect_source_code(target: str, line_range: int = 50) -> dict:
    """관련 소스 코드 수집"""
    path = target_to_path(target)

    result = {
        "file_path": str(path),
        "exists": path.exists(),
        "content": None,
        "line_count": 0
    }

    if path.exists():
        with open(path, encoding="utf-8") as f:
            content = f.read()
            lines = content.split('\n')
            result["content"] = content[:5000]  # 토큰 제한
            result["line_count"] = len(lines)

    return result


def collect_related_files(target: str) -> list:
    """
    관련 파일 목록 수집
    - 같은 도메인의 다른 파일들 (handler, dto, client 등)
    - mod.rs
    """
    path = target_to_path(target)
    if not path.exists():
        return []

    parent = path.parent
    related = []

    for file in parent.glob("*.rs"):
        if file != path:
            related.append({
                "path": str(file),
                "name": file.name
            })

    return related


def collect_git_history(target: str, limit: int = 5) -> list:
    """최근 Git 커밋 이력"""
    path = target_to_path(target)
    relative_path = path.relative_to(PROJECT_ROOT)

    try:
        result = subprocess.run(
            ["git", "log", f"-{limit}", "--format=%H|%s|%an|%ar", "--", str(relative_path)],
            capture_output=True,
            text=True,
            cwd=PROJECT_ROOT,
            timeout=10
        )

        commits = []
        for line in result.stdout.strip().split('\n'):
            if '|' in line:
                parts = line.split('|')
                commits.append({
                    "hash": parts[0][:8],
                    "message": parts[1],
                    "author": parts[2],
                    "relative_time": parts[3]
                })

        return commits
    except Exception as e:
        return [{"error": str(e)}]


def collect_git_diff(target: str) -> str:
    """파일의 최근 변경사항 (diff)"""
    path = target_to_path(target)
    relative_path = path.relative_to(PROJECT_ROOT)

    try:
        # 마지막 커밋과의 diff
        result = subprocess.run(
            ["git", "diff", "HEAD~1", "--", str(relative_path)],
            capture_output=True,
            text=True,
            cwd=PROJECT_ROOT,
            timeout=10
        )

        diff = result.stdout.strip()
        return diff[:2000] if diff else "(변경사항 없음)"
    except Exception:
        return "(diff 수집 실패)"


def collect_all_context(error_log: dict) -> dict:
    """전체 컨텍스트 수집"""
    target = error_log.get("target", "")

    return {
        "error": error_log,
        "source": collect_source_code(target),
        "related_files": collect_related_files(target),
        "git_history": collect_git_history(target),
        "recent_diff": collect_git_diff(target)
    }


if __name__ == "__main__":
    import sys

    if len(sys.argv) < 2:
        print(json.dumps({"error": "Usage: collect-context.py '<json_log>'"}))
        sys.exit(1)

    try:
        error_log = json.loads(sys.argv[1])
        context = collect_all_context(error_log)
        print(json.dumps(context, ensure_ascii=False, indent=2))
    except json.JSONDecodeError as e:
        print(json.dumps({"error": f"Invalid JSON: {e}"}))
        sys.exit(1)
```

#### 컨텍스트 출력 예시

```json
{
  "error": {
    "timestamp": "2025-01-31T14:23:45Z",
    "error_code": "AI5003",
    "target": "server::domain::ai::service",
    "message": "Claude API timeout"
  },
  "source": {
    "file_path": "codes/server/src/domain/ai/service.rs",
    "exists": true,
    "content": "pub struct AiService { ... }",
    "line_count": 245
  },
  "related_files": [
    {"path": "codes/server/src/domain/ai/handler.rs", "name": "handler.rs"},
    {"path": "codes/server/src/domain/ai/dto.rs", "name": "dto.rs"},
    {"path": "codes/server/src/domain/ai/client.rs", "name": "client.rs"}
  ],
  "git_history": [
    {"hash": "abc123", "message": "feat: Add retry logic", "author": "dev", "relative_time": "2 days ago"},
    {"hash": "def456", "message": "fix: Increase timeout", "author": "dev", "relative_time": "1 week ago"}
  ],
  "recent_diff": "@@ -45,7 +45,8 @@ ..."
}
```

### 2.3 AI를 활용한 이슈 분류 및 원인 분석

#### Diagnostic Agent

**파일**: `scripts/issue-analyzer.py`

```python
#!/usr/bin/env python3
"""
Issue Analyzer - AI 기반 이슈 분석 및 분류
"""

import os
import sys
import json
import re
from anthropic import Anthropic

client = Anthropic()

# 심각도 분류 기준
SEVERITY_CRITERIA = """
## 심각도 분류 기준

### Critical (즉시 대응 필요)
- 서비스 전체 중단
- 데이터 손실 위험
- 보안 취약점
- 인증 시스템 장애

### High (당일 대응)
- 주요 기능 장애
- 성능 심각한 저하 (응답 시간 10배 이상)
- 특정 사용자 그룹 영향

### Medium (이번 스프린트 내 대응)
- 부분 기능 장애
- 간헐적 에러 발생
- 성능 저하 (응답 시간 2-10배)

### Low (백로그)
- UI/UX 개선 필요
- 마이너한 버그
- 문서화 필요
"""

# 에러 코드별 예상 원인
ERROR_CODE_HINTS = {
    "AI5001": "API 키 인증 실패 - 환경 변수 또는 키 만료 확인",
    "AI5002": "잘못된 프롬프트 - 입력 검증 로직 확인",
    "AI5003": "API 타임아웃 - 타임아웃 설정 또는 네트워크 확인",
    "AI5004": "Rate limit 초과 - 호출 빈도 또는 쿼터 확인",
    "AI5005": "API 내부 오류 - 외부 서비스 상태 확인",
    "AUTH4001": "토큰 없음 - 클라이언트 인증 흐름 확인",
    "AUTH4002": "토큰 만료 - 토큰 갱신 로직 확인",
    "AUTH4003": "토큰 변조 - 보안 검토 필요",
    "DB5001": "연결 실패 - 데이터베이스 상태 확인",
    "DB5002": "쿼리 타임아웃 - 쿼리 최적화 필요",
}


def analyze_issue(context: dict) -> dict:
    """Claude API로 이슈 분석"""

    error = context.get("error", {})
    source = context.get("source", {})
    git_history = context.get("git_history", [])

    error_code = error.get("error_code", "UNKNOWN")
    error_hint = ERROR_CODE_HINTS.get(error_code, "알려진 패턴 없음")

    # Git 이력 포맷
    git_history_text = "\n".join([
        f"- {c.get('hash', 'N/A')}: {c.get('message', 'N/A')} ({c.get('relative_time', 'N/A')})"
        for c in git_history[:5]
    ]) or "최근 변경 이력 없음"

    prompt = f"""# 역할
당신은 Rust 백엔드 시스템의 에러 진단 전문가입니다.
다음 에러를 분석하고 이슈 분류 결과를 제공하세요.

{SEVERITY_CRITERIA}

# 에러 정보
- **에러 코드**: {error_code}
- **힌트**: {error_hint}
- **위치**: {error.get('target', 'unknown')}
- **메시지**: {error.get('message', 'N/A')}
- **요청 ID**: {error.get('request_id', 'N/A')}
- **소요 시간**: {error.get('duration_ms', 'N/A')}ms

# 관련 소스 코드
```rust
{source.get('content', '(소스 없음)')[:3000]}
```

# 최근 Git 커밋
```
{git_history_text}
```

# 최근 변경사항
```diff
{context.get('recent_diff', '(없음)')[:1500]}
```

# 요청
다음 JSON 형식으로 분석 결과를 제공하세요:

```json
{{
  "severity": "critical|high|medium|low",
  "category": "api|auth|database|validation|configuration|external",
  "root_cause": "근본 원인 (1-2문장)",
  "impact": "영향 범위",
  "affected_users": "all|partial|none",
  "related_to_recent_change": true|false,
  "suspected_commit": "의심되는 커밋 해시 또는 null",
  "recommendations": [
    {{"priority": 1, "action": "권장 조치", "effort": "low|medium|high"}}
  ],
  "auto_fixable": true|false,
  "fix_type": "config|code|dependency|manual",
  "fix_suggestion": "자동 수정 가능한 경우 구체적 변경 내용",
  "branch_name_suggestion": "fix/에러코드-간략설명"
}}
```

JSON만 출력하세요."""

    try:
        model = os.environ.get("DIAGNOSTIC_MODEL", "claude-sonnet-4-20250514")
        response = client.messages.create(
            model=model,
            max_tokens=1500,
            messages=[{"role": "user", "content": prompt}]
        )

        content = response.content[0].text

        # JSON 추출
        json_match = re.search(r'\{[\s\S]*\}', content)
        if json_match:
            result = json.loads(json_match.group())
            result["analysis_model"] = model
            return result

        return {"error": "JSON 파싱 실패", "raw": content[:500]}

    except Exception as e:
        return {"error": str(e)}


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(json.dumps({"error": "Usage: issue-analyzer.py '<context_json>'"}))
        sys.exit(1)

    try:
        context = json.loads(sys.argv[1])
        result = analyze_issue(context)
        print(json.dumps(result, ensure_ascii=False, indent=2))
    except json.JSONDecodeError as e:
        print(json.dumps({"error": f"Invalid JSON: {e}"}))
        sys.exit(1)
```

#### 분석 결과 스키마

```json
{
  "severity": "high",
  "category": "api",
  "root_cause": "Claude API 호출 시 30초 타임아웃이 설정되어 있으나 프롬프트 길이 초과로 응답 지연 발생",
  "impact": "AI 회고 어시스턴트 기능 전체 사용 불가",
  "affected_users": "partial",
  "related_to_recent_change": true,
  "suspected_commit": "abc123",
  "recommendations": [
    {"priority": 1, "action": "타임아웃 값을 30초에서 60초로 증가", "effort": "low"},
    {"priority": 2, "action": "프롬프트 길이 제한 추가", "effort": "medium"},
    {"priority": 3, "action": "청크 단위 처리 구현", "effort": "high"}
  ],
  "auto_fixable": true,
  "fix_type": "config",
  "fix_suggestion": "src/domain/ai/client.rs의 TIMEOUT_SECS를 30에서 60으로 변경",
  "branch_name_suggestion": "fix/ai5003-increase-timeout",
  "analysis_model": "claude-sonnet-4-20250514"
}
```

---

## 3. 브랜치 생성 전략

### 3.1 네이밍 컨벤션

#### 브랜치 이름 형식

```
{type}/{error_code}-{brief-description}
```

#### 타입별 접두어

| 타입 | 사용 시점 | 예시 |
|------|----------|------|
| `fix/` | 버그 수정 | `fix/ai5003-timeout-increase` |
| `hotfix/` | 긴급 수정 (critical) | `hotfix/auth4003-token-validation` |
| `config/` | 설정 변경 | `config/db5002-connection-pool` |
| `refactor/` | 구조 개선 | `refactor/ai-error-handling` |

#### 네이밍 규칙

1. **소문자 사용**: 타입, 설명, 에러 코드 모두 소문자로 통일
2. **하이픈 구분**: 단어는 하이픈(`-`)으로 구분
3. **간결한 설명**: 20자 이내 영문 설명
4. **에러 코드 포함**: 추적 가능성 확보 (소문자로 변환하여 사용, 예: `AI5003` → `ai5003`)

#### 예시

```bash
# 좋은 예
fix/ai5003-increase-timeout
hotfix/auth4002-token-refresh
config/db5001-pool-size

# 나쁜 예
Fix/AI5003_increase_timeout    # 타입이 대문자, 언더스코어 사용
ai-003-fix                     # 타입 없음, 에러코드 형식 불일치
fix/very-long-branch-name-that-describes-everything-in-detail  # 너무 김
```

### 3.2 기반 브랜치 선택 로직

#### 심각도별 기반 브랜치

| 심각도 | 기반 브랜치 | 이유 |
|--------|------------|------|
| Critical | `dev` | 최신 코드에서 즉시 수정 |
| High | `dev` | 최신 코드에서 수정 |
| Medium | `dev` | 일반 개발 플로우 |
| Low | `dev` | 일반 개발 플로우 |

#### 브랜치 선택 스크립트

```bash
#!/bin/bash
# get-base-branch.sh

SEVERITY="$1"

case "$SEVERITY" in
    critical|high|medium|low)
        echo "dev"
        ;;
    *)
        echo "dev"
        ;;
esac
```

> **참고**: 현재 프로젝트는 `dev` 브랜치를 메인 개발 브랜치로 사용합니다.
> `main` 또는 `master` 브랜치는 별도로 관리하지 않습니다.

### 3.3 브랜치 생성 구현

**파일**: `scripts/create-fix-branch.sh`

```bash
#!/bin/bash
# scripts/create-fix-branch.sh - 수정 브랜치 자동 생성

set -e

ANALYSIS="$1"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# 분석 결과 파싱
SEVERITY=$(echo "$ANALYSIS" | jq -r '.severity // "medium"')
ERROR_CODE=$(echo "$ANALYSIS" | jq -r '.error_code // "UNKNOWN"')
BRANCH_SUGGESTION=$(echo "$ANALYSIS" | jq -r '.branch_name_suggestion // ""')
ROOT_CAUSE=$(echo "$ANALYSIS" | jq -r '.root_cause // ""')
FIX_TYPE=$(echo "$ANALYSIS" | jq -r '.fix_type // "manual"')

# 브랜치 이름 결정
if [ -n "$BRANCH_SUGGESTION" ] && [ "$BRANCH_SUGGESTION" != "null" ]; then
    BRANCH_NAME="$BRANCH_SUGGESTION"
else
    # 기본 브랜치 이름 생성
    TIMESTAMP=$(date +%Y%m%d-%H%M)
    case "$SEVERITY" in
        critical) PREFIX="hotfix" ;;
        *)        PREFIX="fix" ;;
    esac
    BRANCH_NAME="${PREFIX}/${ERROR_CODE}-${TIMESTAMP}"
fi

# 기반 브랜치 선택
BASE_BRANCH="dev"

echo "=== Branch Creation ==="
echo "Severity: $SEVERITY"
echo "Error Code: $ERROR_CODE"
echo "Branch Name: $BRANCH_NAME"
echo "Base Branch: $BASE_BRANCH"

# Git 작업
cd "$PROJECT_DIR"

# 현재 브랜치 저장
CURRENT_BRANCH=$(git branch --show-current)

# 최신 dev 가져오기
git fetch origin "$BASE_BRANCH"
git checkout "$BASE_BRANCH"
git pull origin "$BASE_BRANCH"

# 브랜치 존재 여부 확인
if git show-ref --verify --quiet "refs/heads/$BRANCH_NAME"; then
    echo "Branch '$BRANCH_NAME' already exists locally"
    git checkout "$BRANCH_NAME"
elif git show-ref --verify --quiet "refs/remotes/origin/$BRANCH_NAME"; then
    echo "Branch '$BRANCH_NAME' exists on remote, checking out"
    git checkout -b "$BRANCH_NAME" "origin/$BRANCH_NAME"
else
    # 새 브랜치 생성
    git checkout -b "$BRANCH_NAME"
    echo "Created new branch: $BRANCH_NAME"
fi

# 브랜치 정보 파일 생성 (옵션)
BRANCH_INFO_FILE=".branch-info.json"
cat > "$BRANCH_INFO_FILE" << EOF
{
  "branch_name": "$BRANCH_NAME",
  "base_branch": "$BASE_BRANCH",
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "error_code": "$ERROR_CODE",
  "severity": "$SEVERITY",
  "fix_type": "$FIX_TYPE",
  "root_cause": "$ROOT_CAUSE"
}
EOF

# 결과 출력
cat << EOF
{
  "success": true,
  "branch_name": "$BRANCH_NAME",
  "base_branch": "$BASE_BRANCH",
  "ready_for_fix": true
}
EOF
```

---

## 4. Git 자동화 워크플로우

### 4.1 전체 파이프라인

```
에러 발생
    |
    v
[1. 로그 파싱] -----> 구조화된 에러 정보 추출
    |
    v
[2. 컨텍스트 수집] -> 소스 코드, Git 이력 수집
    |
    v
[3. AI 분석] -------> 원인 분석, 심각도 분류
    |
    v
[4. 브랜치 생성] ---> 수정용 브랜치 자동 생성
    |
    v
[5. 알림 발송] -----> Discord 알림 + GitHub Issue
```

### 4.2 통합 스크립트

**파일**: `scripts/analyze-and-branch.sh`

```bash
#!/bin/bash
# scripts/analyze-and-branch.sh - 이슈 분석 및 브랜치 생성 통합 스크립트

set -e

LOG_LINE="$1"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

echo "=== Phase 2: Issue Analysis & Branch Creation ==="
echo "Timestamp: $(date)"

# 1. 에러 파싱
echo "[Step 1] Parsing error log..."
PARSED_ERROR=$(./scripts/parse-error-log.sh "$LOG_LINE")
ERROR_CODE=$(echo "$PARSED_ERROR" | jq -r '.error_code')
echo "Error Code: $ERROR_CODE"

# 2. 컨텍스트 수집
echo "[Step 2] Collecting context..."
CONTEXT=$(python3 ./scripts/collect-context.py "$LOG_LINE")

# 3. AI 분석
echo "[Step 3] Running AI analysis..."
ANALYSIS=$(python3 ./scripts/issue-analyzer.py "$CONTEXT")

SEVERITY=$(echo "$ANALYSIS" | jq -r '.severity // "medium"')
ROOT_CAUSE=$(echo "$ANALYSIS" | jq -r '.root_cause // "분석 필요"')
AUTO_FIXABLE=$(echo "$ANALYSIS" | jq -r '.auto_fixable // false')

echo "Severity: $SEVERITY"
echo "Root Cause: $ROOT_CAUSE"
echo "Auto Fixable: $AUTO_FIXABLE"

# 4. 심각도에 따른 브랜치 생성 결정
if [ "$SEVERITY" = "critical" ] || [ "$SEVERITY" = "high" ]; then
    echo "[Step 4] Creating fix branch..."

    # 분석 결과에 에러 코드 추가
    ANALYSIS_WITH_CODE=$(echo "$ANALYSIS" | jq --arg ec "$ERROR_CODE" '. + {error_code: $ec}')

    BRANCH_RESULT=$(./scripts/create-fix-branch.sh "$ANALYSIS_WITH_CODE")
    BRANCH_NAME=$(echo "$BRANCH_RESULT" | jq -r '.branch_name')

    echo "Branch created: $BRANCH_NAME"

    # Discord 알림에 브랜치 정보 포함
    ./scripts/discord-alert.sh "$SEVERITY" \
        "🔍 [$ERROR_CODE] 이슈 분석 완료" \
        "**심각도**: $SEVERITY\n**근본 원인**: $ROOT_CAUSE\n**수정 브랜치**: \`$BRANCH_NAME\`\n**자동 수정 가능**: $AUTO_FIXABLE" \
        "$ERROR_CODE"
else
    echo "[Step 4] Skipping branch creation (severity: $SEVERITY)"

    # Discord 알림만 발송
    ./scripts/discord-alert.sh "$SEVERITY" \
        "🔍 [$ERROR_CODE] 이슈 분석 완료" \
        "**심각도**: $SEVERITY\n**근본 원인**: $ROOT_CAUSE\n**자동 수정 가능**: $AUTO_FIXABLE" \
        "$ERROR_CODE"
fi

# 5. 결과 출력
echo "=== Analysis Complete ==="
echo "$ANALYSIS" | jq '.'
```

### 4.3 Cron 통합

```bash
# crontab 설정 (5분마다)
*/5 * * * * cd /path/to/project && ./scripts/log-watcher.sh >> logs/watcher.log 2>&1
```

**log-watcher.sh와 연동**:

```bash
# scripts/log-watcher.sh 내에서 호출
if [ "$LEVEL" = "ERROR" ]; then
    # 기존 중복 체크 후...
    ./scripts/analyze-and-branch.sh "$line"
fi
```

---

## 5. 구현 체크리스트

### Phase 2.1: 에러 파싱 (Week 3 전반)

- [ ] `scripts/parse-error-log.sh` 생성
- [ ] JSON 로그 파싱 테스트
- [ ] 에러 코드 체계 정리

### Phase 2.2: 컨텍스트 수집 (Week 3 후반)

- [ ] `scripts/collect-context.py` 생성
- [ ] 소스 코드 매핑 테스트
- [ ] Git 이력 수집 테스트
- [ ] 관련 파일 탐색 테스트

### Phase 2.3: AI 분석 (Week 4 전반)

- [ ] `scripts/issue-analyzer.py` 생성
- [ ] `ANTHROPIC_API_KEY` 환경 변수 설정
- [ ] 분석 프롬프트 튜닝
- [ ] 분석 결과 스키마 검증

### Phase 2.4: 브랜치 자동화 (Week 4 후반)

- [ ] `scripts/create-fix-branch.sh` 생성
- [ ] 브랜치 네이밍 규칙 적용
- [ ] `scripts/analyze-and-branch.sh` 통합 스크립트 생성
- [ ] Log Watcher 연동

### Phase 2.5: 문서화 및 테스트

- [ ] 사용 가이드 작성
- [ ] 엔드투엔드 테스트 수행
- [ ] 팀 리뷰

---

## 6. 테스트 시나리오

### 6.1 단위 테스트

#### 에러 파싱 테스트

```bash
# 테스트 1: 정상 에러 로그 파싱
./scripts/parse-error-log.sh '{"level":"ERROR","target":"server::domain::ai::service","fields":{"error_code":"AI5003"},"message":"timeout"}'

# 예상 결과
# {
#   "level": "ERROR",
#   "error_code": "AI5003",
#   ...
# }
```

#### 컨텍스트 수집 테스트

```bash
# 테스트 2: 소스 코드 수집
python3 ./scripts/collect-context.py '{"target":"server::domain::ai::service","fields":{"error_code":"AI5003"}}'

# 예상 결과
# - source.exists: true
# - source.content: 실제 코드
# - git_history: 최근 커밋 목록
```

#### AI 분석 테스트

```bash
# 테스트 3: AI 분석 (API 호출)
export ANTHROPIC_API_KEY="sk-ant-xxx"
python3 ./scripts/issue-analyzer.py '{"error":{"error_code":"AI5003","target":"server::domain::ai::service"},"source":{"content":"pub fn call_api()..."}}'

# 예상 결과
# - severity: high 또는 critical
# - root_cause: 구체적인 원인
# - recommendations: 최소 1개 이상
```

### 6.2 통합 테스트

#### 전체 파이프라인 테스트

```bash
# 테스트 4: 전체 플로우
./scripts/analyze-and-branch.sh '{"level":"ERROR","target":"server::domain::ai::service","fields":{"error_code":"AI5003","duration_ms":35000},"message":"Claude API timeout after 30000ms"}'

# 예상 결과
# 1. 에러 파싱 완료
# 2. 컨텍스트 수집 완료
# 3. AI 분석 완료 (severity: high)
# 4. 브랜치 생성: fix/ai5003-increase-timeout
# 5. Discord 알림 발송
```

### 6.3 시나리오별 테스트

| 시나리오 | 입력 | 예상 결과 |
|---------|------|----------|
| Critical 에러 | `AI5001` (인증 실패) | `hotfix/` 브랜치 생성 |
| High 에러 | `AI5003` (타임아웃) | `fix/` 브랜치 생성 |
| Medium 에러 | `VAL4001` (검증 실패) | 브랜치 생성 안 함, 알림만 |
| Low 에러 | `DB5004` (데이터 없음) | 브랜치 생성 안 함, 알림만 |
| 파일 없음 | 존재하지 않는 target | 컨텍스트 없이 분석 진행 |
| API 오류 | ANTHROPIC_API_KEY 없음 | 기본 알림만 발송 |

### 6.4 롤백 테스트

```bash
# 테스트: 브랜치 생성 실패 시 롤백
# 1. 의도적으로 잘못된 브랜치 이름 사용
# 2. Git 에러 발생 확인
# 3. 원래 브랜치로 복귀 확인
```

---

## 참고 문서

- [Phase 1: Event Trigger](./phase-1-event-trigger.md)
- [Phase 3: AI Diagnostic](./phase-3-ai-diagnostic.md)
- [Phase 4: Issue Automation](./phase-4-issue-automation.md)
- [Phase 5: Auto-Fix & PR](./phase-5-auto-fix-pr.md)
- [Overview](./overview.md)

---

#phase-2 #issue-analysis #branch-automation #ai-diagnostic
