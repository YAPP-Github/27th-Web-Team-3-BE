//! DTOs for webhook payloads
//!
//! Contains data structures for:
//! - Discord webhook interactions
//! - GitHub webhook events (issues, issue_comment, pull_request)

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// Discord Webhook DTOs
// ============================================================================

/// Discord webhook interaction payload
#[derive(Debug, Clone, Deserialize)]
pub struct DiscordWebhookPayload {
    /// Interaction type (1 = Ping, 2 = Application Command, etc.)
    #[serde(rename = "type")]
    pub interaction_type: i32,
    /// Interaction token
    pub token: String,
    /// Channel ID
    pub channel_id: String,
    /// Guild ID (optional)
    pub guild_id: Option<String>,
    /// Message data (for message interactions)
    pub data: Option<DiscordMessageData>,
}

/// Discord message data within interaction
#[derive(Debug, Clone, Deserialize)]
pub struct DiscordMessageData {
    /// Message content
    pub content: String,
    /// Author information
    pub author: DiscordAuthor,
    /// Timestamp
    pub timestamp: String,
}

/// Discord author information
#[derive(Debug, Clone, Deserialize)]
pub struct DiscordAuthor {
    /// User ID
    pub id: String,
    /// Username
    pub username: String,
}

/// Discord webhook response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DiscordWebhookResponse {
    /// Response type (1 = Pong, 4 = Channel Message with Source, etc.)
    #[serde(rename = "type")]
    pub response_type: i32,
    /// Response data
    pub data: Option<DiscordResponseData>,
}

impl DiscordWebhookResponse {
    /// Create a pong response (for type 1 ping)
    pub fn pong() -> Self {
        Self {
            response_type: 1,
            data: None,
        }
    }

    /// Create a message response
    pub fn message(content: impl Into<String>) -> Self {
        Self {
            response_type: 4,
            data: Some(DiscordResponseData {
                content: content.into(),
            }),
        }
    }
}

/// Discord response data
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DiscordResponseData {
    /// Message content
    pub content: String,
}

/// Discord command types parsed from messages
#[derive(Debug, Clone, PartialEq)]
pub enum DiscordCommand {
    /// @AI analyze request
    Analyze { args: String },
    /// @AI fix request
    Fix { args: String },
    /// @AI review request
    Review { args: String },
    /// @AI status request
    Status,
    /// Unknown or unsupported command
    Unknown { content: String },
}

impl DiscordCommand {
    /// Parse command from message content
    pub fn parse(content: &str) -> Self {
        let content_lower = content.to_lowercase();

        if content_lower.contains("@ai") {
            if content_lower.contains("분석") || content_lower.contains("analyze") {
                let args = extract_args_after(content, &["분석해줘", "analyze", "분석"]);
                return DiscordCommand::Analyze { args };
            }
            if content_lower.contains("수정") || content_lower.contains("fix") {
                let args = extract_args_after(content, &["수정해줘", "fix", "수정"]);
                return DiscordCommand::Fix { args };
            }
            if content_lower.contains("리뷰") || content_lower.contains("review") {
                let args = extract_args_after(content, &["리뷰해줘", "review", "리뷰"]);
                return DiscordCommand::Review { args };
            }
            if content_lower.contains("상태") || content_lower.contains("status") {
                return DiscordCommand::Status;
            }
        }

        DiscordCommand::Unknown {
            content: content.to_string(),
        }
    }

    /// Get the command type as string
    pub fn command_type(&self) -> &str {
        match self {
            DiscordCommand::Analyze { .. } => "analyze",
            DiscordCommand::Fix { .. } => "fix",
            DiscordCommand::Review { .. } => "review",
            DiscordCommand::Status => "status",
            DiscordCommand::Unknown { .. } => "unknown",
        }
    }
}

