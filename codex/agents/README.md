# Codex Agent Suite

A comprehensive suite of AI agents for automated software development, from requirements generation to code review and debugging.

## Overview

The Codex Agent Suite consists of five specialized agents that work together to automate the entire software development lifecycle:

- **Spec Agent** - Generates requirements, tests, and acceptance criteria from prompts
- **Code Agent** - Writes code changes with file-level edit streams
- **Test Generator Agent** - Creates unit/integration/e2e tests for changes
- **Reviewer Agent** - Automated code review with annotated diffs
- **Debug Agent** - Analyzes logs and suggests fixes with patch candidates

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Spec Agent    │───▶│   Code Agent    │───▶│  Reviewer Agent │
│                 │    │                 │    │                 │
│ Requirements    │    │ Code Changes    │    │ Code Review     │
│ Tests           │    │ File Edits      │    │ Annotations     │
│ Acceptance      │    │ Streaming       │    │ Findings        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Test Generator  │    │   Debug Agent   │    │   Integration   │
│                 │    │                 │    │                 │
│ Unit Tests      │    │ Log Analysis    │    │   Orchestrator  │
│ Integration     │    │ Error Detection │    │                 │
│ E2E Tests       │    │ Fix Suggestions │    │   GitHub App    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Quick Start

```rust
use codex_agents::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize model provider (OpenAI, Anthropic, etc.)
    let model_provider = Box::new(OpenAIModelProvider::new(
        "your-api-key".to_string(),
        "gpt-4".to_string(),
    ));

    // Create agent suite
    let agent_suite = AgentSuite::new(
        SpecAgent::new(model_provider.clone()),
        CodeAgent::new(model_provider.clone()),
        TestGeneratorAgent::new(model_provider.clone()),
        ReviewerAgent::new(model_provider.clone()),
        DebugAgent::new(model_provider),
    );

    // Orchestrate full development workflow
    let request = AgentOrchestrationRequest {
        prompt: "Add user authentication with email/password".to_string(),
        context: Some(HashMap::from([
            ("project_type".to_string(), "web-app".to_string()),
            ("language".to_string(), "javascript".to_string()),
        ])),
        agent_sequence: None, // Use default sequence
        options: None,
    };

    let response = agent_suite.orchestrate(request).await?;

    println!("Execution completed in {}ms", response.metadata.total_execution_time_ms);
    println!("Success rate: {:.1}%", response.metadata.success_rate);

    for execution in response.execution_order {
        println!("{:?}: {}ms - {}",
            execution.agent_type,
            execution.execution_time_ms,
            if execution.success { "✓" } else { "✗" }
        );
    }

    Ok(())
}
```

## Individual Agent Usage

### Spec Agent

Generates comprehensive specifications from prompts:

```rust
use codex_spec_agent::{SpecAgent, SpecRequest};

let spec_agent = SpecAgent::new(model_provider);
let request = SpecRequest {
    prompt: "Implement user authentication".to_string(),
    context: Some(HashMap::from([("project_type".to_string(), "web".to_string())])),
    project_type: Some("web-app".to_string()),
    existing_requirements: None,
};

let response = spec_agent.generate_spec(request).await?;
// Contains: requirements, test_cases, user_stories, acceptance_criteria
```

### Code Agent

Generates code changes with streaming support:

```rust
use codex_code_agent::{CodeAgent, CodeRequest};

let code_agent = CodeAgent::new(model_provider);
let request = CodeRequest {
    prompt: "Add login functionality".to_string(),
    context: None,
    requirements: Some(vec!["REQ-001".to_string()]),
    existing_files: Some(vec![/* existing file contents */]),
    target_files: Some(vec!["src/auth.js".to_string()]),
};

let response = code_agent.generate_code(request).await?;
// Contains: code changes, metadata, dependencies, warnings
```

### Test Generator Agent

Creates comprehensive test suites:

```rust
use codex_test_generator_agent::{TestGeneratorAgent, TestRequest};

let test_agent = TestGeneratorAgent::new(model_provider);
let request = TestRequest {
    code_changes: vec![/* generated code changes */],
    requirements: Some(vec!["REQ-001".to_string()]),
    existing_tests: None,
    test_framework: Some("jest".to_string()),
    coverage_goals: Some(vec!["80%".to_string()]),
};

let response = test_agent.generate_tests(request).await?;
// Contains: unit, integration, e2e tests with metadata
```

### Reviewer Agent

Automated code review with findings and annotations:

```rust
use codex_reviewer_agent::{ReviewerAgent, ReviewRequest, ReviewFocus};

let reviewer = ReviewerAgent::new(model_provider);
let request = ReviewRequest {
    code_changes: vec![/* code changes to review */],
    requirements: None,
    context: None,
    review_focus: Some(vec![
        ReviewFocus::Security,
        ReviewFocus::Performance,
        ReviewFocus::BestPractices,
    ]),
};

let response = reviewer.review_changes(request).await?;
// Contains: findings, annotated diffs, summary, recommendations
```

### Debug Agent

Analyzes logs and generates fix suggestions:

