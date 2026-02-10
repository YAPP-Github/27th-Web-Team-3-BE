# Phase 3: AI Diagnostic

> **버전**: 2.0
> **최종 수정**: 2026-02-06
> **상태**: 구현 대기
> **의존성**: Phase 2 (Issue Analysis) 완료

---

## 개요

| 항목 | 내용 |
|------|------|
| Phase | 3: AI Diagnostic |
| 기간 | Week 5-6 |
| 목표 | Claude API를 활용한 에러 진단 및 근본 원인 분석 |
| 선행 조건 | Phase 2 (Issue Analysis) 완료 |
| 후속 단계 | Phase 4 (Issue Automation), Phase 5 (Auto-Fix & PR) |

```
Phase 3 완료 상태
┌─────────────────────────────────────────────────────────────────────────────┐
│  ⬜ Claude API 연동    ⬜ 컨텍스트 수집    ⬜ 진단 보고서    ⬜ 심각도 분류  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 1. 목표 및 범위

### 1.1 왜 필요한가?

Phase 2까지 구현된 시스템은 에러를 감지하고 기본 정보를 수집하지만,
**근본 원인 분석**은 개발자가 수동으로 수행해야 합니다.

**현재 문제:**
- 에러 로그만으로는 원인 파악 어려움
- 개발자가 직접 코드 분석 필요
- 유사 에러 패턴 학습 불가

**Phase 3 해결:**
- Claude API로 지능형 에러 진단
- 코드 컨텍스트 기반 근본 원인 분석
- 자동 수정 가능 여부 판단
- 권장 조치 생성

### 1.2 목표

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Phase 3 목표                                      │
├─────────────────────────────────────────────────────────────────────────┤
│  1. 에러 로그 + 소스 코드 컨텍스트 수집                                  │
│  2. Claude API를 통한 근본 원인 분석                                     │
│  3. 심각도 및 영향 범위 자동 분류                                        │
│  4. 자동 수정 가능 여부 판단 (auto_fixable)                              │
│  5. 권장 조치 및 수정 제안 생성                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.3 범위

**포함 (In Scope):**
- Claude API 연동 및 프롬프트 엔지니어링
- 소스 코드 컨텍스트 수집
- Git 이력 분석
- 진단 보고서 생성
- 심각도/auto_fixable 판단

**제외 (Out of Scope):**
- GitHub Issue 자동 생성 (Phase 4)
- 자동 코드 수정 (Phase 5)
- Draft PR 생성 (Phase 5)

---

## 2. 아키텍처

### 2.1 전체 흐름

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Phase 3 AI Diagnostic 파이프라인                     │
└─────────────────────────────────────────────────────────────────────────┘

  Phase 2 완료 (에러 감지 + 기본 분석)
       │
       ▼
┌──────────────────┐
│ 1. 컨텍스트 수집 │
│   - 소스 코드    │
│   - Git 이력     │
│   - 관련 파일    │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ 2. Claude API    │
│    진단 요청     │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ 3. 결과 파싱     │
│   - severity     │
│   - root_cause   │
│   - auto_fixable │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ 4. 진단 보고서   │
│    생성          │
└────────┬─────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
 Phase 4   Phase 5
 (Issue)   (Auto-Fix)
```

### 2.2 컴포넌트 구조

```
scripts/
├── log-watcher.sh          # Phase 2 (기존)
├── discord-alert.sh        # Phase 2 (기존)
├── collect-context.py      # Phase 3 (컨텍스트 수집)
├── diagnostic-agent.py     # Phase 3 (AI 진단) ⬅️
└── parse-error-log.sh      # Phase 2 (기존)
```

---

## 3. 구현 상세

### 3.1 컨텍스트 수집

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
    relative = target.replace("server::", "").replace("::", "/")
    return SERVER_DIR / "src" / f"{relative}.rs"


def collect_source_code(target: str) -> dict:
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
    """관련 파일 목록 수집 (같은 도메인의 다른 파일들)"""
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

### 3.2 AI 진단 에이전트

**파일**: `scripts/diagnostic-agent.py`