/// Extract arguments after specific keywords (Unicode-safe)
fn extract_args_after(content: &str, keywords: &[&str]) -> String {
    let content_lower = content.to_lowercase();

    for keyword in keywords {
        if let Some(pos) = content_lower.find(*keyword) {
            // Find byte position in original string by counting characters
            let char_start = content_lower[..pos].chars().count();
            let keyword_chars = keyword.chars().count();

            // Convert character position to byte position in original string
            let byte_start: usize = content
                .chars()
                .take(char_start + keyword_chars)
                .map(|c| c.len_utf8())
                .sum();

            if byte_start < content.len() {
                return content[byte_start..].trim().to_string();
            }
        }
    }

    String::new()
}

// ============================================================================
// GitHub Webhook DTOs
// ============================================================================

/// GitHub event type from X-GitHub-Event header
#[derive(Debug, Clone, PartialEq)]
pub enum GitHubEventType {
    /// Issues events
    Issues,
    /// Issue comment events
    IssueComment,
    /// Pull request events
    PullRequest,
    /// Unknown event type
    Unknown(String),
}

impl From<&str> for GitHubEventType {
    fn from(s: &str) -> Self {
        match s {
            "issues" => GitHubEventType::Issues,
            "issue_comment" => GitHubEventType::IssueComment,
            "pull_request" => GitHubEventType::PullRequest,
            other => GitHubEventType::Unknown(other.to_string()),
        }
    }
}

/// GitHub Issues event payload (X-GitHub-Event: issues)
#[derive(Debug, Clone, Deserialize)]
pub struct IssuesPayload {
    /// Action (opened, labeled, etc.)
    pub action: String,
    /// Issue information
    pub issue: GitHubIssue,
    /// Label that was added (for labeled action)
    pub label: Option<GitHubLabel>,
    /// Repository information
    pub repository: GitHubRepository,
    /// User who triggered the event
    pub sender: GitHubUser,
}

/// GitHub Issue Comment event payload (X-GitHub-Event: issue_comment)
#[derive(Debug, Clone, Deserialize)]
pub struct IssueCommentPayload {
    /// Action (created, edited, deleted)
    pub action: String,
    /// Issue information
    pub issue: GitHubIssue,
    /// Comment information
    pub comment: GitHubComment,
    /// Repository information
    pub repository: GitHubRepository,
    /// User who triggered the event
    pub sender: GitHubUser,
}

/// GitHub Pull Request event payload (X-GitHub-Event: pull_request)
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestPayload {
    /// Action (opened, labeled, etc.)
    pub action: String,
    /// Pull request information
    pub pull_request: GitHubPullRequest,
    /// Label that was added (for labeled action)
    pub label: Option<GitHubLabel>,
    /// Repository information
    pub repository: GitHubRepository,
    /// User who triggered the event
    pub sender: GitHubUser,
}

/// GitHub Issue information
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubIssue {
    /// Issue number
    pub number: i64,
    /// Issue title
    pub title: String,
    /// Issue body (optional)
    pub body: Option<String>,
    /// Labels attached to the issue
    #[serde(default)]
    pub labels: Vec<GitHubLabel>,
    /// Issue author
    pub user: GitHubUser,
}

/// GitHub Pull Request information
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubPullRequest {
    /// PR number
    pub number: i64,
    /// PR title
    pub title: String,
    /// PR body (optional)
    pub body: Option<String>,
    /// Labels attached to the PR
    #[serde(default)]
    pub labels: Vec<GitHubLabel>,
    /// Head branch reference
    pub head: GitHubGitRef,
    /// Base branch reference
    pub base: GitHubGitRef,
    /// PR author
    pub user: GitHubUser,
}

/// GitHub Comment information
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubComment {
    /// Comment ID
    pub id: i64,
    /// Comment body
    pub body: String,
    /// Comment author
    pub user: GitHubUser,
}

/// GitHub Label information
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubLabel {
    /// Label name
    pub name: String,
}

/// GitHub Git reference (branch)
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubGitRef {
    /// Reference name (branch name)
    #[serde(rename = "ref")]
    pub ref_name: String,
    /// Commit SHA
    pub sha: String,
}

