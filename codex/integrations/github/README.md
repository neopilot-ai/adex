# Codex GitHub Integration

This crate provides GitHub App integration for Codex, allowing it to:

- Create branches automatically
- Push commits with generated code changes
- Create pull requests with proper descriptions
- Handle GitHub webhooks for PR events

## Features

### Branch Management
- Create feature branches from main/master
- Support for custom base branches

### Commit Creation
- Create commits with multiple file changes
- Proper author attribution
- Atomic operations

### Pull Request Management
- Create PRs with generated titles and descriptions
- Support for draft PRs
- Link to session recordings

### Webhook Handling
- Verify webhook signatures
- Handle PR events (opened, closed, merged, etc.)
- Trigger Codex workflows based on GitHub events

## Usage

```rust
use codex_github_integration::{GitHubApp, GitHubConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = GitHubConfig {
        app_id: "your-app-id".to_string(),
        private_key_path: "/path/to/github-app-private-key.pem".to_string(),
        webhook_secret: "your-webhook-secret".to_string(),
        base_url: None, // Use default GitHub API
    };

    let github_app = GitHubApp::new(config);

    // Create a feature branch
    let branch_request = CreateBranchRequest {
        repo_owner: "myorg".to_string(),
        repo_name: "myrepo".to_string(),
        branch_name: "codex/feature/amazing-feature".to_string(),
        base_branch: "main".to_string(),
    };

    let branch_response = github_app.create_branch(branch_request).await?;
    println!("Created branch: {}", branch_response.branch_ref);

    Ok(())
}
```

## Setup

### GitHub App Configuration

1. Create a GitHub App at https://github.com/settings/apps
2. Set the following permissions:
   - Repository contents: Read & Write
   - Pull requests: Read & Write
   - Metadata: Read-only
3. Generate a private key and download it
4. Set the webhook URL to your Codex server's webhook endpoint
5. Subscribe to the following events:
   - Pull requests
   - Push

### Environment Variables

```bash
export GITHUB_APP_ID="your-app-id"
export GITHUB_PRIVATE_KEY_PATH="/path/to/private-key.pem"
export GITHUB_WEBHOOK_SECRET="your-webhook-secret"
```

## API Reference

### Types

- `GitHubConfig`: Configuration for the GitHub App
- `CreateBranchRequest`: Parameters for creating a branch
- `CreateCommitRequest`: Parameters for creating a commit
- `CreatePullRequestRequest`: Parameters for creating a PR
- `PullRequestEvent`: Structure for webhook PR events

### Methods

- `GitHubApp::new(config)`: Create a new GitHub App instance
- `create_branch(request)`: Create a new branch
- `create_commit(request)`: Create a commit with file changes
- `create_pull_request(request)`: Create a pull request
- `handle_webhook(payload, signature)`: Process incoming webhooks

## Webhook Event Handling

The integration can handle various GitHub webhook events:

- `pull_request.opened`: Trigger code review or additional checks
- `pull_request.closed`: Archive or cleanup resources
- `pull_request.merged`: Trigger deployment workflows
- `push`: Handle commits to tracked branches

## Security

- All webhook payloads are verified using HMAC-SHA256
- Private keys are loaded from secure file paths
- API tokens are not logged in production

## Error Handling

The crate provides detailed error messages for common issues:
- Invalid signatures
- Missing permissions
- Network connectivity issues
- Rate limiting
