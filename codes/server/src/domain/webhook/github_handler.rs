//! GitHub webhook handler for AI automation pipeline
//!
//! Handles GitHub webhook events:
//! - Issues (opened, labeled with ai-fix/ai-review)
//! - Issue comments (@ai-bot mentions)
//! - Pull requests (opened, labeled with ai-review)
//!
//! Includes branch validation for security governance:
//! - Allowed patterns: fix/*, hotfix/*, ai-fix/*, config/*, refactor/*
//! - Blocked patterns: main, master, dev, release/*

use crate::domain::webhook::dto::{
    GitHubEventType, GitHubWebhookResponse, IssueCommentPayload, IssuesPayload, PullRequestPayload,
};
use crate::event::{Event, Priority};
use crate::utils::AppError;
use axum::{body::Bytes, http::HeaderMap, Json};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::{debug, error, info, warn};

// ============================================================================
// Branch Validation
// ============================================================================

/// Branch validation for security governance
///
/// Validates branches against allowed and blocked patterns as defined in
/// docs/ai-automation/security-governance.md
#[derive(Debug, Clone)]
pub struct BranchValidator {
    /// Patterns for branches that AI automation is allowed to push to
    allowed_patterns: Vec<String>,
    /// Patterns for branches that are blocked from AI automation
    blocked_patterns: Vec<String>,
}

impl Default for BranchValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl BranchValidator {
    /// Create a new BranchValidator with default patterns from security governance
    pub fn new() -> Self {
        Self {
            allowed_patterns: vec![
                "fix/*".to_string(),
                "hotfix/*".to_string(),
                "ai-fix/*".to_string(),
                "config/*".to_string(),
                "refactor/*".to_string(),
            ],
            blocked_patterns: vec![
                "main".to_string(),
                "master".to_string(),
                "dev".to_string(),
                "release/*".to_string(),
            ],
        }
    }

    /// Check if a branch matches a glob pattern
    /// Supports `*` wildcard for any characters after the pattern prefix
    fn matches_pattern(branch: &str, pattern: &str) -> bool {
        if pattern.ends_with("/*") {
            // Pattern like "fix/*" - check if branch starts with "fix/"
            let prefix = &pattern[..pattern.len() - 1]; // "fix/"
            branch.starts_with(prefix)
        } else {
            // Exact match
            branch == pattern
        }
    }

    /// Check if a branch is in the allowed patterns
    pub fn is_allowed(&self, branch: &str) -> bool {
        self.allowed_patterns
            .iter()
            .any(|pattern| Self::matches_pattern(branch, pattern))
    }

    /// Check if a branch is in the blocked patterns
    pub fn is_blocked(&self, branch: &str) -> bool {
        self.blocked_patterns
            .iter()
            .any(|pattern| Self::matches_pattern(branch, pattern))
    }

    /// Validate a PR's branches (both head and base)
    /// Returns Ok(()) if valid, or Err with reason if invalid
    pub fn validate_pr_branches(
        &self,
        head_ref: &str,
        base_ref: &str,
    ) -> Result<(), BranchValidationError> {
        // Check if head branch (source) is blocked
        if self.is_blocked(head_ref) {
            return Err(BranchValidationError::BlockedSourceBranch(
                head_ref.to_string(),
            ));
        }

        // Check if base branch (target) is not a protected branch for direct push
        // For PRs, we allow targeting protected branches, but the source must be valid
        // Base validation is informational - PRs can target main/dev
        // The key restriction is that AI automation cannot push to protected branches directly

        debug!(
            head = %head_ref,
            base = %base_ref,
            head_allowed = self.is_allowed(head_ref),
            "Branch validation passed"
        );

        Ok(())
    }
}

/// Error type for branch validation
#[derive(Debug, Clone, PartialEq)]
pub enum BranchValidationError {
    /// Source branch is blocked
    BlockedSourceBranch(String),
    /// Target branch is blocked (reserved for future use)
    #[allow(dead_code)]
    BlockedTargetBranch(String),
}