/// GitHub Repository information
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRepository {
    /// Full repository name (owner/repo)
    pub full_name: String,
}

/// GitHub User information
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubUser {
    /// Username/login
    pub login: String,
}

/// Generic GitHub webhook response
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GitHubWebhookResponse {
    /// Processing status
    pub status: String,
    /// Reason (for ignored events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl GitHubWebhookResponse {
    /// Create an accepted response
    pub fn accepted() -> Self {
        Self {
            status: "accepted".to_string(),
            reason: None,
        }
    }

    /// Create an ignored response with reason
    pub fn ignored(reason: impl Into<String>) -> Self {
        Self {
            status: "ignored".to_string(),
            reason: Some(reason.into()),
        }
    }
}

// ============================================================================
// Helper functions for GitHub payload processing
// ============================================================================

impl IssuesPayload {
    /// Check if issue has a specific label
    pub fn has_label(&self, label_name: &str) -> bool {
        self.issue.labels.iter().any(|l| l.name == label_name)
    }

    /// Check if the action is adding a specific label
    pub fn is_adding_label(&self, label_name: &str) -> bool {
        self.action == "labeled"
            && self
                .label
                .as_ref()
                .map(|l| l.name == label_name)
                .unwrap_or(false)
    }
}

impl IssueCommentPayload {
    /// Check if comment mentions @ai-bot
    pub fn mentions_ai_bot(&self) -> bool {
        self.comment.body.to_lowercase().contains("@ai-bot")
    }

    /// Parse AI bot command from comment
    pub fn parse_ai_command(&self) -> Option<String> {
        let body_lower = self.comment.body.to_lowercase();
        if body_lower.contains("@ai-bot") {
            // Extract command after @ai-bot mention
            if let Some(pos) = body_lower.find("@ai-bot") {
                let after = &self.comment.body[pos + 7..].trim();
                if !after.is_empty() {
                    return Some(after.to_string());
                }
            }
        }
        None
    }
}

impl PullRequestPayload {
    /// Check if PR has a specific label
    pub fn has_label(&self, label_name: &str) -> bool {
        self.pull_request
            .labels
            .iter()
            .any(|l| l.name == label_name)
    }

