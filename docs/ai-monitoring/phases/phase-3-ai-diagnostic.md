# Phase 3 (AI): AI 진단 연동

## 개요

| 항목 | 내용 |
|------|------|
| Phase | 3: AI |
| 기간 | Week 5-6 |
| 목표 | Claude API 진단, 컨텍스트 수집, 구조화된 보고서 |
| 의존성 | Phase 2 (MVP) 완료 |

```
Phase 3 완료 상태
┌─────────────────────────────────────────────────────────────┐
│  ✅ Diagnostic Agent    ✅ 컨텍스트 수집    ✅ 진단 보고서  │
└─────────────────────────────────────────────────────────────┘
```

## 완료 조건

- [ ] 에러 발생 시 Claude API로 자동 진단
- [ ] 소스 코드 + git 이력 컨텍스트 포함
- [ ] Discord 알림에 근본 원인 포함

---

## 사전 조건

### 환경 설정
```bash
# .env에 추가
ANTHROPIC_API_KEY=sk-ant-xxx
# 선택: 모델명 커스터마이징 (기본값: claude-sonnet-4-20250514)
DIAGNOSTIC_MODEL=claude-sonnet-4-20250514
```

### Python 의존성
```bash
pip install anthropic
```

---

## 태스크 3.1: Diagnostic Agent

### 구현

**파일**: `scripts/diagnostic-agent.py`

```python
#!/usr/bin/env python3
"""
AI Diagnostic Agent - 에러 로그 분석 및 진단
"""

import os
import sys
import json
import subprocess
import re
from anthropic import Anthropic

client = Anthropic()

def collect_source_context(target: str) -> str:
    """target에서 소스 파일 추출하고 읽기"""
    # server::domain::ai::service → src/domain/ai/service.rs
    path = target.replace("server::", "src/").replace("::", "/") + ".rs"
    full_path = f"codes/server/{path}"

    if os.path.exists(full_path):
        with open(full_path, encoding="utf-8") as f:
            content = f.read()
            # 길이 제한 (토큰 절약)
            return content[:3000] if len(content) > 3000 else content
    return "(소스 파일을 찾을 수 없음)"


def collect_git_context(target: str) -> str:
    """최근 커밋 이력"""
    path = target.replace("server::", "src/").replace("::", "/") + ".rs"

    try:
        result = subprocess.run(
            ["git", "log", "--oneline", "-5", "--", path],
            capture_output=True,
            text=True,
            cwd="codes/server",
            timeout=10
        )
        return result.stdout.strip() or "(최근 커밋 없음)"
    except Exception:
        return "(git 정보 수집 실패)"


def diagnose(error_log: dict) -> dict:
    """Claude API로 에러 진단"""
    target = error_log.get("target", "unknown")
    error_code = error_log.get("fields", {}).get("error_code", "UNKNOWN")
    message = error_log.get("message", "")

    source = collect_source_context(target)
    git_log = collect_git_context(target)

    prompt = f"""# 역할
당신은 Rust 백엔드 시스템의 에러 진단 전문가입니다.

# 에러 정보
- **에러 코드**: {error_code}
- **위치**: {target}
- **메시지**: {message}

# 관련 소스 코드
```rust
{source}
```

# 최근 커밋
```
{git_log}
```

# 요청
다음 JSON 형식으로 진단 결과를 제공하세요:

```json
{{
  "severity": "critical|warning|info",
  "root_cause": "근본 원인 (1-2문장)",
  "impact": "영향 범위",
  "recommendations": [
    {{"priority": 1, "action": "권장 조치", "effort": "low|medium|high"}}
  ],
  "auto_fixable": true|false,
  "fix_suggestion": "자동 수정 가능한 경우 구체적 변경 내용"
}}
```

JSON만 출력하세요."""

    try:
        # 모델명은 환경변수로 설정 가능 (기본값: claude-sonnet-4-20250514)
        model = os.environ.get("DIAGNOSTIC_MODEL", "claude-sonnet-4-20250514")
        response = client.messages.create(
            model=model,
            max_tokens=1024,
            messages=[{"role": "user", "content": prompt}]
        )

        content = response.content[0].text

        # JSON 추출
        json_match = re.search(r'\{[\s\S]*\}', content)
        if json_match:
            return json.loads(json_match.group())

        return {"error": "JSON 파싱 실패", "raw": content[:200]}

    except Exception as e:
        return {"error": str(e)}


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(json.dumps({"error": "Usage: diagnostic-agent.py '<json_log>'"}))
        sys.exit(1)

    try:
        error_log = json.loads(sys.argv[1])
        result = diagnose(error_log)
        print(json.dumps(result, ensure_ascii=False, indent=2))
    except json.JSONDecodeError as e:
        print(json.dumps({"error": f"Invalid JSON: {e}"}))
        sys.exit(1)
```

### 체크리스트

- [ ] Python 스크립트 생성
- [ ] `anthropic` 패키지 설치
- [ ] `ANTHROPIC_API_KEY` 환경변수 설정
- [ ] 실행 권한: `chmod +x scripts/diagnostic-agent.py`

### 테스트

```bash
export ANTHROPIC_API_KEY="sk-ant-xxx"

./scripts/diagnostic-agent.py '{"level":"ERROR","target":"server::domain::ai::service","fields":{"error_code":"AI_003"},"message":"timeout"}'
```

---

## 태스크 3.2: Log Watcher 연동

### 구현

**파일**: `scripts/log-watcher.sh` (수정)

기존 Discord 알림 부분을 다음으로 교체:

> **중요**: 비용 제한 로직(`check_rate_limit`)을 진단 호출 **전에** 확인합니다.

