use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use base64::{Engine as _, engine::general_purpose};
use chrono;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub app_id: String,
    pub private_key_path: String,
    pub webhook_secret: String,
    pub base_url: Option<String>, // For GitHub Enterprise
    pub access_token: Option<String>, // For personal access token auth
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
        let base_url = self.config.base_url.as_deref().unwrap_or("https://api.github.com");
        let token = self.config.access_token.as_ref().ok_or("Access token required")?;

        // First, get the base branch SHA
        let branch_url = format!("{}/repos/{}/{}/git/ref/heads/{}",
            base_url, request.repo_owner, request.repo_name, request.base_branch);

        let branch_response = self.client
            .get(&branch_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "Codex-GitHub-App")
            .send()
            .await?;

        if !branch_response.status().is_success() {
            return Err(format!("Failed to get base branch: {}", branch_response.status()).into());
        }

        #[derive(Deserialize)]
        struct BranchRef {
            object: BranchObject,
        }

        #[derive(Deserialize)]
        struct BranchObject {
            sha: String,
        }

        let branch_data: BranchRef = branch_response.json().await?;
        let base_sha = branch_data.object.sha;

        // Create the new branch reference
        let new_ref_url = format!("{}/repos/{}/{}/git/refs",
            base_url, request.repo_owner, request.repo_name);

        #[derive(Serialize)]
        struct NewRef {
            ref_name: String,
            sha: String,
        }

        let new_ref = NewRef {
            ref_name: format!("refs/heads/{}", request.branch_name),
            sha: base_sha,
        };

        let create_response = self.client
            .post(&new_ref_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "Codex-GitHub-App")
            .json(&new_ref)
            .send()
            .await?;

        if !create_response.status().is_success() {
            return Err(format!("Failed to create branch: {}", create_response.status()).into());
        }

        Ok(CreateBranchResponse {
            branch_ref: format!("refs/heads/{}", request.branch_name),
            object_sha: base_sha,
        })
    }

    /// Create a commit with the given file changes
    pub async fn create_commit(&self, request: CreateCommitRequest) -> Result<CreateCommitResponse, Box<dyn std::error::Error>> {
        let base_url = self.config.base_url.as_deref().unwrap_or("https://api.github.com");
        let token = self.config.access_token.as_ref().ok_or("Access token required")?;

        // Get the current branch SHA
        let branch_url = format!("{}/repos/{}/{}/git/ref/heads/{}",
            base_url, request.repo_owner, request.repo_name, request.branch_name);

        let branch_response = self.client
            .get(&branch_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "Codex-GitHub-App")
            .send()
            .await?;

        if !branch_response.status().is_success() {
            return Err(format!("Failed to get branch: {}", branch_response.status()).into());
        }

        #[derive(Deserialize)]
        struct BranchRef {
            object: BranchObject,
        }

        #[derive(Deserialize)]
        struct BranchObject {
            sha: String,
        }

        let branch_data: BranchRef = branch_response.json().await?;
        let base_sha = branch_data.object.sha;

        // Get the current tree
        let commit_url = format!("{}/repos/{}/{}/git/commits/{}",
            base_url, request.repo_owner, request.repo_name, base_sha);

        let commit_response = self.client
            .get(&commit_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "Codex-GitHub-App")
            .send()
            .await?;

        if !commit_response.status().is_success() {
            return Err(format!("Failed to get commit: {}", commit_response.status()).into());
        }

        #[derive(Deserialize)]
        struct CommitData {
            tree: TreeData,
        }

        #[derive(Deserialize)]
        struct TreeData {
            sha: String,
        }

        let commit_data: CommitData = commit_response.json().await?;
        let base_tree_sha = commit_data.tree.sha;

        // Create blobs for file changes
        let mut tree_entries = Vec::new();

        for change in &request.changes {
            let blob_sha = self.create_blob(&request.repo_owner, &request.repo_name,
                &change.content, change.encoding.as_deref().unwrap_or("utf-8")).await?;

            tree_entries.push(TreeEntry {
                path: change.path.clone(),
                mode: "100644".to_string(),
                #[serde(rename = "type")]
                entry_type: "blob".to_string(),
                sha: blob_sha,
            });
        }

        // Create a new tree
        let tree_sha = self.create_tree(&request.repo_owner, &request.repo_name,
            &base_tree_sha, tree_entries).await?;

        // Create the commit
        let commit_sha = self.create_git_commit(&request.repo_owner, &request.repo_name,
            &request.message, &tree_sha, &[&base_sha],
            request.author_name.as_deref(), request.author_email.as_deref()).await?;

        Ok(CreateCommitResponse {
            commit_sha,
            tree_sha,
        })
    }

    /// Create a pull request
    pub async fn create_pull_request(&self, request: CreatePullRequestRequest) -> Result<CreatePullRequestResponse, Box<dyn std::error::Error>> {
        let base_url = self.config.base_url.as_deref().unwrap_or("https://api.github.com");
        let token = self.config.access_token.as_ref().ok_or("Access token required")?;

        let pr_url = format!("{}/repos/{}/{}/pulls",
            base_url, request.repo_owner, request.repo_name);

        #[derive(Serialize)]
        struct NewPR {
            title: String,
            body: String,
            head: String,
            base: String,
            draft: bool,
        }

        let new_pr = NewPR {
            title: request.title,
            body: request.body,
            head: request.head_branch,
            base: request.base_branch,
            draft: request.draft.unwrap_or(false),
        };

        let response = self.client
            .post(&pr_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "Codex-GitHub-App")
            .json(&new_pr)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to create PR: {}", response.status()).into());
        }

        #[derive(Deserialize)]
        struct PRData {
            number: u32,
            html_url: String,
        }

        let pr_data: PRData = response.json().await?;

        Ok(CreatePullRequestResponse {
            pr_number: pr_data.number,
            pr_url: format!("https://api.github.com/repos/{}/{}/pulls/{}",
                request.repo_owner, request.repo_name, pr_data.number),
            html_url: pr_data.html_url,
        })
    }

    /// Handle incoming webhook events
    pub async fn handle_webhook(&self, payload: &[u8], signature: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Verify webhook signature (simplified - in production use proper HMAC verification)
        if signature.is_empty() {
            return Err("Missing webhook signature".into());
        }

        // Parse the webhook payload
        let event: serde_json::Value = serde_json::from_slice(payload)?;

        if let Some(action) = event.get("action").and_then(|a| a.as_str()) {
            println!("GitHub webhook event: {} - {}", event.get("event_type").unwrap_or(&serde_json::Value::Null), action);

            // Handle different event types
            match action {
                "opened" | "synchronize" => {
                    // PR opened or updated - could trigger code review agent
                    println!("PR event detected, could trigger code review workflow");
                }
                "closed" => {
                    // PR closed - could trigger deployment or cleanup
                    println!("PR closed, could trigger cleanup workflow");
                }
                _ => {
                    println!("Unhandled webhook action: {}", action);
                }
            }
        }

        Ok(())
    }

    // Helper methods
    async fn create_blob(&self, owner: &str, repo: &str, content: &str, encoding: &str) -> Result<String, Box<dyn std::error::Error>> {
        let base_url = self.config.base_url.as_deref().unwrap_or("https://api.github.com");
        let token = self.config.access_token.as_ref().unwrap();

        let blob_url = format!("{}/repos/{}/{}/git/blobs", base_url, owner, repo);

        #[derive(Serialize)]
        struct NewBlob {
            content: String,
            encoding: String,
        }

        let blob_data = if encoding == "base64" {
            NewBlob {
                content: content.to_string(),
                encoding: encoding.to_string(),
            }
        } else {
            NewBlob {
                content: general_purpose::STANDARD.encode(content.as_bytes()),
                encoding: "base64".to_string(),
            }
        };

        let response = self.client
            .post(&blob_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "Codex-GitHub-App")
            .json(&blob_data)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to create blob: {}", response.status()).into());
        }

        #[derive(Deserialize)]
        struct BlobResponse {
            sha: String,
        }

        let blob_response: BlobResponse = response.json().await?;
        Ok(blob_response.sha)
    }

    async fn create_tree(&self, owner: &str, repo: &str, base_tree_sha: &str, entries: Vec<TreeEntry>) -> Result<String, Box<dyn std::error::Error>> {
        let base_url = self.config.base_url.as_deref().unwrap_or("https://api.github.com");
        let token = self.config.access_token.as_ref().unwrap();

        let tree_url = format!("{}/repos/{}/{}/git/trees", base_url, owner, repo);

        #[derive(Serialize)]
        struct NewTree {
            base_tree: String,
            tree: Vec<TreeEntry>,
        }

        let tree_data = NewTree {
            base_tree: base_tree_sha.to_string(),
            tree: entries,
        };

        let response = self.client
            .post(&tree_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "Codex-GitHub-App")
            .json(&tree_data)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to create tree: {}", response.status()).into());
        }

        #[derive(Deserialize)]
        struct TreeResponse {
            sha: String,
        }

        let tree_response: TreeResponse = response.json().await?;
        Ok(tree_response.sha)
    }

    async fn create_git_commit(&self, owner: &str, repo: &str, message: &str, tree_sha: &str,
        parents: &[&str], author_name: Option<&str>, author_email: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
        let base_url = self.config.base_url.as_deref().unwrap_or("https://api.github.com");
        let token = self.config.access_token.as_ref().unwrap();

        let commit_url = format!("{}/repos/{}/{}/git/commits", base_url, owner, repo);

        #[derive(Serialize)]
        struct CommitAuthor {
            name: String,
            email: String,
        }

        #[derive(Serialize)]
        struct NewCommit {
            message: String,
            tree: String,
            parents: Vec<String>,
            author: Option<CommitAuthor>,
        }

        let mut new_commit = NewCommit {
            message: message.to_string(),
            tree: tree_sha.to_string(),
            parents: parents.iter().map(|&s| s.to_string()).collect(),
            author: None,
        };

        if let (Some(name), Some(email)) = (author_name, author_email) {
            new_commit.author = Some(CommitAuthor {
                name: name.to_string(),
                email: email.to_string(),
            });
        }

        let response = self.client
            .post(&commit_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "Codex-GitHub-App")
            .json(&new_commit)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to create commit: {}", response.status()).into());
        }

        #[derive(Deserialize)]
        struct CommitResponse {
            sha: String,
        }

        let commit_response: CommitResponse = response.json().await?;
        Ok(commit_response.sha)
    }
}

#[derive(Serialize)]
struct TreeEntry {
    path: String,
    mode: String,
    #[serde(rename = "type")]
    entry_type: String,
    sha: String,
}