    /// Check if the action is adding a specific label
    pub fn is_adding_label(&self, label_name: &str) -> bool {
        self.action == "labeled"
            && self
                .label
                .as_ref()
                .map(|l| l.name == label_name)
                .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_discord_analyze_command_korean() {
        // Arrange
        let content = "@AI 분석해줘 AI5001 에러가 발생했어요";

        // Act
        let command = DiscordCommand::parse(content);

        // Assert
        match command {
            DiscordCommand::Analyze { args } => {
                assert!(args.contains("AI5001"));
            }
            _ => panic!("Expected Analyze command"),
        }
    }

    #[test]
    fn should_parse_discord_fix_command() {
        // Arrange
        let content = "@AI 수정해줘 이슈 #123";

        // Act
        let command = DiscordCommand::parse(content);

        // Assert
        match command {
            DiscordCommand::Fix { args } => {
                assert!(args.contains("#123"));
            }
            _ => panic!("Expected Fix command"),
        }
    }

    #[test]
    fn should_parse_discord_status_command() {
        // Arrange
        let content = "@AI 상태";

        // Act
        let command = DiscordCommand::parse(content);

        // Assert
        assert_eq!(command, DiscordCommand::Status);
    }

    #[test]
    fn should_parse_unknown_discord_command() {
        // Arrange
        let content = "Hello world";

        // Act
        let command = DiscordCommand::parse(content);

        // Assert
        match command {
            DiscordCommand::Unknown { content } => {
                assert_eq!(content, "Hello world");
            }
            _ => panic!("Expected Unknown command"),
        }
    }

    #[test]
    fn should_convert_github_event_type_from_header() {
        // Arrange & Act & Assert
        assert_eq!(GitHubEventType::from("issues"), GitHubEventType::Issues);
        assert_eq!(
            GitHubEventType::from("issue_comment"),
            GitHubEventType::IssueComment
        );
        assert_eq!(
            GitHubEventType::from("pull_request"),
            GitHubEventType::PullRequest
        );
        assert_eq!(
            GitHubEventType::from("push"),
            GitHubEventType::Unknown("push".to_string())
        );
    }

    #[test]
    fn should_create_discord_pong_response() {
        // Arrange & Act
        let response = DiscordWebhookResponse::pong();

        // Assert
        assert_eq!(response.response_type, 1);
        assert!(response.data.is_none());
    }

    #[test]
    fn should_create_discord_message_response() {
        // Arrange & Act
        let response = DiscordWebhookResponse::message("Test message");

        // Assert
        assert_eq!(response.response_type, 4);
        assert_eq!(response.data.unwrap().content, "Test message");
    }

    #[test]
    fn should_create_github_accepted_response() {
        // Arrange & Act
        let response = GitHubWebhookResponse::accepted();

        // Assert
        assert_eq!(response.status, "accepted");
        assert!(response.reason.is_none());
    }

    #[test]
    fn should_create_github_ignored_response() {
        // Arrange & Act
        let response = GitHubWebhookResponse::ignored("Unsupported event");

        // Assert
        assert_eq!(response.status, "ignored");
        assert_eq!(response.reason, Some("Unsupported event".to_string()));
    }

    #[test]
    fn should_deserialize_github_issues_payload() {
        // Arrange
        let json = r#"{
            "action": "labeled",
            "issue": {
                "number": 123,
                "title": "Test Issue",
                "body": "Issue body",
                "labels": [{"name": "ai-fix"}],
                "user": {"login": "testuser"}
            },
            "label": {"name": "ai-fix"},
            "repository": {"full_name": "org/repo"},
            "sender": {"login": "testuser"}
        }"#;

        // Act
        let payload: IssuesPayload = serde_json::from_str(json).expect("Failed to parse");

        // Assert
        assert_eq!(payload.action, "labeled");
        assert_eq!(payload.issue.number, 123);
        assert!(payload.has_label("ai-fix"));
        assert!(payload.is_adding_label("ai-fix"));
    }

    #[test]
    fn should_deserialize_github_issue_comment_payload() {
        // Arrange
        let json = r#"{
            "action": "created",
            "issue": {
                "number": 123,
                "title": "Test Issue",
                "labels": [],
                "user": {"login": "testuser"}
            },
            "comment": {
                "id": 456,
                "body": "@ai-bot analyze this issue",
                "user": {"login": "developer"}
            },
            "repository": {"full_name": "org/repo"},
            "sender": {"login": "developer"}
        }"#;

        // Act
        let payload: IssueCommentPayload = serde_json::from_str(json).expect("Failed to parse");

        // Assert
        assert_eq!(payload.action, "created");
        assert!(payload.mentions_ai_bot());
        assert!(payload.parse_ai_command().is_some());
    }

    #[test]
    fn should_deserialize_github_pull_request_payload() {
        // Arrange
        let json = r#"{
            "action": "opened",
            "pull_request": {
                "number": 456,
                "title": "Fix: Bug fix",
                "body": "PR description",
                "labels": [{"name": "ai-review"}],
                "head": {"ref": "fix/bug", "sha": "abc123"},
                "base": {"ref": "main", "sha": "def456"},
                "user": {"login": "developer"}
            },
            "repository": {"full_name": "org/repo"},
            "sender": {"login": "developer"}
        }"#;

        // Act
        let payload: PullRequestPayload = serde_json::from_str(json).expect("Failed to parse");

        // Assert
        assert_eq!(payload.action, "opened");
        assert_eq!(payload.pull_request.number, 456);
        assert!(payload.has_label("ai-review"));
    }
}
