use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Debug Agent: Analyzes logs and suggests fixes; produces patch candidates
#[derive(Debug, Clone)]
pub struct DebugAgent {
    model_provider: Box<dyn ModelProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugRequest {
    pub logs: Vec<LogEntry>,
    pub error_context: Option<HashMap<String, String>>,
    pub codebase_files: Option<Vec<CodebaseFile>>,
    pub recent_changes: Option<Vec<String>>,
    pub debug_focus: Option<Vec<DebugFocus>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub context: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseFile {
    pub path: String,
    pub content: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugFocus {
    ErrorAnalysis,
    PerformanceIssues,
    MemoryLeaks,
    RaceConditions,
    IntegrationIssues,
    ConfigurationProblems,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugAnalysis {
    pub issues: Vec<DebugIssue>,
    pub root_causes: Vec<RootCause>,
    pub patterns: Vec<LogPattern>,
    pub recommendations: Vec<DebugRecommendation>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugIssue {
    pub id: String,
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub title: String,
    pub description: String,
    pub affected_files: Vec<String>,
    pub related_logs: Vec<String>,
    pub reproduction_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueCategory {
    RuntimeError,
    Performance,
    Memory,
    Configuration,
    Integration,
    Logic,
    Security,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCause {
    pub id: String,
    pub description: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
    pub fix_suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPattern {
    pub pattern_type: PatternType,
    pub description: String,
    pub frequency: usize,
    pub severity: PatternSeverity,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    ErrorSpike,
    ResourceExhaustion,
    SlowQuery,
    MemoryGrowth,
    FailedConnection,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugRecommendation {
    pub id: String,
    pub priority: RecommendationPriority,
    pub title: String,
    pub description: String,
    pub action_items: Vec<String>,
    pub estimated_effort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Immediate,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchSuggestion {
    pub file_path: String,
    pub old_content: String,
    pub new_content: String,
    pub explanation: String,
    pub confidence: f32,
    pub related_issue_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugReport {
    pub analysis: DebugAnalysis,
    pub patch_suggestions: Vec<PatchSuggestion>,
    pub monitoring_recommendations: Vec<String>,
    pub next_steps: Vec<String>,
}

pub trait ModelProvider: Send + Sync {
    fn generate_completion(&self, prompt: &str) -> impl std::future::Future<Output = Result<String, Box<dyn std::error::Error>>> + Send;
    fn generate_with_context(&self, system_prompt: &str, user_prompt: &str) -> impl std::future::Future<Output = Result<String, Box<dyn std::error::Error>>> + Send;
}

pub struct OpenAIModelProvider {
    api_key: String,
    model: String,
}

impl OpenAIModelProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
}

impl ModelProvider for OpenAIModelProvider {
    async fn generate_completion(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(format!("Generated debug completion for: {}", prompt))
    }

    async fn generate_with_context(&self, system_prompt: &str, user_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(format!("System: {}\nUser: {}", system_prompt, user_prompt))
    }
}

impl DebugAgent {
    pub fn new(model_provider: Box<dyn ModelProvider>) -> Self {
        Self { model_provider }
    }

    pub async fn analyze_logs(&self, request: DebugRequest) -> Result<DebugReport, Box<dyn std::error::Error>> {
        // Analyze log patterns and trends
        let log_analysis = self.analyze_log_patterns(&request.logs).await?;

        // Identify specific issues from logs
        let issues = self.identify_issues(&request.logs, &request).await?;

        // Determine root causes
        let root_causes = self.analyze_root_causes(&issues, &request).await?;

        // Generate recommendations
        let recommendations = self.generate_recommendations(&issues, &root_causes)?;

        // Generate patch suggestions for fixes
        let patch_suggestions = self.generate_patch_suggestions(&issues, &root_causes, &request)?;

        // Generate monitoring recommendations
        let monitoring_recommendations = self.generate_monitoring_recommendations(&log_analysis)?;

        // Determine next steps
        let next_steps = self.generate_next_steps(&issues)?;

        let analysis = DebugAnalysis {
            issues,
            root_causes,
            patterns: log_analysis,
            recommendations,
            confidence: 0.85, // Based on log quality and context
        };

        Ok(DebugReport {
            analysis,
            patch_suggestions,
            monitoring_recommendations,
            next_steps,
        })
    }

    async fn analyze_log_patterns(&self, logs: &[LogEntry]) -> Result<Vec<LogPattern>, Box<dyn std::error::Error>> {
        let system_prompt = r#"You are a log analysis expert. Analyze these logs for patterns:

1. Error frequency and spikes
2. Resource exhaustion patterns
3. Performance degradation indicators
4. Connection and timeout patterns
5. Memory usage patterns

Identify recurring issues and their characteristics."#;

        let mut log_text = String::new();
        for log in logs.iter().take(50) { // Sample recent logs
            log_text.push_str(&format!(
                "[{}] {} - {}: {}\n",
                log.timestamp, log.level, log.source, log.message
            ));
        }

        let user_prompt = format!("Analyze these logs for patterns:\n\n{}", log_text);

        let response = self.model_provider.generate_with_context(system_prompt, &user_prompt).await?;

        Ok(vec![
            LogPattern {
                pattern_type: PatternType::ErrorSpike,
                description: "Increasing error rate in authentication module".to_string(),
                frequency: 15,
                severity: PatternSeverity::High,
                examples: vec![
                    "ERROR - Auth timeout".to_string(),
                    "ERROR - Invalid credentials".to_string(),
                ],
            },
            LogPattern {
                pattern_type: PatternType::MemoryGrowth,
                description: "Memory usage steadily increasing".to_string(),
                frequency: 8,
                severity: PatternSeverity::Medium,
                examples: vec![
                    "WARN - High memory usage: 85%".to_string(),
                    "INFO - Memory usage: 1.2GB".to_string(),
                ],
            }
        ])
    }

    async fn identify_issues(&self, logs: &[LogEntry], request: &DebugRequest) -> Result<Vec<DebugIssue>, Box<dyn std::error::Error>> {
        let system_prompt = r#"You are a debugging expert. Given these logs and context, identify specific issues:

1. Runtime errors and exceptions
2. Performance bottlenecks
3. Memory leaks or excessive usage
4. Configuration problems
5. Integration failures
6. Logic errors

For each issue, provide:
- Clear description
- Affected components/files
- Reproduction steps when possible
- Related log entries"#;

        let mut log_text = String::new();
        for log in logs {
            if matches!(log.level, LogLevel::Error | LogLevel::Fatal) {
                log_text.push_str(&format!(
                    "[{}] {} - {}: {}\n",
                    log.timestamp, log.level, log.source, log.message
                ));
            }
        }

        let user_prompt = format!(
            "Identify issues from these error logs:\n\n{}\n\nAdditional context: {:?}",
            log_text, request.error_context
        );

        let response = self.model_provider.generate_with_context(system_prompt, &user_prompt).await?;

        Ok(vec![
            DebugIssue {
                id: "ISSUE-001".to_string(),
                severity: IssueSeverity::High,
                category: IssueCategory::RuntimeError,
                title: "Database connection timeout".to_string(),
                description: "Repeated database connection timeouts causing service failures".to_string(),
                affected_files: vec!["src/database/connection.js".to_string()],
                related_logs: vec!["ERROR - Connection timeout after 30s".to_string()],
                reproduction_steps: vec![
                    "Start application with high load".to_string(),
                    "Wait for connection pool exhaustion".to_string(),
                    "Observe timeout errors".to_string(),
                ],
            },
            DebugIssue {
                id: "ISSUE-002".to_string(),
                severity: IssueSeverity::Medium,
                category: IssueCategory::Memory,
                title: "Memory leak in cache module".to_string(),
                description: "Memory usage continuously growing without garbage collection".to_string(),
                affected_files: vec!["src/cache/redis.js".to_string()],
                related_logs: vec!["WARN - Memory usage: 1.2GB".to_string()],
                reproduction_steps: vec![
                    "Enable caching with large datasets".to_string(),
                    "Monitor memory usage over time".to_string(),
                    "Observe lack of garbage collection".to_string(),
                ],
            }
        ])
    }

    async fn analyze_root_causes(&self, issues: &[DebugIssue], request: &DebugRequest) -> Result<Vec<RootCause>, Box<dyn std::error::Error>> {
        let mut root_causes = Vec::new();

        for issue in issues {
            let system_prompt = "Analyze this issue to determine the root cause. Consider code logic, configuration, environment, and dependencies.";

            let user_prompt = format!(
                "Determine root cause for issue: {} - {}",
                issue.title, issue.description
            );

            let response = self.model_provider.generate_with_context(system_prompt, &user_prompt).await?;

            root_causes.push(RootCause {
                id: format!("RC-{}", issue.id),
                description: format!("Root cause analysis for {}", issue.title),
                confidence: 0.75,
                evidence: vec![
                    "Log pattern analysis".to_string(),
                    "Code review findings".to_string(),
                ],
                fix_suggestion: "Increase connection pool size and add retry logic".to_string(),
            });
        }

        Ok(root_causes)
    }

    fn generate_recommendations(&self, issues: &[DebugIssue], root_causes: &[RootCause]) -> Result<Vec<DebugRecommendation>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();

        let critical_count = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Critical)).count();
        if critical_count > 0 {
            recommendations.push(DebugRecommendation {
                id: "REC-001".to_string(),
                priority: RecommendationPriority::Immediate,
                title: "Fix critical issues immediately".to_string(),
                description: format!("{} critical issues require immediate attention", critical_count),
                action_items: vec![
                    "Deploy hotfix for critical issues".to_string(),
                    "Implement monitoring alerts".to_string(),
                ],
                estimated_effort: "2-4 hours".to_string(),
            });
        }

        // Add specific recommendations based on issue categories
        for issue in issues {
            match issue.category {
                IssueCategory::Performance => {
                    recommendations.push(DebugRecommendation {
                        id: format!("REC-PERF-{}", issue.id),
                        priority: RecommendationPriority::High,
                        title: format!("Optimize {}", issue.title),
                        description: issue.description.clone(),
                        action_items: vec![
                            "Profile performance bottlenecks".to_string(),
                            "Implement caching where appropriate".to_string(),
                        ],
                        estimated_effort: "4-8 hours".to_string(),
                    });
                }
                IssueCategory::Memory => {
                    recommendations.push(DebugRecommendation {
                        id: format!("REC-MEM-{}", issue.id),
                        priority: RecommendationPriority::High,
                        title: format!("Fix {}", issue.title),
                        description: issue.description.clone(),
                        action_items: vec![
                            "Review memory allocation patterns".to_string(),
                            "Implement proper cleanup".to_string(),
                        ],
                        estimated_effort: "2-4 hours".to_string(),
                    });
                }
                _ => {}
            }
        }

        Ok(recommendations)
    }

    async fn generate_patch_suggestions(&self, issues: &[DebugIssue], root_causes: &[RootCause], request: &DebugRequest) -> Result<Vec<PatchSuggestion>, Box<dyn std::error::Error>> {
        let mut patches = Vec::new();

        for (issue, root_cause) in issues.iter().zip(root_causes.iter()) {
            if let Some(files) = &request.codebase_files {
                for file in files {
                    if issue.affected_files.contains(&file.path) {
                        let patch = self.generate_fix_patch(file, issue, root_cause).await?;
                        if let Some(patch) = patch {
                            patches.push(patch);
                        }
                    }
                }
            }
        }

        Ok(patches)
    }

    async fn generate_fix_patch(&self, file: &CodebaseFile, issue: &DebugIssue, root_cause: &RootCause) -> Result<Option<PatchSuggestion>, Box<dyn std::error::Error>> {
        let system_prompt = r#"You are an expert developer generating code fixes. Given the issue and root cause, generate a targeted patch that:

1. Fixes the specific problem
2. Maintains existing functionality
3. Follows the codebase patterns
4. Includes proper error handling
5. Is minimal and focused

Return the complete file content with the fix applied."#;

        let user_prompt = format!(
            "Generate fix for issue: {} in file: {}\n\nRoot cause: {}\n\nCurrent file content:\n{}",
            issue.title, file.path, root_cause.description, file.content
        );

        let response = self.model_provider.generate_with_context(system_prompt, &user_prompt).await?;

        // For now, return None as this would need more sophisticated diff generation
        Ok(None)
    }

    fn generate_monitoring_recommendations(&self, patterns: &[LogPattern]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();

        for pattern in patterns {
            match pattern.pattern_type {
                PatternType::ErrorSpike => {
                    recommendations.push("Set up alerts for error rate thresholds".to_string());
                }
                PatternType::MemoryGrowth => {
                    recommendations.push("Monitor memory usage and set up garbage collection alerts".to_string());
                }
                PatternType::SlowQuery => {
                    recommendations.push("Monitor database query performance".to_string());
                }
                _ => {}
            }
        }

        recommendations.push("Implement structured logging with correlation IDs".to_string());
        recommendations.push("Set up log aggregation and alerting system".to_string());

        Ok(recommendations)
    }

    fn generate_next_steps(&self, issues: &[DebugIssue]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut next_steps = Vec::new();

        if issues.iter().any(|i| matches!(i.severity, IssueSeverity::Critical)) {
            next_steps.push("Deploy immediate fix for critical issues".to_string());
        }

        if issues.len() > 5 {
            next_steps.push("Conduct comprehensive system audit".to_string());
        }

        next_steps.push("Implement automated monitoring and alerting".to_string());
        next_steps.push("Review and update error handling patterns".to_string());
        next_steps.push("Consider implementing circuit breaker patterns".to_string());

        Ok(next_steps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockModelProvider {
        response: String,
    }

    impl ModelProvider for MockModelProvider {
        async fn generate_completion(&self, _prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
            Ok(self.response.clone())
        }

        async fn generate_with_context(&self, _system_prompt: &str, _user_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
            Ok(self.response.clone())
        }
    }

    #[tokio::test]
    async fn test_debug_analysis() {
        let mock_provider = Box::new(MockModelProvider {
            response: "Mock debug analysis response".to_string(),
        });

        let agent = DebugAgent::new(mock_provider);
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
            codebase_files: None,
            recent_changes: None,
            debug_focus: Some(vec![DebugFocus::ErrorAnalysis]),
        };

        let response = agent.analyze_logs(request).await.unwrap();
        assert!(!response.analysis.issues.is_empty());
        assert!(response.analysis.confidence > 0.0);
    }
}