impl std::fmt::Display for BranchValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BranchValidationError::BlockedSourceBranch(branch) => {
                write!(f, "Source branch '{}' is blocked for AI automation", branch)
            }
            BranchValidationError::BlockedTargetBranch(branch) => {
                write!(f, "Target branch '{}' is blocked for AI automation", branch)
            }
        }
    }
}

impl std::error::Error for BranchValidationError {}

/// Verify GitHub webhook signature using HMAC-SHA256 with constant-time comparison
fn verify_github_signature(secret: &str, signature: &str, body: &[u8]) -> Result<(), AppError> {
    // Skip verification if GITHUB_SKIP_VERIFICATION=true (development)
    if std::env::var("GITHUB_SKIP_VERIFICATION").unwrap_or_default() == "true" {
        warn!("GitHub signature verification skipped (development mode)");
        return Ok(());
    }

    let signature_hex = signature.strip_prefix("sha256=").ok_or_else(|| {
        warn!("Invalid GitHub signature format: missing sha256= prefix");
        AppError::Unauthorized("Invalid signature format".to_string())
    })?;

    // Decode the hex signature from GitHub
    let signature_bytes = hex::decode(signature_hex).map_err(|e| {
        warn!(error = %e, "Invalid hex in GitHub signature");
        AppError::Unauthorized("Invalid signature format".to_string())
    })?;

    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|e| {
        error!(error = %e, "HMAC initialization failed");
        AppError::InternalError("HMAC error".to_string())
    })?;

    mac.update(body);

    // Use constant-time comparison to prevent timing attacks
    mac.verify_slice(&signature_bytes).map_err(|_| {
        warn!("GitHub signature mismatch");
        AppError::Unauthorized("Signature mismatch".to_string())
    })?;

    Ok(())
}

/// Handle GitHub webhook events
///
/// Endpoint: POST /api/webhooks/github
///
/// This handler:
/// 1. Verifies the GitHub signature (security)
/// 2. Routes events based on X-GitHub-Event header
/// 3. Parses payloads and creates events for the AI pipeline
pub async fn handle_github_webhook(
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<GitHubWebhookResponse>, AppError> {
    // 1. Verify signature
    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing X-Hub-Signature-256 header");
            AppError::Unauthorized("Missing signature header".to_string())
        })?;

    let secret = std::env::var("GITHUB_WEBHOOK_SECRET").map_err(|_| {
        error!("GITHUB_WEBHOOK_SECRET not configured");
        AppError::InternalError("GITHUB_WEBHOOK_SECRET not configured".to_string())
    })?;

    verify_github_signature(&secret, signature, &body)?;

    // 2. Extract event type from header
    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .map(GitHubEventType::from)
        .ok_or_else(|| {
            warn!("Missing X-GitHub-Event header");
            AppError::BadRequest("Missing X-GitHub-Event header".to_string())
        })?;

    debug!(event_type = ?event_type, "Received GitHub webhook");

    // 3. Route based on event type
    let event = match event_type {
        GitHubEventType::Issues => {
            let payload: IssuesPayload = serde_json::from_slice(&body).map_err(|e| {
                error!(error = %e, "Failed to parse issues payload");
                AppError::BadRequest(format!("Invalid payload: {}", e))
            })?;
            handle_issues_event(payload)?
        }
        GitHubEventType::IssueComment => {
            let payload: IssueCommentPayload = serde_json::from_slice(&body).map_err(|e| {
                error!(error = %e, "Failed to parse issue_comment payload");
                AppError::BadRequest(format!("Invalid payload: {}", e))
            })?;
            handle_issue_comment_event(payload)?
        }
        GitHubEventType::PullRequest => {
            let payload: PullRequestPayload = serde_json::from_slice(&body).map_err(|e| {
                error!(error = %e, "Failed to parse pull_request payload");
                AppError::BadRequest(format!("Invalid payload: {}", e))
            })?;
            handle_pull_request_event(payload)?
        }
        GitHubEventType::Unknown(event_name) => {
            warn!(event = %event_name, "Unsupported GitHub event type");
            return Ok(Json(GitHubWebhookResponse::ignored(format!(
                "Unsupported event type: {}",
                event_name
            ))));
        }
    };

    // 4. If event was created, log and return accepted
    if let Some(event) = event {
        info!(
            event_id = %event.id,
            event_type = %event.event_type,
            priority = ?event.priority,
            "Created event from GitHub webhook"
        );

        // In a real implementation, we would push this to the event queue
        // For now, just return accepted

        Ok(Json(GitHubWebhookResponse::accepted()))
    } else {
        Ok(Json(GitHubWebhookResponse::ignored(
            "Event did not match trigger conditions",
        )))
    }
}

