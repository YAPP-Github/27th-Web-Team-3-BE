//! AI 코드 수정 안전장치 모듈
//!
//! 코드 수정 전 검증 항목, 수정 범위 제한, 수정 허용/불허 범위를 정의합니다.
//!
//! ## 사용 예시
//! ```rust,ignore
//! use server::automation::{SafetyChecks, FixLimits, FixScope};
//!
//! // 안전 검증
//! let mut checks = SafetyChecks::new();
//! checks.syntax_valid = true;
//! checks.compiles = true;
//! checks.tests_pass = true;
//! checks.no_security_impact = true;
//!
//! if checks.is_safe() {
//!     // 수정 적용
//! }
//!
//! // 수정 범위 제한
//! let limits = FixLimits::default();
//! ```

use serde::{Deserialize, Serialize};

/// 수정 전 검증 항목
///
/// AI 코드 수정이 적용되기 전에 반드시 통과해야 하는 검증 항목들을 정의합니다.
/// 필수 검증 항목이 모두 통과해야 수정이 적용됩니다.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SafetyChecks {
    // ============== 필수 검증 ==============
    /// 문법 오류 없음 (cargo check 통과)
    pub syntax_valid: bool,

    /// 컴파일 성공 (cargo build 통과)
    pub compiles: bool,

    /// 기존 테스트 통과 (cargo test 통과)
    pub tests_pass: bool,

    /// 보안 영향 없음 (보안 관련 파일 미수정)
    pub no_security_impact: bool,

    // ============== 권장 검증 ==============
    /// Clippy 경고 없음 (cargo clippy -- -D warnings 통과)
    pub clippy_clean: bool,

    /// 커버리지 유지 (테스트 커버리지 저하 없음)
    pub coverage_maintained: bool,

    /// API 호환성 유지 (breaking changes 없음)
    pub no_breaking_changes: bool,
}

impl SafetyChecks {
    /// 모든 검증 항목을 false로 초기화한 새 인스턴스 생성
    ///
    /// # Examples
    /// ```rust
    /// use server::automation::SafetyChecks;
    ///
    /// let checks = SafetyChecks::new();
    /// assert!(!checks.is_safe());
    /// ```
    pub fn new() -> Self {
        Self {
            syntax_valid: false,
            compiles: false,
            tests_pass: false,
            no_security_impact: false,
            clippy_clean: false,
            coverage_maintained: false,
            no_breaking_changes: false,
        }
    }

    /// 필수 검증 항목이 모두 통과했는지 확인
    ///
    /// 필수 검증 항목:
    /// - `syntax_valid`: 문법 오류 없음
    /// - `compiles`: 컴파일 성공
    /// - `tests_pass`: 기존 테스트 통과
    /// - `no_security_impact`: 보안 영향 없음
    ///
    /// # Returns
    /// 모든 필수 검증 항목이 true이면 true, 그렇지 않으면 false
    ///
    /// # Examples
    /// ```rust
    /// use server::automation::SafetyChecks;
    ///
    /// let mut checks = SafetyChecks::new();
    /// assert!(!checks.is_safe());
    ///
    /// checks.syntax_valid = true;
    /// checks.compiles = true;
    /// checks.tests_pass = true;
    /// checks.no_security_impact = true;
    /// assert!(checks.is_safe());
    /// ```
    pub fn is_safe(&self) -> bool {
        self.syntax_valid && self.compiles && self.tests_pass && self.no_security_impact
    }

    /// 모든 검증 항목을 검사하고 실패한 항목 목록 반환
    ///
    /// # Returns
    /// - `Ok(())`: 모든 필수 검증 통과
    /// - `Err(Vec<String>)`: 실패한 검증 항목 목록
    ///
    /// # Examples
    /// ```rust
    /// use server::automation::SafetyChecks;
    ///
    /// let checks = SafetyChecks::new();
    /// let result = checks.validate_all();
    /// assert!(result.is_err());
    ///
    /// let errors = result.unwrap_err();
    /// assert!(errors.contains(&"syntax_valid: 문법 검증 실패".to_string()));
    /// ```
    pub fn validate_all(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        // 필수 검증 항목 확인
        if !self.syntax_valid {
            failures.push("syntax_valid: 문법 검증 실패".to_string());
        }
        if !self.compiles {
            failures.push("compiles: 컴파일 실패".to_string());
        }
        if !self.tests_pass {
            failures.push("tests_pass: 테스트 실패".to_string());
        }
        if !self.no_security_impact {
            failures.push("no_security_impact: 보안 영향 검증 실패".to_string());
        }

        // 권장 검증 항목 확인 (경고만)
        if !self.clippy_clean {
            failures.push("clippy_clean: Clippy 경고 존재 (권장)".to_string());
        }
        if !self.coverage_maintained {
            failures.push("coverage_maintained: 커버리지 저하 (권장)".to_string());
        }
        if !self.no_breaking_changes {
            failures.push("no_breaking_changes: API 호환성 문제 (권장)".to_string());
        }

        if self.is_safe() {
            Ok(())
        } else {
            Err(failures)
        }
    }

