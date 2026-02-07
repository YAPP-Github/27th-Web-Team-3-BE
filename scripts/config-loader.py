#!/usr/bin/env python3
"""
Config Loader - automation.config.yaml 설정 로더
다른 스크립트에서 설정 값을 읽을 때 사용

Usage:
    # 단일 값 조회
    python3 config-loader.py automation.enabled
    python3 config-loader.py phases.auto_fix.enabled
    python3 config-loader.py models.diagnostic.model

    # 전체 설정 JSON 출력
    python3 config-loader.py --json

    # 환경변수 형식 출력 (source로 사용 가능)
    python3 config-loader.py --env
"""

import sys
import json
from pathlib import Path

try:
    import yaml
except ImportError:
    print("Error: PyYAML not installed. Run: pip install pyyaml", file=sys.stderr)
    sys.exit(1)


def get_config_path() -> Path:
    """설정 파일 경로 반환"""
    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    return project_root / "automation.config.yaml"


def load_config() -> dict:
    """설정 파일 로드"""
    config_path = get_config_path()

    if not config_path.exists():
        return get_default_config()

    with open(config_path, 'r', encoding='utf-8') as f:
        return yaml.safe_load(f) or {}


def get_default_config() -> dict:
    """기본 설정 (설정 파일이 없을 때)"""
    return {
        "automation": {"enabled": False},
        "phases": {
            "log_watcher": {"enabled": False, "interval_minutes": 5},
            "ai_diagnostic": {"enabled": False},
            "issue_creation": {"enabled": False},
            "auto_fix": {"enabled": False},
        },
        "models": {
            "diagnostic": {
                "provider": "openai",
                "model": "gpt-4o-mini",
                "max_tokens": 1024,
            },
            "auto_fix": {
                "provider": "anthropic",
                "model": "claude-sonnet-4-20250514",
            },
        },
        "rate_limits": {
            "diagnostic": {"max_calls_per_hour": 10},
            "auto_fix": {"max_daily_prs": 5},
        },
        "notifications": {
            "discord": {"enabled": True},
            "severity_filter": ["critical", "warning"],
        },
        "git": {
            "base_branch": "dev",
            "auto_push": True,
        },
    }


def get_nested_value(config: dict, key_path: str):
    """점(.)으로 구분된 경로로 중첩 값 조회"""
    keys = key_path.split('.')
    value = config

    for key in keys:
        if isinstance(value, dict) and key in value:
            value = value[key]
        else:
            return None

    return value


def is_enabled(phase: str = None) -> bool:
    """자동화 또는 특정 Phase 활성화 여부 확인"""
    config = load_config()

    # 전체 자동화가 꺼져있으면 항상 False
    if not config.get("automation", {}).get("enabled", False):
        return False

    # 특정 Phase 확인
    if phase:
        return config.get("phases", {}).get(phase, {}).get("enabled", False)

    return True


def to_env_format(config: dict, prefix: str = "AUTOMATION") -> list:
    """설정을 환경변수 형식으로 변환"""
    env_vars = []

    def flatten(obj, path=""):
        if isinstance(obj, dict):
            for k, v in obj.items():
                new_path = f"{path}_{k}" if path else k
                flatten(v, new_path)
        elif isinstance(obj, list):
            env_vars.append(f'{prefix}_{path.upper()}="{",".join(str(x) for x in obj)}"')
        elif isinstance(obj, bool):
            env_vars.append(f'{prefix}_{path.upper()}={"true" if obj else "false"}')
        else:
            env_vars.append(f'{prefix}_{path.upper()}="{obj}"')

    flatten(config)
    return env_vars


def main():
    if len(sys.argv) < 2:
        print("Usage: config-loader.py <key.path> | --json | --env | --check [phase]")
        sys.exit(1)

    arg = sys.argv[1]

    if arg == "--json":
        config = load_config()
        print(json.dumps(config, indent=2, ensure_ascii=False))

    elif arg == "--env":
        config = load_config()
        for line in to_env_format(config):
            print(line)

    elif arg == "--check":
        phase = sys.argv[2] if len(sys.argv) > 2 else None
        enabled = is_enabled(phase)
        print("true" if enabled else "false")
        sys.exit(0 if enabled else 1)

    else:
        config = load_config()
        value = get_nested_value(config, arg)

        if value is None:
            print(f"Key not found: {arg}", file=sys.stderr)
            sys.exit(1)

        if isinstance(value, bool):
            print("true" if value else "false")
        elif isinstance(value, (dict, list)):
            print(json.dumps(value, ensure_ascii=False))
        else:
            print(value)


if __name__ == "__main__":
    main()