```rust
use codex_debug_agent::{DebugAgent, DebugRequest, LogEntry, LogLevel};

let debug_agent = DebugAgent::new(model_provider);
let request = DebugRequest {
    logs: vec![
        LogEntry {
            timestamp: "2024-01-01T10:00:00Z".to_string(),
            level: LogLevel::Error,
            message: "Connection timeout".to_string(),
            source: "database".to_string(),
            context: None,
        }
    ],
    error_context: None,
    codebase_files: Some(vec![/* relevant source files */]),
    recent_changes: None,
    debug_focus: Some(vec![DebugFocus::ErrorAnalysis]),
};

let response = debug_agent.analyze_logs(request).await?;
// Contains: analysis, patch suggestions, monitoring recommendations
```

## Agent Orchestration

The `AgentSuite` provides intelligent orchestration:

### Default Sequence
1. **Spec Agent** - Generate requirements and specifications
2. **Code Agent** - Generate code based on specifications
3. **Reviewer Agent** - Review generated code for issues
4. **Test Generator** - Create tests for the implementation

### Custom Sequences

```rust
let request = AgentOrchestrationRequest {
    prompt: "Fix authentication bug".to_string(),
    context: None,
    agent_sequence: Some(vec![
        AgentType::Debug,      // First analyze logs
        AgentType::Code,       // Generate fixes
        AgentType::Reviewer,   // Review fixes
        AgentType::TestGenerator, // Update tests
    ]),
    options: None,
};
```

## Model Providers

### OpenAI Provider

```rust
use codex_agents::*;

struct OpenAIModelProvider {
    api_key: String,
    model: String,
}

impl ModelProvider for OpenAIModelProvider {
    async fn generate_completion(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Implement OpenAI API call
        todo!("Implement OpenAI API integration")
    }

    async fn generate_with_context(&self, system_prompt: &str, user_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Implement OpenAI API call with system/user messages
        todo!("Implement OpenAI API integration")
    }
}
```

### Custom Providers

Implement the `ModelProvider` trait for other AI services:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn generate_completion(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>>;
    async fn generate_with_context(&self, system_prompt: &str, user_prompt: &str) -> Result<String, Box<dyn std::error::Error>>;
}
```

## Configuration

### Environment Variables

```bash
# OpenAI Configuration
OPENAI_API_KEY=your-api-key
OPENAI_MODEL=gpt-4

# Agent-specific settings
CODEX_SPEC_AGENT_MODEL=gpt-4
CODEX_CODE_AGENT_MODEL=gpt-4
CODEX_REVIEWER_AGENT_MODEL=gpt-3.5-turbo
```

### Agent-Specific Options

```rust
let options = HashMap::from([
    ("test_framework".to_string(), "jest".to_string()),
    ("review_focus".to_string(), "security,performance".to_string()),
    ("coverage_goals".to_string(), "80%".to_string()),
]);
```

## Error Handling

Agents provide detailed error information:

```rust
match agent_suite.orchestrate(request).await {
    Ok(response) => {
        if response.metadata.success_rate < 90.0 {
            println!("Warning: Low success rate: {:.1}%", response.metadata.success_rate);
            for warning in response.metadata.warnings {
                println!("Warning: {}", warning);
            }
        }
    }
    Err(e) => {
        eprintln!("Orchestration failed: {}", e);
    }
}
```

## Performance

- **Streaming Support**: Code agent supports real-time streaming
- **Parallel Execution**: Agents can run concurrently where dependencies allow
- **Caching**: Intelligent caching of intermediate results
- **Incremental Updates**: Only regenerate affected components

## Integration

### With Codex Backend

```rust
// In your backend service
let agent_suite = Arc::new(AgentSuite::new(/* agents */));

// Handle orchestration requests
let response = agent_suite.orchestrate(request).await?;
```

### With GitHub Integration

```rust
// Generate PR description from agent outputs
let pr_description = format!(
    "## Generated Changes\n\n{}\n\n## Tests Added\n\n{}",
    code_response.changes.len(),
    test_response.tests.len()
);
```

## Monitoring & Observability

### Metrics

- Execution time per agent
- Success/failure rates
- Token usage and costs
- Quality scores (reviewer agent)

### Logging

```rust
// Structured logging for agent executions
tracing::info!(
    agent = ?execution.agent_type,
    duration_ms = execution.execution_time_ms,
    success = execution.success,
    "Agent execution completed"
);
```

## Security

- **Input Validation**: All agent inputs are validated and sanitized
- **Output Filtering**: Generated code is scanned for security issues
- **API Key Management**: Secure credential handling
- **Audit Logging**: All agent executions are logged for compliance

## Contributing

### Adding New Agents

1. Create new agent in `codex/agents/{agent-name}/`
2. Implement the agent trait and request/response types
3. Add to `AgentSuite::new()` and orchestration logic
4. Update documentation and tests

### Testing

```bash
# Run all agent tests
cargo test --package codex-agents

# Run specific agent tests
cargo test --package codex-spec-agent
cargo test --package codex-code-agent
```

## License

This Agent Suite is part of the Codex project and follows the same licensing terms.
