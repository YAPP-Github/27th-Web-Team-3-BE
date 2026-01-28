# Gemini Project Context

## Core Instruction
이 프로젝트는 `CLAUDE.md`와 `.claude/` 디렉토리에 정의된 개발 가이드라인을 엄격히 따릅니다. Gemini는 모든 작업 시 다음 사항을 준수해야 합니다.

구현 후 테스트할때는 src/tests안에 테스트 파일을 만들어서 작성해주세요 (mock 형 단위테스트면 됩니다.)
테스트 후에는 반드시 docs/reviews에 리뷰 문서를 작성해주세요. auth-api와 동일한 형식을 따르세요.

**필수 준수 사항:**
1.  **`CLAUDE.md` 우선 참조**: 프로젝트 구조, 빌드/테스트 명령, TDD 워크플로우를 이해하기 위해 항상 최우선으로 참조합니다.
2.  **`.claude/rules/` 규칙 적용**:
    *   Rust 코드 작성 시: `rust-src.md` 적용
    *   테스트 작성 시: `rust-tests.md` 적용
    *   API 설계 시: `api-design.md` 적용
    *   커밋 시: `tidy-first-commit.md` 적용
3.  **`.claude/commands/` 도구 활용**:
    *   빌드, 테스트, 품질 검사 시 해당 디렉토리의 마크다운 파일에 정의된 표준 명령어를 사용합니다.
4.  **`.claude/skills/` 및 `hooks/` 참조**: 특정 상황(아키텍처 체크, 코드 품질 관리 등) 발생 시 해당 가이드를 따릅니다.

## 핵심 개발 원칙 (CLAUDE.md 요약)
*   **TDD (Test-Driven Development)**: Red(실패하는 테스트) -> Green(최소 구현) -> Refactor(개선) 순서를 지킵니다.
*   **Error Handling**: `unwrap()`이나 `expect()` 사용을 금지하며, `Result`와 `?` 연산자를 사용합니다.
*   **Serialization**: 모든 DTO는 `#[serde(rename_all = "camelCase")]`를 필수적으로 포함해야 합니다.
*   **Tidy First**: 구조적 변경과 행동적 변경을 분리하여 커밋합니다.

이 설정은 Gemini가 프로젝트의 일관성을 유지하고 기존 도구들과 완벽하게 협업하도록 보장합니다.