/// Handle Issues events (opened, labeled)
fn handle_issues_event(payload: IssuesPayload) -> Result<Option<Event>, AppError> {
    info!(
        action = %payload.action,
        issue_number = payload.issue.number,
        repository = %payload.repository.full_name,
        "Processing issues event"
    );

    match payload.action.as_str() {
        "opened" => {
            // Trigger if issue has ai-review label
            if payload.has_label("ai-review") {
                let event = Event::new(
                    "github.issue_opened",
                    "github",
                    Priority::P1,
                    serde_json::json!({
                        "action": "opened",
                        "issue_number": payload.issue.number,
                        "issue_title": payload.issue.title,
                        "issue_body": payload.issue.body,
                        "labels": payload.issue.labels.iter().map(|l| &l.name).collect::<Vec<_>>(),
                        "repository": payload.repository.full_name,
                        "sender": payload.sender.login,
                    }),
                )
                .with_user(&payload.sender.login);

                return Ok(Some(event));
            }
        }
        "labeled" => {
            // Trigger if ai-fix label was added
            if payload.is_adding_label("ai-fix") {
                let event = Event::new(
                    "github.issue_labeled",
                    "github",
                    Priority::P1,
                    serde_json::json!({
                        "action": "labeled",
                        "label": "ai-fix",
                        "issue_number": payload.issue.number,
                        "issue_title": payload.issue.title,
                        "issue_body": payload.issue.body,
                        "repository": payload.repository.full_name,
                        "sender": payload.sender.login,
                    }),
                )
                .with_user(&payload.sender.login);

                return Ok(Some(event));
            }

            // Also trigger for ai-review label
            if payload.is_adding_label("ai-review") {
                let event = Event::new(
                    "github.issue_labeled",
                    "github",
                    Priority::P1,
                    serde_json::json!({
                        "action": "labeled",
                        "label": "ai-review",
                        "issue_number": payload.issue.number,
                        "issue_title": payload.issue.title,
                        "issue_body": payload.issue.body,
                        "repository": payload.repository.full_name,
                        "sender": payload.sender.login,
                    }),
                )
                .with_user(&payload.sender.login);

                return Ok(Some(event));
            }
        }
        _ => {
            debug!(action = %payload.action, "Ignoring issues action");
        }
    }

    Ok(None)
}

/// Handle Issue Comment events (created)
fn handle_issue_comment_event(payload: IssueCommentPayload) -> Result<Option<Event>, AppError> {
    info!(
        action = %payload.action,
        issue_number = payload.issue.number,
        comment_id = payload.comment.id,
        repository = %payload.repository.full_name,
        "Processing issue_comment event"
    );

    if payload.action != "created" {
        debug!(action = %payload.action, "Ignoring issue_comment action");
        return Ok(None);
    }

    // Check for @ai-bot mention
    if payload.mentions_ai_bot() {
        let ai_command = payload.parse_ai_command();

        let event = Event::new(
            "github.issue_comment_created",
            "github",
            Priority::P1,
            serde_json::json!({
                "action": "created",
                "issue_number": payload.issue.number,
                "issue_title": payload.issue.title,
                "comment_id": payload.comment.id,
                "comment_body": payload.comment.body,
                "ai_command": ai_command,
                "repository": payload.repository.full_name,
                "sender": payload.sender.login,
            }),
        )
        .with_user(&payload.sender.login);

        return Ok(Some(event));
    }

    Ok(None)
}