```bash
# ERROR 감지 시 (기존 중복 체크 이후)
if [ "$LEVEL" = "ERROR" ]; then
    # ... 중복 체크 로직 ...

    # 비용 제한 체크 (진단 호출 전 필수)
    if ! python3 -c "
import time
from pathlib import Path

RATE_LIMIT_FILE = Path('/tmp/diagnostic-rate-limit')
MAX_CALLS_PER_HOUR = 10

now = time.time()
hour_ago = now - 3600

if not RATE_LIMIT_FILE.exists():
    RATE_LIMIT_FILE.write_text(str(now))
    exit(0)  # 허용

calls = [float(t) for t in RATE_LIMIT_FILE.read_text().split('\n') if t]
recent_calls = [t for t in calls if t > hour_ago]

if len(recent_calls) >= MAX_CALLS_PER_HOUR:
    exit(1)  # 제한 초과

recent_calls.append(now)
RATE_LIMIT_FILE.write_text('\n'.join(str(t) for t in recent_calls))
exit(0)  # 허용
"; then
        # 비용 제한 초과 - 기본 알림만 발송
        echo "[$(date)] Rate limit exceeded, skipping diagnostic"
        ./scripts/discord-alert.sh "critical" \
            "🚨 [$ERROR_CODE] Error Detected (진단 제한 초과)" \
            "**Location**: $TARGET\n**Request ID**: $REQUEST_ID\n\n$MESSAGE" \
            "$ERROR_CODE"
        continue
    fi

    # Diagnostic Agent 호출 (비용 제한 통과 후)
    echo "[$(date)] Running diagnostic for: $ERROR_CODE"
    DIAGNOSTIC=$(python3 ./scripts/diagnostic-agent.py "$line" 2>/dev/null)

    if echo "$DIAGNOSTIC" | jq -e '.error' > /dev/null 2>&1; then
        # 진단 실패 - 기본 알림
        ./scripts/discord-alert.sh "critical" \
            "🚨 [$ERROR_CODE] Error Detected" \
            "**Location**: $TARGET\n**Request ID**: $REQUEST_ID\n\n$MESSAGE" \
            "$ERROR_CODE"
    else
        # 진단 성공 - 상세 알림
        SEVERITY=$(echo "$DIAGNOSTIC" | jq -r '.severity // "critical"')
        ROOT_CAUSE=$(echo "$DIAGNOSTIC" | jq -r '.root_cause // "분석 중"')
        RECOMMENDATIONS=$(echo "$DIAGNOSTIC" | jq -r '.recommendations[0].action // "검토 필요"')

        ./scripts/discord-alert.sh "$SEVERITY" \
            "🔍 [$ERROR_CODE] AI 진단 완료" \
            "**근본 원인**: $ROOT_CAUSE\n\n**권장 조치**: $RECOMMENDATIONS\n\n**위치**: $TARGET" \
            "$ERROR_CODE"
    fi
fi
```

### 체크리스트

- [ ] Log Watcher에 진단 호출 추가
- [ ] 진단 실패 시 fallback 알림
- [ ] 진단 결과로 알림 메시지 개선

---

## 태스크 3.3: 비용 관리

### API 호출 제한

**파일**: `scripts/diagnostic-agent.py` (추가)

```python
import time
from pathlib import Path

RATE_LIMIT_FILE = Path("/tmp/diagnostic-rate-limit")
MAX_CALLS_PER_HOUR = 10

def check_rate_limit() -> bool:
    """시간당 호출 제한 확인"""
    now = time.time()
    hour_ago = now - 3600

    if not RATE_LIMIT_FILE.exists():
        RATE_LIMIT_FILE.write_text("")
        return True

    # 1시간 내 호출 기록
    calls = [float(t) for t in RATE_LIMIT_FILE.read_text().split('\n') if t]
    recent_calls = [t for t in calls if t > hour_ago]

    if len(recent_calls) >= MAX_CALLS_PER_HOUR:
        return False

    # 현재 호출 기록
    recent_calls.append(now)
    RATE_LIMIT_FILE.write_text('\n'.join(str(t) for t in recent_calls))
    return True
```

### 비용 추정

| 항목 | 수치 |
|------|------|
| 호출당 입력 토큰 | ~2,000 |
| 호출당 출력 토큰 | ~500 |
| 호출당 비용 | ~$0.01 |
| 일일 예상 호출 | 50회 |
| **월간 예상 비용** | **~$15** |

---

## 진단 출력 스키마

```json
{
  "severity": "critical",
  "root_cause": "Claude API 호출 시 30초 타임아웃이 설정되어 있으나 응답 지연 발생",
  "impact": "회고 어시스턴트 기능 전체 사용 불가",
  "recommendations": [
    {
      "priority": 1,
      "action": "타임아웃 값을 30초에서 45초로 증가",
      "effort": "low"
    },
    {
      "priority": 2,
      "action": "재시도 로직에 지수 백오프 적용",
      "effort": "medium"
    }
  ],
  "auto_fixable": true,
  "fix_suggestion": "src/domain/ai/client.rs의 TIMEOUT_SECS를 30에서 45로 변경"
}
```

---

## 산출물

Phase 3 완료 시:

1. **AI 기반 진단**
   - 에러 발생 시 자동으로 근본 원인 분석

2. **컨텍스트 기반 분석**
   - 소스 코드 + git 이력 참조

3. **구조화된 보고서**
   - 심각도, 원인, 권장 조치 포함

4. **개선된 알림**
   - Discord에 진단 결과 포함

---

## 다음 Phase 연결

Phase 4에서:
- `auto_fixable: true` → Auto-Fix Agent 트리거
- 진단 결과 → GitHub Issue 자동 생성