    /// 모든 필수 검증 항목을 true로 설정
    ///
    /// 테스트 또는 특수한 경우에만 사용합니다.
    pub fn set_all_required_passed(&mut self) {
        self.syntax_valid = true;
        self.compiles = true;
        self.tests_pass = true;
        self.no_security_impact = true;
    }

    /// 모든 검증 항목을 true로 설정
    ///
    /// 테스트 또는 특수한 경우에만 사용합니다.
    pub fn set_all_passed(&mut self) {
        self.set_all_required_passed();
        self.clippy_clean = true;
        self.coverage_maintained = true;
        self.no_breaking_changes = true;
    }
}

/// 수정 범위 제한 설정
///
/// AI 코드 수정의 범위를 제한하여 과도한 변경을 방지합니다.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixLimits {
    /// 한 번에 수정 가능한 최대 파일 수
    pub max_files_per_fix: u32,

    /// 파일당 최대 수정 라인 수
    pub max_lines_per_file: u32,

    /// 수정 가능한 최대 함수 수
    pub max_functions_modified: u32,

    /// 수정 허용 경로 패턴 목록
    /// 예: `["codes/server/src/domain/**/*.rs", "codes/server/src/utils/**/*.rs"]`
    pub allowed_paths: Vec<String>,

    /// 수정 금지 경로 패턴 목록
    /// 예: `["codes/server/src/main.rs", "codes/server/src/config/**", "**/*.env*"]`
    pub forbidden_paths: Vec<String>,
}

impl Default for FixLimits {
    /// Phase 3 문서에 정의된 기본 제한 값
    fn default() -> Self {
        Self {
            max_files_per_fix: 3,
            max_lines_per_file: 50,
            max_functions_modified: 2,
            allowed_paths: vec![
                "codes/server/src/domain/**/*.rs".to_string(),
                "codes/server/src/utils/**/*.rs".to_string(),
            ],
            forbidden_paths: vec![
                "codes/server/src/main.rs".to_string(),
                "codes/server/src/config/**".to_string(),
                "**/*.sql".to_string(),
                "**/*.env*".to_string(),
                "**/migrations/**".to_string(),
            ],
        }
    }
}

impl FixLimits {
    /// 새 FixLimits 인스턴스 생성
    ///
    /// # Arguments
    /// * `max_files` - 최대 수정 파일 수
    /// * `max_lines` - 파일당 최대 수정 라인 수
    /// * `max_functions` - 최대 수정 함수 수
    pub fn new(max_files: u32, max_lines: u32, max_functions: u32) -> Self {
        Self {
            max_files_per_fix: max_files,
            max_lines_per_file: max_lines,
            max_functions_modified: max_functions,
            allowed_paths: Vec::new(),
            forbidden_paths: Vec::new(),
        }
    }

    /// 허용 경로 추가
    pub fn add_allowed_path(&mut self, path: impl Into<String>) -> &mut Self {
        self.allowed_paths.push(path.into());
        self
    }

    /// 금지 경로 추가
    pub fn add_forbidden_path(&mut self, path: impl Into<String>) -> &mut Self {
        self.forbidden_paths.push(path.into());
        self
    }

    /// 경로가 수정 가능한지 검사
    ///
    /// 금지 경로에 해당하면 false, 허용 경로에 해당하면 true
    /// 둘 다 해당하지 않으면 false (명시적 허용 필요)
    ///
    /// # Arguments
    /// * `path` - 검사할 파일 경로
    ///
    /// # Returns
    /// 수정 가능 여부를 나타내는 `FixScope`
    pub fn check_path(&self, path: &str) -> FixScope {
        // 금지 경로 검사 (우선)
        for forbidden in &self.forbidden_paths {
            if Self::matches_glob(path, forbidden) {
                return FixScope::Forbidden(format!("금지된 경로: {}", forbidden));
            }
        }

        // 허용 경로 검사
        for allowed in &self.allowed_paths {
            if Self::matches_glob(path, allowed) {
                return FixScope::Allowed(format!("허용된 경로: {}", allowed));
            }
        }

        // 둘 다 해당하지 않으면 리뷰 필요
        FixScope::RequiresReview(format!(
            "경로가 허용 목록에 없음: {}. 수동 검토 필요.",
            path
        ))
    }