/// Handle Pull Request events (opened, labeled)
///
/// Includes branch validation:
/// - Validates head_ref (source branch) is not blocked
/// - Logs warning if source branch is from blocked patterns
fn handle_pull_request_event(payload: PullRequestPayload) -> Result<Option<Event>, AppError> {
    info!(
        action = %payload.action,
        pr_number = payload.pull_request.number,
        repository = %payload.repository.full_name,
        head_ref = %payload.pull_request.head.ref_name,
        base_ref = %payload.pull_request.base.ref_name,
        "Processing pull_request event"
    );

    // Validate branches before processing
    let validator = BranchValidator::new();
    if let Err(e) = validator.validate_pr_branches(
        &payload.pull_request.head.ref_name,
        &payload.pull_request.base.ref_name,
    ) {
        warn!(
            error = %e,
            head_ref = %payload.pull_request.head.ref_name,
            base_ref = %payload.pull_request.base.ref_name,
            pr_number = payload.pull_request.number,
            "PR branch validation failed - ignoring event"
        );
        return Ok(None);
    }

    match payload.action.as_str() {
        "opened" => {
            // Trigger if PR has ai-review label
            if payload.has_label("ai-review") {
                let event = Event::new(
                    "github.pr_opened",
                    "github",
                    Priority::P2,
                    serde_json::json!({
                        "action": "opened",
                        "pr_number": payload.pull_request.number,
                        "pr_title": payload.pull_request.title,
                        "pr_body": payload.pull_request.body,
                        "head_ref": payload.pull_request.head.ref_name,
                        "head_sha": payload.pull_request.head.sha,
                        "base_ref": payload.pull_request.base.ref_name,
                        "labels": payload.pull_request.labels.iter().map(|l| &l.name).collect::<Vec<_>>(),
                        "repository": payload.repository.full_name,
                        "sender": payload.sender.login,
                        "branch_validated": true,
                    }),
                )
                .with_user(&payload.sender.login);

                return Ok(Some(event));
            }
        }
        "labeled" => {
            // Trigger if ai-review label was added
            if payload.is_adding_label("ai-review") {
                let event = Event::new(
                    "github.pr_labeled",
                    "github",
                    Priority::P2,
                    serde_json::json!({
                        "action": "labeled",
                        "label": "ai-review",
                        "pr_number": payload.pull_request.number,
                        "pr_title": payload.pull_request.title,
                        "pr_body": payload.pull_request.body,
                        "head_ref": payload.pull_request.head.ref_name,
                        "head_sha": payload.pull_request.head.sha,
                        "base_ref": payload.pull_request.base.ref_name,
                        "repository": payload.repository.full_name,
                        "sender": payload.sender.login,
                        "branch_validated": true,
                    }),
                )
                .with_user(&payload.sender.login);

                return Ok(Some(event));
            }
        }
        _ => {
            debug!(action = %payload.action, "Ignoring pull_request action");
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::webhook::dto::{
        GitHubComment, GitHubGitRef, GitHubIssue, GitHubLabel, GitHubPullRequest, GitHubRepository,
        GitHubUser,
    };

    fn create_test_issue(labels: Vec<&str>) -> GitHubIssue {
        GitHubIssue {
            number: 123,
            title: "Test Issue".to_string(),
            body: Some("Issue body".to_string()),
            labels: labels
                .into_iter()
                .map(|n| GitHubLabel {
                    name: n.to_string(),
                })
                .collect(),
            user: GitHubUser {
                login: "testuser".to_string(),
            },
        }
    }

    fn create_test_pr(labels: Vec<&str>) -> GitHubPullRequest {
        GitHubPullRequest {
            number: 456,
            title: "Test PR".to_string(),
            body: Some("PR body".to_string()),
            labels: labels
                .into_iter()
                .map(|n| GitHubLabel {
                    name: n.to_string(),
                })
                .collect(),
            head: GitHubGitRef {
                ref_name: "feature".to_string(),
                sha: "abc123".to_string(),
            },
            base: GitHubGitRef {
                ref_name: "main".to_string(),
                sha: "def456".to_string(),
            },
            user: GitHubUser {
                login: "developer".to_string(),
            },
        }
    }

    fn create_test_repo() -> GitHubRepository {
        GitHubRepository {
            full_name: "org/repo".to_string(),
        }
    }

    fn create_test_user() -> GitHubUser {
        GitHubUser {
            login: "testuser".to_string(),
        }
    }

    #[test]
    fn should_create_event_for_issue_with_ai_review_label_on_open() {
        // Arrange
        let payload = IssuesPayload {
            action: "opened".to_string(),
            issue: create_test_issue(vec!["bug", "ai-review"]),
            label: None,
            repository: create_test_repo(),
            sender: create_test_user(),
        };

        // Act
        let result = handle_issues_event(payload).unwrap();

        // Assert
        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.event_type, "github.issue_opened");
        assert_eq!(event.priority, Priority::P1);
    }

    #[test]
    fn should_create_event_when_ai_fix_label_added() {
        // Arrange
        let payload = IssuesPayload {
            action: "labeled".to_string(),
            issue: create_test_issue(vec!["bug", "ai-fix"]),
            label: Some(GitHubLabel {
                name: "ai-fix".to_string(),
            }),
            repository: create_test_repo(),
            sender: create_test_user(),
        };

        // Act
        let result = handle_issues_event(payload).unwrap();

        // Assert
        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.event_type, "github.issue_labeled");
        assert_eq!(
            event.data.get("label").and_then(|v| v.as_str()),
            Some("ai-fix")
        );
    }

    #[test]
    fn should_not_create_event_for_issue_without_ai_labels() {
        // Arrange
        let payload = IssuesPayload {
            action: "opened".to_string(),
            issue: create_test_issue(vec!["bug"]),
            label: None,
            repository: create_test_repo(),
            sender: create_test_user(),
        };

        // Act
        let result = handle_issues_event(payload).unwrap();

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn should_create_event_for_comment_with_ai_bot_mention() {
        // Arrange
        let payload = IssueCommentPayload {
            action: "created".to_string(),
            issue: create_test_issue(vec![]),
            comment: GitHubComment {
                id: 789,
                body: "@ai-bot analyze this issue".to_string(),
                user: GitHubUser {
                    login: "developer".to_string(),
                },
            },
            repository: create_test_repo(),
            sender: create_test_user(),
        };

        // Act
        let result = handle_issue_comment_event(payload).unwrap();

        // Assert
        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.event_type, "github.issue_comment_created");
        assert_eq!(event.priority, Priority::P1);
    }

    #[test]
    fn should_not_create_event_for_comment_without_ai_bot_mention() {
        // Arrange
        let payload = IssueCommentPayload {
            action: "created".to_string(),
            issue: create_test_issue(vec![]),
            comment: GitHubComment {
                id: 789,
                body: "Regular comment without mention".to_string(),
                user: GitHubUser {
                    login: "developer".to_string(),
                },
            },
            repository: create_test_repo(),
            sender: create_test_user(),
        };

        // Act
        let result = handle_issue_comment_event(payload).unwrap();

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn should_create_event_for_pr_with_ai_review_label() {
        // Arrange
        let payload = PullRequestPayload {
            action: "opened".to_string(),
            pull_request: create_test_pr(vec!["ai-review"]),
            label: None,
            repository: create_test_repo(),
            sender: create_test_user(),
        };

        // Act
        let result = handle_pull_request_event(payload).unwrap();

        // Assert
        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.event_type, "github.pr_opened");
        assert_eq!(event.priority, Priority::P2);
    }

    #[test]
    fn should_create_event_when_ai_review_label_added_to_pr() {
        // Arrange
        let payload = PullRequestPayload {
            action: "labeled".to_string(),
            pull_request: create_test_pr(vec!["ai-review"]),
            label: Some(GitHubLabel {
                name: "ai-review".to_string(),
            }),
            repository: create_test_repo(),
            sender: create_test_user(),
        };

        // Act
        let result = handle_pull_request_event(payload).unwrap();

        // Assert
        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.event_type, "github.pr_labeled");
    }

    #[test]
    fn should_not_create_event_for_pr_without_ai_labels() {
        // Arrange
        let payload = PullRequestPayload {
            action: "opened".to_string(),
            pull_request: create_test_pr(vec!["bugfix"]),
            label: None,
            repository: create_test_repo(),
            sender: create_test_user(),
        };

        // Act
        let result = handle_pull_request_event(payload).unwrap();

        // Assert
        assert!(result.is_none());
    }

    // ========================================================================
    // Branch Validator Tests
    // ========================================================================

    #[test]
    fn should_allow_fix_branch_pattern() {
        // Arrange
        let validator = BranchValidator::new();

        // Act & Assert
        assert!(validator.is_allowed("fix/bug-123"));
        assert!(validator.is_allowed("fix/issue-456"));
        assert!(validator.is_allowed("fix/"));
    }

    #[test]
    fn should_allow_hotfix_branch_pattern() {
        // Arrange
        let validator = BranchValidator::new();

        // Act & Assert
        assert!(validator.is_allowed("hotfix/critical-bug"));
        assert!(validator.is_allowed("hotfix/urgent-fix"));
    }

    #[test]
    fn should_allow_ai_fix_branch_pattern() {
        // Arrange
        let validator = BranchValidator::new();

        // Act & Assert
        assert!(validator.is_allowed("ai-fix/auto-generated"));
        assert!(validator.is_allowed("ai-fix/issue-789"));
    }

    #[test]
    fn should_allow_config_branch_pattern() {
        // Arrange
        let validator = BranchValidator::new();

        // Act & Assert
        assert!(validator.is_allowed("config/update-settings"));
        assert!(validator.is_allowed("config/env-changes"));
    }

    #[test]
    fn should_allow_refactor_branch_pattern() {
        // Arrange
        let validator = BranchValidator::new();

        // Act & Assert
        assert!(validator.is_allowed("refactor/cleanup"));
        assert!(validator.is_allowed("refactor/code-improvements"));
    }

    #[test]
    fn should_not_allow_feature_branch() {
        // Arrange
        let validator = BranchValidator::new();

        // Act & Assert
        assert!(!validator.is_allowed("feature/new-feature"));
        assert!(!validator.is_allowed("main"));
        assert!(!validator.is_allowed("dev"));
    }

    #[test]
    fn should_block_main_branch() {
        // Arrange
        let validator = BranchValidator::new();

        // Act & Assert
        assert!(validator.is_blocked("main"));
    }

    #[test]
    fn should_block_master_branch() {
        // Arrange
        let validator = BranchValidator::new();

        // Act & Assert
        assert!(validator.is_blocked("master"));
    }

    #[test]
    fn should_block_dev_branch() {
        // Arrange
        let validator = BranchValidator::new();

        // Act & Assert
        assert!(validator.is_blocked("dev"));
    }

    #[test]
    fn should_block_release_branch_pattern() {
        // Arrange
        let validator = BranchValidator::new();

        // Act & Assert
        assert!(validator.is_blocked("release/1.0.0"));
        assert!(validator.is_blocked("release/v2.0"));
        assert!(validator.is_blocked("release/"));
    }

    #[test]
    fn should_not_block_fix_branch() {
        // Arrange
        let validator = BranchValidator::new();

        // Act & Assert
        assert!(!validator.is_blocked("fix/bug-123"));
        assert!(!validator.is_blocked("feature/something"));
    }

    #[test]
    fn should_validate_pr_with_allowed_head_branch() {
        // Arrange
        let validator = BranchValidator::new();

        // Act
        let result = validator.validate_pr_branches("fix/bug-123", "dev");

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_reject_pr_with_blocked_head_branch() {
        // Arrange
        let validator = BranchValidator::new();

        // Act
        let result = validator.validate_pr_branches("main", "dev");

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            BranchValidationError::BlockedSourceBranch("main".to_string())
        );
    }

    #[test]
    fn should_reject_pr_from_release_branch() {
        // Arrange
        let validator = BranchValidator::new();

        // Act
        let result = validator.validate_pr_branches("release/1.0.0", "dev");

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            BranchValidationError::BlockedSourceBranch("release/1.0.0".to_string())
        );
    }

    #[test]
    fn should_allow_pr_targeting_main() {
        // Arrange
        let validator = BranchValidator::new();

        // Act - PRs can target main, the restriction is on the source branch
        let result = validator.validate_pr_branches("fix/bug-123", "main");

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_match_glob_pattern_with_wildcard() {
        // Arrange & Act & Assert
        assert!(BranchValidator::matches_pattern("fix/bug", "fix/*"));
        assert!(BranchValidator::matches_pattern("fix/nested/path", "fix/*"));
        assert!(!BranchValidator::matches_pattern("feature/bug", "fix/*"));
        assert!(!BranchValidator::matches_pattern("fix", "fix/*")); // must have slash
    }

    #[test]
    fn should_match_exact_pattern() {
        // Arrange & Act & Assert
        assert!(BranchValidator::matches_pattern("main", "main"));
        assert!(!BranchValidator::matches_pattern("main-backup", "main"));
        assert!(!BranchValidator::matches_pattern("mains", "main"));
    }

    #[test]
    fn should_not_create_event_for_pr_from_blocked_branch() {
        // Arrange
        let mut pr = create_test_pr(vec!["ai-review"]);
        pr.head.ref_name = "main".to_string(); // blocked source branch

        let payload = PullRequestPayload {
            action: "opened".to_string(),
            pull_request: pr,
            label: None,
            repository: create_test_repo(),
            sender: create_test_user(),
        };

        // Act
        let result = handle_pull_request_event(payload).unwrap();

        // Assert - event should be ignored due to blocked branch
        assert!(result.is_none());
    }

    #[test]
    fn should_not_create_event_for_pr_labeled_from_blocked_branch() {
        // Arrange
        let mut pr = create_test_pr(vec!["ai-review"]);
        pr.head.ref_name = "release/1.0.0".to_string(); // blocked source branch

        let payload = PullRequestPayload {
            action: "labeled".to_string(),
            pull_request: pr,
            label: Some(GitHubLabel {
                name: "ai-review".to_string(),
            }),
            repository: create_test_repo(),
            sender: create_test_user(),
        };

        // Act
        let result = handle_pull_request_event(payload).unwrap();

        // Assert - event should be ignored due to blocked branch
        assert!(result.is_none());
    }

    #[test]
    fn should_create_event_for_pr_from_allowed_branch() {
        // Arrange
        let mut pr = create_test_pr(vec!["ai-review"]);
        pr.head.ref_name = "fix/bug-123".to_string(); // allowed source branch

        let payload = PullRequestPayload {
            action: "opened".to_string(),
            pull_request: pr,
            label: None,
            repository: create_test_repo(),
            sender: create_test_user(),
        };

        // Act
        let result = handle_pull_request_event(payload).unwrap();

        // Assert - event should be created
        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.event_type, "github.pr_opened");
        assert_eq!(
            event.data.get("branch_validated").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn branch_validation_error_display() {
        // Arrange & Act & Assert
        let error = BranchValidationError::BlockedSourceBranch("main".to_string());
        assert_eq!(
            error.to_string(),
            "Source branch 'main' is blocked for AI automation"
        );

        let error = BranchValidationError::BlockedTargetBranch("release/1.0".to_string());
        assert_eq!(
            error.to_string(),
            "Target branch 'release/1.0' is blocked for AI automation"
        );
    }

    #[test]
    fn should_verify_github_signature() {
        // Arrange
        std::env::set_var("GITHUB_SKIP_VERIFICATION", "false");
        let secret = "test_secret";
        let body = b"test body";

        // Calculate expected signature
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let expected = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        // Act
        let result = verify_github_signature(secret, &expected, body);

        // Assert
        assert!(result.is_ok());

        // Cleanup
        std::env::remove_var("GITHUB_SKIP_VERIFICATION");
    }

    #[test]
    fn should_reject_invalid_github_signature() {
        // Arrange
        std::env::set_var("GITHUB_SKIP_VERIFICATION", "false");
        let secret = "test_secret";
        let body = b"test body";
        let invalid_signature = "sha256=invalid";

        // Act
        let result = verify_github_signature(secret, invalid_signature, body);

        // Assert
        assert!(result.is_err());

        // Cleanup
        std::env::remove_var("GITHUB_SKIP_VERIFICATION");
    }
}
