//! GitHub webhook handler for AI automation pipeline
//!
//! Handles GitHub webhook events:
//! - Issues (opened, labeled with ai-fix/ai-review)
//! - Issue comments (@ai-bot mentions)
//! - Pull requests (opened, labeled with ai-review)

use crate::domain::webhook::dto::{
    GitHubEventType, GitHubWebhookResponse, IssueCommentPayload, IssuesPayload, PullRequestPayload,
};
use crate::event::{Event, Priority};
use crate::utils::AppError;
use axum::{body::Bytes, http::HeaderMap, Json};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::{debug, error, info, warn};

/// Verify GitHub webhook signature using HMAC-SHA256
fn verify_github_signature(secret: &str, signature: &str, body: &[u8]) -> Result<(), AppError> {
    // Skip verification if GITHUB_SKIP_VERIFICATION=true (development)
    if std::env::var("GITHUB_SKIP_VERIFICATION").unwrap_or_default() == "true" {
        warn!("GitHub signature verification skipped (development mode)");
        return Ok(());
    }

    let signature = signature.strip_prefix("sha256=").ok_or_else(|| {
        warn!("Invalid GitHub signature format: missing sha256= prefix");
        AppError::Unauthorized("Invalid signature format".to_string())
    })?;

    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|e| {
        error!(error = %e, "HMAC initialization failed");
        AppError::InternalError("HMAC error".to_string())
    })?;

    mac.update(body);

    let expected = hex::encode(mac.finalize().into_bytes());

    if signature != expected {
        warn!("GitHub signature mismatch");
        return Err(AppError::Unauthorized("Signature mismatch".to_string()));
    }

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
fn handle_pull_request_event(payload: PullRequestPayload) -> Result<Option<Event>, AppError> {
    info!(
        action = %payload.action,
        pr_number = payload.pull_request.number,
        repository = %payload.repository.full_name,
        "Processing pull_request event"
    );

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