```python
#!/usr/bin/env python3
"""
Diagnostic Agent - Claude API를 활용한 에러 진단
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

# 에러 코드별 힌트
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

# 자동 수정 가능한 패턴
AUTO_FIXABLE_PATTERNS = {
    "timeout": "타임아웃 값 조정",
    "retry": "재시도 로직 추가",
    "validation": "검증 로직 추가",
    "null_check": "null 체크 추가",
    "logging": "로깅 개선",
}


def diagnose_error(context: dict) -> dict:
    """Claude API로 에러 진단"""

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
다음 에러를 분석하고 진단 결과를 제공하세요.

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

# 자동 수정 가능 판단 기준
다음 경우에만 auto_fixable: true로 설정하세요:
- 타임아웃 값 조정 (설정값 변경)
- 로깅 추가/개선
- 간단한 null 체크 추가
- 포맷팅/clippy 경고 해결
- 에러 메시지 개선

다음 경우는 auto_fixable: false:
- 비즈니스 로직 변경
- 아키텍처 변경
- 보안 관련 수정
- DB 스키마 변경
- API 인터페이스 변경

# 요청
다음 JSON 형식으로 분석 결과를 제공하세요:

```json
{{
  "severity": "critical|high|medium|low",
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
  "fix_suggestion": "자동 수정 가능한 경우 구체적 변경 내용"
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
            result["error_code"] = error_code
            result["analysis_model"] = model
            return result

        return {"error": "JSON 파싱 실패", "raw": content[:500]}

    except Exception as e:
        return {"error": str(e)}


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(json.dumps({"error": "Usage: diagnostic-agent.py '<context_json>'"}))
        sys.exit(1)

    try:
        context = json.loads(sys.argv[1])
        result = diagnose_error(context)
        print(json.dumps(result, ensure_ascii=False, indent=2))
    except json.JSONDecodeError as e:
        print(json.dumps({"error": f"Invalid JSON: {e}"}))
        sys.exit(1)
```

### 3.3 진단 결과 스키마

```json
{
  "error_code": "AI5003",
  "severity": "high",
  "root_cause": "Claude API 호출 시 30초 타임아웃 설정으로 인해 긴 프롬프트 처리 시 실패",
  "impact": "AI 회고 어시스턴트 기능 전체 사용 불가",
  "affected_users": "partial",
  "related_to_recent_change": true,
  "suspected_commit": "abc123",
  "recommendations": [
    {"priority": 1, "action": "타임아웃 값을 30초에서 45초로 증가", "effort": "low"},
    {"priority": 2, "action": "프롬프트 길이 제한 추가", "effort": "medium"},
    {"priority": 3, "action": "청크 단위 처리 구현", "effort": "high"}
  ],
  "auto_fixable": true,
  "fix_type": "config",
  "fix_suggestion": "src/domain/ai/client.rs의 TIMEOUT_SECS를 30에서 45로 변경",
  "analysis_model": "claude-sonnet-4-20250514"
}
```

---

## 4. 심각도 분류 체계

### 4.1 심각도 레벨

| 레벨 | 설명 | 대응 시간 | 자동화 액션 |
|------|------|----------|------------|
| `critical` | 서비스 중단 | 즉시 | Discord 알림 + Issue + Auto-Fix 시도 |
| `high` | 주요 기능 장애 | 4시간 내 | Discord 알림 + Issue 생성 |
| `medium` | 부분 장애 | 1일 내 | Issue 생성 |
| `low` | 마이너 | 1주 내 | 로그만 기록 |

### 4.2 자동 수정 판단 기준

**auto_fixable: true 조건:**
- 설정값 변경 (타임아웃, 재시도 횟수)
- 로깅 추가/개선
- 간단한 null/에러 체크 추가
- 포맷팅/clippy 경고 해결
- 에러 메시지 개선

**auto_fixable: false 조건:**
- 비즈니스 로직 변경
- 아키텍처 변경
- 보안 관련 수정
- DB 스키마 변경
- API 인터페이스 변경

---

## 5. 환경 설정

### 5.1 필수 환경 변수

```bash
# Claude API 키
export ANTHROPIC_API_KEY=sk-ant-xxxxx

# 진단 모델 (선택, 기본값: claude-sonnet-4-20250514)
export DIAGNOSTIC_MODEL=claude-sonnet-4-20250514
```

### 5.2 비용 제한

| 항목 | 제한 | 이유 |
|------|------|------|
| API 호출 | 시간당 10회 | 비용 제어 |
| 토큰 제한 | 입력 5000자, 출력 1500토큰 | 비용 최적화 |
| 모델 | claude-sonnet-4-20250514 | 비용 효율적 |

---

## 6. 테스트 시나리오

### 6.1 기본 진단 테스트

```bash
# 컨텍스트 수집 테스트
python3 ./scripts/collect-context.py '{
    "target": "server::domain::ai::service",
    "error_code": "AI5003",
    "message": "Claude API timeout"
}'

# AI 진단 테스트
export ANTHROPIC_API_KEY="sk-ant-xxx"
python3 ./scripts/diagnostic-agent.py '{
    "error": {
        "error_code": "AI5003",
        "target": "server::domain::ai::service",
        "message": "timeout after 30000ms"
    },
    "source": {
        "content": "pub async fn call_api() { ... }"
    }
}'
```

### 6.2 예상 결과

| 에러 코드 | 예상 severity | 예상 auto_fixable |
|----------|--------------|------------------|
| AI5001 | critical | false (보안 관련) |
| AI5003 | high | true (타임아웃 조정) |
| DB5002 | high | false (쿼리 최적화 필요) |
| AUTH4002 | medium | false (로직 검토 필요) |

---

## 7. 구현 체크리스트

### Day 1-2: 기본 구현

- [ ] `scripts/collect-context.py` 작성
- [ ] `scripts/diagnostic-agent.py` 작성
- [ ] ANTHROPIC_API_KEY 설정
- [ ] 기본 테스트

### Day 3-4: 연동 및 테스트

- [ ] `log-watcher.sh`와 연동
- [ ] 다양한 에러 코드 테스트
- [ ] 진단 결과 정확도 검증
- [ ] 프롬프트 튜닝

### Day 5: 검증 및 문서화

- [ ] E2E 테스트
- [ ] 비용 모니터링 설정
- [ ] 운영 가이드 문서화

---

## 8. 다음 단계

Phase 3 완료 후:

- **Phase 4 (Issue Automation)**: 진단 결과 기반 GitHub Issue 자동 생성
- **Phase 5 (Auto-Fix & PR)**: auto_fixable 에러 자동 수정 및 Draft PR 생성

---

## 참고 문서

- [Phase 1: Event Trigger](./phase-1-event-trigger.md)
- [Phase 2: Issue Analysis](./phase-2-issue-analysis.md)
- [Phase 4: Issue Automation](./phase-4-issue-automation.md)
- [Phase 5: Auto-Fix & PR](./phase-5-auto-fix-pr.md)
- [Overview](./overview.md)

---

#phase3 #ai-diagnostic #claude-api #error-analysis