    /// 간단한 glob 패턴 매칭
    ///
    /// 지원하는 패턴:
    /// - `*`: 디렉토리 구분자를 제외한 모든 문자
    /// - `**`: 모든 문자 (디렉토리 구분자 포함)
    fn matches_glob(path: &str, pattern: &str) -> bool {
        let regex_pattern = pattern
            .replace(".", "\\.")
            .replace("**", "{{DOUBLE_STAR}}")
            .replace('*', "[^/]*")
            .replace("{{DOUBLE_STAR}}", ".*");

        regex::Regex::new(&format!("^{}$", regex_pattern))
            .map(|re| re.is_match(path))
            .unwrap_or(false)
    }

    /// 수정 횟수가 제한 내에 있는지 검사
    pub fn check_limits(&self, files: u32, lines: u32, functions: u32) -> Result<(), Vec<String>> {
        let mut violations = Vec::new();

        if files > self.max_files_per_fix {
            violations.push(format!(
                "파일 수 초과: {} > {} (최대)",
                files, self.max_files_per_fix
            ));
        }
        if lines > self.max_lines_per_file {
            violations.push(format!(
                "라인 수 초과: {} > {} (최대)",
                lines, self.max_lines_per_file
            ));
        }
        if functions > self.max_functions_modified {
            violations.push(format!(
                "함수 수 초과: {} > {} (최대)",
                functions, self.max_functions_modified
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
}

/// 수정 허용/불허 범위 판단 결과
///
/// AI 코드 수정의 허용 여부와 그 이유를 나타냅니다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "reason")]
pub enum FixScope {
    /// 수정 허용됨
    Allowed(String),

    /// 수정 금지됨
    Forbidden(String),

    /// 수동 검토 필요
    RequiresReview(String),
}

impl FixScope {
    /// 수정이 허용되었는지 확인
    pub fn is_allowed(&self) -> bool {
        matches!(self, FixScope::Allowed(_))
    }

    /// 수정이 금지되었는지 확인
    pub fn is_forbidden(&self) -> bool {
        matches!(self, FixScope::Forbidden(_))
    }

    /// 수동 검토가 필요한지 확인
    pub fn requires_review(&self) -> bool {
        matches!(self, FixScope::RequiresReview(_))
    }

    /// 이유 문자열 반환
    pub fn reason(&self) -> &str {
        match self {
            FixScope::Allowed(reason) => reason,
            FixScope::Forbidden(reason) => reason,
            FixScope::RequiresReview(reason) => reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============== SafetyChecks 테스트 ==============

    mod safety_checks_tests {
        use super::*;

        #[test]
        fn should_create_new_instance_with_all_false() {
            // Arrange & Act
            let checks = SafetyChecks::new();

            // Assert
            assert!(!checks.syntax_valid);
            assert!(!checks.compiles);
            assert!(!checks.tests_pass);
            assert!(!checks.no_security_impact);
            assert!(!checks.clippy_clean);
            assert!(!checks.coverage_maintained);
            assert!(!checks.no_breaking_changes);
        }

        #[test]
        fn should_return_false_when_any_required_check_fails() {
            // Arrange
            let mut checks = SafetyChecks::new();
            checks.syntax_valid = true;
            checks.compiles = true;
            checks.tests_pass = true;
            // no_security_impact is still false

            // Act
            let is_safe = checks.is_safe();

            // Assert
            assert!(!is_safe);
        }

        #[test]
        fn should_return_true_when_all_required_checks_pass() {
            // Arrange
            let mut checks = SafetyChecks::new();
            checks.syntax_valid = true;
            checks.compiles = true;
            checks.tests_pass = true;
            checks.no_security_impact = true;
            // 권장 검증은 false여도 is_safe()는 true

            // Act
            let is_safe = checks.is_safe();

            // Assert
            assert!(is_safe);
        }

        #[test]
        fn should_return_ok_when_all_required_checks_pass() {
            // Arrange
            let mut checks = SafetyChecks::new();
            checks.syntax_valid = true;
            checks.compiles = true;
            checks.tests_pass = true;
            checks.no_security_impact = true;

            // Act
            let result = checks.validate_all();

            // Assert
            assert!(result.is_ok());
        }

        #[test]
        fn should_return_error_with_failed_checks_list() {
            // Arrange
            let checks = SafetyChecks::new();

            // Act
            let result = checks.validate_all();

            // Assert
            assert!(result.is_err());
            let errors = result.unwrap_err();
            assert!(errors.contains(&"syntax_valid: 문법 검증 실패".to_string()));
            assert!(errors.contains(&"compiles: 컴파일 실패".to_string()));
            assert!(errors.contains(&"tests_pass: 테스트 실패".to_string()));
            assert!(errors.contains(&"no_security_impact: 보안 영향 검증 실패".to_string()));
        }

        #[test]
        fn should_include_recommended_checks_in_validation() {
            // Arrange
            let mut checks = SafetyChecks::new();
            checks.set_all_required_passed();
            // 권장 검증은 false

            // Act
            let result = checks.validate_all();

            // Assert
            assert!(result.is_ok()); // 필수 통과시 Ok 반환
        }

        #[test]
        fn should_set_all_required_passed() {
            // Arrange
            let mut checks = SafetyChecks::new();

            // Act
            checks.set_all_required_passed();

            // Assert
            assert!(checks.syntax_valid);
            assert!(checks.compiles);
            assert!(checks.tests_pass);
            assert!(checks.no_security_impact);
            assert!(!checks.clippy_clean);
            assert!(!checks.coverage_maintained);
            assert!(!checks.no_breaking_changes);
        }

        #[test]
        fn should_set_all_passed() {
            // Arrange
            let mut checks = SafetyChecks::new();

            // Act
            checks.set_all_passed();

            // Assert
            assert!(checks.syntax_valid);
            assert!(checks.compiles);
            assert!(checks.tests_pass);
            assert!(checks.no_security_impact);
            assert!(checks.clippy_clean);
            assert!(checks.coverage_maintained);
            assert!(checks.no_breaking_changes);
        }

        #[test]
        fn should_serialize_to_camel_case() {
            // Arrange
            let mut checks = SafetyChecks::new();
            checks.syntax_valid = true;

            // Act
            let json = serde_json::to_string(&checks).unwrap();

            // Assert
            assert!(json.contains("\"syntaxValid\":true"));
            assert!(json.contains("\"noSecurityImpact\":false"));
        }

        #[test]
        fn should_deserialize_from_camel_case() {
            // Arrange
            let json = r#"{"syntaxValid":true,"compiles":false,"testsPass":false,"noSecurityImpact":false,"clippyClean":false,"coverageMaintained":false,"noBreakingChanges":false}"#;

            // Act
            let checks: SafetyChecks = serde_json::from_str(json).unwrap();

            // Assert
            assert!(checks.syntax_valid);
            assert!(!checks.compiles);
        }
    }

    // ============== FixLimits 테스트 ==============

    mod fix_limits_tests {
        use super::*;

        #[test]
        fn should_create_default_limits() {
            // Arrange & Act
            let limits = FixLimits::default();

            // Assert
            assert_eq!(limits.max_files_per_fix, 3);
            assert_eq!(limits.max_lines_per_file, 50);
            assert_eq!(limits.max_functions_modified, 2);
            assert!(!limits.allowed_paths.is_empty());
            assert!(!limits.forbidden_paths.is_empty());
        }

        #[test]
        fn should_create_custom_limits() {
            // Arrange & Act
            let limits = FixLimits::new(5, 100, 4);

            // Assert
            assert_eq!(limits.max_files_per_fix, 5);
            assert_eq!(limits.max_lines_per_file, 100);
            assert_eq!(limits.max_functions_modified, 4);
            assert!(limits.allowed_paths.is_empty());
            assert!(limits.forbidden_paths.is_empty());
        }

        #[test]
        fn should_add_allowed_path() {
            // Arrange
            let mut limits = FixLimits::new(1, 1, 1);

            // Act
            limits.add_allowed_path("src/**/*.rs");

            // Assert
            assert_eq!(limits.allowed_paths.len(), 1);
            assert_eq!(limits.allowed_paths[0], "src/**/*.rs");
        }

        #[test]
        fn should_add_forbidden_path() {
            // Arrange
            let mut limits = FixLimits::new(1, 1, 1);

            // Act
            limits.add_forbidden_path("**/*.env");

            // Assert
            assert_eq!(limits.forbidden_paths.len(), 1);
            assert_eq!(limits.forbidden_paths[0], "**/*.env");
        }

        #[test]
        fn should_return_forbidden_for_forbidden_path() {
            // Arrange
            let limits = FixLimits::default();

            // Act
            let result = limits.check_path("codes/server/src/main.rs");

            // Assert
            assert!(result.is_forbidden());
        }

        #[test]
        fn should_return_allowed_for_allowed_path() {
            // Arrange
            let limits = FixLimits::default();

            // Act
            let result = limits.check_path("codes/server/src/domain/ai/service.rs");

            // Assert
            assert!(result.is_allowed());
        }

        #[test]
        fn should_return_requires_review_for_unknown_path() {
            // Arrange
            let limits = FixLimits::default();

            // Act
            let result = limits.check_path("some/other/path.rs");

            // Assert
            assert!(result.requires_review());
        }

        #[test]
        fn should_prioritize_forbidden_over_allowed() {
            // Arrange
            let mut limits = FixLimits::new(1, 1, 1);
            limits.add_allowed_path("src/**/*.rs");
            limits.add_forbidden_path("src/config/**");

            // Act
            let result = limits.check_path("src/config/database.rs");

            // Assert
            assert!(result.is_forbidden());
        }

        #[test]
        fn should_check_limits_within_bounds() {
            // Arrange
            let limits = FixLimits::default();

            // Act
            let result = limits.check_limits(2, 30, 1);

            // Assert
            assert!(result.is_ok());
        }

        #[test]
        fn should_check_limits_exceeding_bounds() {
            // Arrange
            let limits = FixLimits::default();

            // Act
            let result = limits.check_limits(5, 100, 5);

            // Assert
            assert!(result.is_err());
            let violations = result.unwrap_err();
            assert_eq!(violations.len(), 3);
        }

        #[test]
        fn should_match_glob_pattern_with_double_star() {
            // Arrange
            let limits = FixLimits::default();

            // Act & Assert
            let result = limits.check_path("codes/server/src/domain/auth/handler.rs");
            assert!(result.is_allowed());
        }

        #[test]
        fn should_match_glob_pattern_with_env_extension() {
            // Arrange
            let limits = FixLimits::default();

            // Act
            let result = limits.check_path("codes/server/.env.local");

            // Assert
            assert!(result.is_forbidden());
        }

        #[test]
        fn should_serialize_to_camel_case() {
            // Arrange
            let limits = FixLimits::default();

            // Act
            let json = serde_json::to_string(&limits).unwrap();

            // Assert
            assert!(json.contains("\"maxFilesPerFix\":3"));
            assert!(json.contains("\"maxLinesPerFile\":50"));
            assert!(json.contains("\"allowedPaths\""));
            assert!(json.contains("\"forbiddenPaths\""));
        }
    }

    // ============== FixScope 테스트 ==============

    mod fix_scope_tests {
        use super::*;

        #[test]
        fn should_identify_allowed_scope() {
            // Arrange
            let scope = FixScope::Allowed("허용됨".to_string());

            // Assert
            assert!(scope.is_allowed());
            assert!(!scope.is_forbidden());
            assert!(!scope.requires_review());
        }

        #[test]
        fn should_identify_forbidden_scope() {
            // Arrange
            let scope = FixScope::Forbidden("금지됨".to_string());

            // Assert
            assert!(!scope.is_allowed());
            assert!(scope.is_forbidden());
            assert!(!scope.requires_review());
        }

        #[test]
        fn should_identify_requires_review_scope() {
            // Arrange
            let scope = FixScope::RequiresReview("검토 필요".to_string());

            // Assert
            assert!(!scope.is_allowed());
            assert!(!scope.is_forbidden());
            assert!(scope.requires_review());
        }

        #[test]
        fn should_return_reason_string() {
            // Arrange
            let scope = FixScope::Allowed("테스트 이유".to_string());

            // Act
            let reason = scope.reason();

            // Assert
            assert_eq!(reason, "테스트 이유");
        }

        #[test]
        fn should_serialize_with_type_tag() {
            // Arrange
            let scope = FixScope::Allowed("허용됨".to_string());

            // Act
            let json = serde_json::to_string(&scope).unwrap();

            // Assert
            assert!(json.contains("\"type\":\"allowed\""));
            assert!(json.contains("\"reason\":\"허용됨\""));
        }

        #[test]
        fn should_deserialize_from_type_tag() {
            // Arrange
            let json = r#"{"type":"forbidden","reason":"금지된 경로"}"#;

            // Act
            let scope: FixScope = serde_json::from_str(json).unwrap();

            // Assert
            assert!(scope.is_forbidden());
            assert_eq!(scope.reason(), "금지된 경로");
        }

        #[test]
        fn should_implement_eq() {
            // Arrange
            let scope1 = FixScope::Allowed("테스트".to_string());
            let scope2 = FixScope::Allowed("테스트".to_string());
            let scope3 = FixScope::Forbidden("테스트".to_string());

            // Assert
            assert_eq!(scope1, scope2);
            assert_ne!(scope1, scope3);
        }
    }
}
