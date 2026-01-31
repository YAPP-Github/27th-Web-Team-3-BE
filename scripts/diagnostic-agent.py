#!/usr/bin/env python3
"""
AI Diagnostic Agent - 에러 로그 분석 및 진단
"""

import os
import sys
import json
import subprocess
import re
import time
from pathlib import Path
from anthropic import Anthropic

client = Anthropic()

# Rate limit 설정
RATE_LIMIT_FILE = Path("/tmp/diagnostic-rate-limit")
MAX_CALLS_PER_HOUR = 10

def check_rate_limit() -> bool:
    """시간당 호출 제한 확인"""
    now = time.time()
    hour_ago = now - 3600

    if not RATE_LIMIT_FILE.exists():
        RATE_LIMIT_FILE.write_text("")
        return True

    calls = [float(t) for t in RATE_LIMIT_FILE.read_text().split('\n') if t]
    recent_calls = [t for t in calls if t > hour_ago]

    if len(recent_calls) >= MAX_CALLS_PER_HOUR:
        return False

    recent_calls.append(now)
    RATE_LIMIT_FILE.write_text('\n'.join(str(t) for t in recent_calls))
    return True

def collect_source_context(target: str) -> str:
    """target에서 소스 파일 추출하고 읽기"""
    # server::domain::ai::service → src/domain/ai/service.rs
    path = target.replace("server::", "src/").replace("::", "/") + ".rs"
    full_path = f"codes/server/{path}"

    if os.path.exists(full_path):
        with open(full_path, encoding="utf-8") as f:
            content = f.read()
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
    # Rate limit 체크
    if not check_rate_limit():
        return {"error": "Rate limit exceeded (max 10 calls/hour)"}

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
        model = os.environ.get("DIAGNOSTIC_MODEL", "claude-sonnet-4-20250514")
        response = client.messages.create(
            model=model,
            max_tokens=1024,
            messages=[{"role": "user", "content": prompt}]
        )

        content = response.content[0].text
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
