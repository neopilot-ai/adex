use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub app_id: String,
    pub private_key_path: String,
    pub webhook_secret: String,
    pub base_url: Option<String>, // For GitHub Enterprise
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBranchRequest {
    pub repo_owner: String,
    pub repo_name: String,
    pub branch_name: String,
    pub base_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBranchResponse {
    pub branch_ref: String,
    pub object_sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommitRequest {
    pub repo_owner: String,
    pub repo_name: String,
    pub branch_name: String,
    pub message: String,
    pub changes: Vec<FileChange>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub content: String,
    pub encoding: Option<String>, // "utf-8" or "base64"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommitResponse {
    pub commit_sha: String,
    pub tree_sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePullRequestRequest {
    pub repo_owner: String,
    pub repo_name: String,
    pub title: String,
    pub body: String,
    pub head_branch: String,
    pub base_branch: String,
    pub draft: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePullRequestResponse {
    pub pr_number: u32,
    pub pr_url: String,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestEvent {
    pub action: String,
    pub number: u32,
    pub pull_request: PullRequestData,
    pub repository: RepositoryData,
    pub sender: UserData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestData {
    pub id: u64,
    pub number: u32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
    pub base: BranchData,
    pub head: BranchData,
    pub user: UserData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchData {
    pub ref_name: String,
    pub sha: String,
    pub repo: RepositoryData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryData {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub html_url: String,
    pub clone_url: String,
    pub ssh_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub id: u64,
    pub login: String,
    pub html_url: String,
}

pub struct GitHubApp {
    config: GitHubConfig,
    client: reqwest::Client,
}

impl GitHubApp {
    pub fn new(config: GitHubConfig) -> Self {
        let client = reqwest::Client::new();
        Self { config, client }
    }

    /// Create a new branch from the specified base branch
    pub async fn create_branch(&self, request: CreateBranchRequest) -> Result<CreateBranchResponse, Box<dyn std::error::Error>> {
        // Implementation would:
        // 1. Get the base branch SHA
        // 2. Create a new ref pointing to that SHA
        // For now, return a placeholder response
        Ok(CreateBranchResponse {
            branch_ref: format!("refs/heads/{}", request.branch_name),
            object_sha: "dummy_sha".to_string(),
        })
    }

    /// Create a commit with the given file changes
    pub async fn create_commit(&self, request: CreateCommitRequest) -> Result<CreateCommitResponse, Box<dyn std::error::Error>> {
        // Implementation would:
        // 1. Get the current tree of the branch
        // 2. Create blobs for file changes
        // 3. Create a new tree with the blobs
        // 4. Create a commit pointing to the new tree
        Ok(CreateCommitResponse {
            commit_sha: "dummy_commit_sha".to_string(),
            tree_sha: "dummy_tree_sha".to_string(),
        })
    }

    /// Create a pull request
    pub async fn create_pull_request(&self, request: CreatePullRequestRequest) -> Result<CreatePullRequestResponse, Box<dyn std::error::Error>> {
        // Implementation would:
        // 1. Create the PR via GitHub API
        // 2. Return the PR details
        Ok(CreatePullRequestResponse {
            pr_number: 123,
            pr_url: format!("https://api.github.com/repos/{}/{}/pulls/123", request.repo_owner, request.repo_name),
            html_url: format!("https://github.com/{}/{}/pull/123", request.repo_owner, request.repo_name),
        })
    }

    /// Handle incoming webhook events
    pub async fn handle_webhook(&self, payload: &[u8], signature: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Implementation would:
        // 1. Verify the webhook signature
        // 2. Parse the event type and handle accordingly
        // 3. For PR events, potentially trigger codex workflows

        // For now, just log the event type
        if let Ok(event) = serde_json::from_slice::<serde_json::Value>(payload) {
            if let Some(action) = event.get("action").and_then(|a| a.as_str()) {
                println!("GitHub webhook event: {} - {}", event.get("event_type").unwrap_or(&serde_json::Value::Null), action);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_config_serialization() {
        let config = GitHubConfig {
            app_id: "12345".to_string(),
            private_key_path: "/path/to/key.pem".to_string(),
            webhook_secret: "secret".to_string(),
            base_url: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: GitHubConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.app_id, deserialized.app_id);
        assert_eq!(config.private_key_path, deserialized.private_key_path);
    }
}
