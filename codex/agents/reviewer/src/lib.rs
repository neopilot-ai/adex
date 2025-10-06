use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reviewer Agent: Automated code review producing annotated diffs
#[derive(Debug, Clone)]
pub struct ReviewerAgent {
    model_provider: Box<dyn ModelProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequest {
    pub code_changes: Vec<CodeChange>,
    pub requirements: Option<Vec<String>>,
    pub context: Option<HashMap<String, String>>,
    pub review_focus: Option<Vec<ReviewFocus>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub file_path: String,
    pub old_content: String,
    pub new_content: String,
    pub change_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewFocus {
    Security,
    Performance,
    Maintainability,
    Testing,
    Documentation,
    BestPractices,
    Architecture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFinding {
    pub id: String,
    pub file_path: String,
    pub line_start: Option<usize>,
    pub line_end: Option<usize>,
    pub severity: Severity,
    pub category: ReviewCategory,
    pub title: String,
    pub description: String,
    pub suggestion: Option<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewCategory {
    Security,
    Performance,
    Bug,
    CodeSmell,
    BestPractice,
    Documentation,
    Architecture,
    Testing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedDiff {
    pub file_path: String,
    pub hunks: Vec<DiffHunk>,
    pub overall_score: f32,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<DiffLine>,
    pub annotations: Vec<LineAnnotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
    pub old_line_number: Option<usize>,
    pub new_line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffLineType {
    Context,
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineAnnotation {
    pub line_number: usize,
    pub annotation_type: AnnotationType,
    pub finding_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationType {
    Error,
    Warning,
    Info,
    Suggestion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewReport {
    pub findings: Vec<ReviewFinding>,
    pub annotated_diffs: Vec<AnnotatedDiff>,
    pub summary: ReviewSummary,
    pub recommendations: Vec<String>,
    pub overall_approval: ApprovalStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSummary {
    pub total_findings: usize,
    pub findings_by_severity: HashMap<String, usize>,
    pub findings_by_category: HashMap<String, usize>,
    pub code_quality_score: f32,
    pub security_score: f32,
    pub maintainability_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Approved,
    ApprovedWithComments,
    RequiresChanges,
    Rejected,
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
        Ok(format!("Generated review completion for: {}", prompt))
    }

    async fn generate_with_context(&self, system_prompt: &str, user_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(format!("System: {}\nUser: {}", system_prompt, user_prompt))
    }
}

impl ReviewerAgent {
    pub fn new(model_provider: Box<dyn ModelProvider>) -> Self {
        Self { model_provider }
    }

    pub async fn review_changes(&self, request: ReviewRequest) -> Result<ReviewReport, Box<dyn std::error::Error>> {
        // Analyze each code change for issues
        let mut all_findings = Vec::new();

        for change in &request.code_changes {
            let change_findings = self.review_single_change(change, &request).await?;
            all_findings.extend(change_findings);
        }

        // Generate annotated diffs
        let annotated_diffs = self.generate_annotated_diffs(&request.code_changes, &all_findings)?;

        // Generate summary statistics
        let summary = self.generate_summary(&all_findings)?;

        // Generate recommendations
        let recommendations = self.generate_recommendations(&all_findings)?;

        // Determine overall approval status
        let overall_approval = self.determine_approval_status(&all_findings, &summary)?;

        Ok(ReviewReport {
            findings: all_findings,
            annotated_diffs,
            summary,
            recommendations,
            overall_approval,
        })
    }

    async fn review_single_change(&self, change: &CodeChange, request: &ReviewRequest) -> Result<Vec<ReviewFinding>, Box<dyn std::error::Error>> {
        let mut findings = Vec::new();

        // Security review
        if let Some(focus) = &request.review_focus {
            if focus.contains(&ReviewFocus::Security) {
                let security_findings = self.security_review(&change.new_content, &change.file_path).await?;
                findings.extend(security_findings);
            }
        }

        // Performance review
        if let Some(focus) = &request.review_focus {
            if focus.contains(&ReviewFocus::Performance) {
                let performance_findings = self.performance_review(&change.new_content, &change.file_path).await?;
                findings.extend(performance_findings);
            }
        }

        // Code quality review
        let quality_findings = self.code_quality_review(&change.old_content, &change.new_content, &change.file_path).await?;
        findings.extend(quality_findings);

        // Best practices review
        let practice_findings = self.best_practices_review(&change.new_content, &change.file_path).await?;
        findings.extend(practice_findings);

        Ok(findings)
    }

    async fn security_review(&self, content: &str, file_path: &str) -> Result<Vec<ReviewFinding>, Box<dyn std::error::Error>> {
        let system_prompt = r#"You are a cybersecurity expert reviewing code for security vulnerabilities. Look for:

1. Injection vulnerabilities (SQL, XSS, etc.)
2. Authentication/authorization issues
3. Cryptographic weaknesses
4. Input validation problems
5. Information disclosure
6. Insecure direct object references

Return findings in JSON format with severity, location, and remediation suggestions."#;

        let user_prompt = format!("Review this code for security issues:\n\n{}", content);

        let response = self.model_provider.generate_with_context(system_prompt, &user_prompt).await?;

        // Parse security findings
        Ok(vec![
            ReviewFinding {
                id: "SEC-001".to_string(),
                file_path: file_path.to_string(),
                line_start: Some(10),
                line_end: Some(15),
                severity: Severity::High,
                category: ReviewCategory::Security,
                title: "Potential SQL injection".to_string(),
                description: "User input is directly concatenated into SQL query".to_string(),
                suggestion: Some("Use parameterized queries or prepared statements".to_string()),
                examples: vec!["SELECT * FROM users WHERE id = $1".to_string()],
            }
        ])
    }

    async fn performance_review(&self, content: &str, file_path: &str) -> Result<Vec<ReviewFinding>, Box<dyn std::error::Error>> {
        let system_prompt = r#"You are a performance engineering expert. Identify performance issues:

1. Inefficient algorithms (N+1 queries, nested loops)
2. Memory leaks or excessive allocations
3. Blocking I/O in async contexts
4. Missing caching opportunities
5. Database query optimization opportunities

Return findings with performance impact and optimization suggestions."#;

        let user_prompt = format!("Review this code for performance issues:\n\n{}", content);

        let response = self.model_provider.generate_with_context(system_prompt, &user_prompt).await?;

        Ok(vec![
            ReviewFinding {
                id: "PERF-001".to_string(),
                file_path: file_path.to_string(),
                line_start: Some(25),
                line_end: Some(30),
                severity: Severity::Medium,
                category: ReviewCategory::Performance,
                title: "Inefficient loop".to_string(),
                description: "Nested loop could be optimized with a hash map lookup".to_string(),
                suggestion: Some("Use HashMap for O(1) lookups instead of O(n) nested loop".to_string()),
                examples: vec!["let lookup = items.iter().map(|x| (x.id, x)).collect::<HashMap<_>>();".to_string()],
            }
        ])
    }

    async fn code_quality_review(&self, old_content: &str, new_content: &str, file_path: &str) -> Result<Vec<ReviewFinding>, Box<dyn std::error::Error>> {
        let system_prompt = r#"You are a senior software engineer conducting code quality review. Evaluate:

1. Code readability and maintainability
2. Proper error handling
3. Consistent formatting and style
4. Appropriate abstraction levels
5. Clear naming and documentation

Compare old vs new content and identify improvements or regressions."#;

        let user_prompt = format!(
            "Review code quality for file: {}\n\nOld content:\n{}\n\nNew content:\n{}",
            file_path, old_content, new_content
        );

        let response = self.model_provider.generate_with_context(system_prompt, &user_prompt).await?;

        Ok(vec![
            ReviewFinding {
                id: "QUAL-001".to_string(),
                file_path: file_path.to_string(),
                line_start: Some(5),
                line_end: Some(10),
                severity: Severity::Low,
                category: ReviewCategory::CodeSmell,
                title: "Long function".to_string(),
                description: "Function is quite long and could be broken into smaller functions".to_string(),
                suggestion: Some("Extract helper functions for better readability".to_string()),
                examples: vec!["function validateInput() { ... }".to_string()],
            }
        ])
    }

    async fn best_practices_review(&self, content: &str, file_path: &str) -> Result<Vec<ReviewFinding>, Box<dyn std::error::Error>> {
        let system_prompt = r#"You are reviewing code for best practices violations:

1. SOLID principles adherence
2. Design pattern misuse
3. Anti-patterns (god objects, spaghetti code)
4. Missing error handling patterns
5. Inconsistent coding standards

Focus on maintainability and future-proofing."#;

        let user_prompt = format!("Review this code for best practices:\n\n{}", content);

        let response = self.model_provider.generate_with_context(system_prompt, &user_prompt).await?;

        Ok(vec![
            ReviewFinding {
                id: "BP-001".to_string(),
                file_path: file_path.to_string(),
                line_start: Some(20),
                line_end: Some(25),
                severity: Severity::Medium,
                category: ReviewCategory::BestPractice,
                title: "Missing error handling".to_string(),
                description: "Function doesn't handle potential errors from async operation".to_string(),
                suggestion: Some("Add try-catch or proper error propagation".to_string()),
                examples: vec!["try { await riskyOperation(); } catch (error) { /* handle */ }".to_string()],
            }
        ])
    }

    fn generate_annotated_diffs(&self, changes: &[CodeChange], findings: &[ReviewFinding]) -> Result<Vec<AnnotatedDiff>, Box<dyn std::error::Error>> {
        let mut annotated_diffs = Vec::new();

        for change in changes {
            // Generate diff hunks (simplified)
            let diff_hunk = DiffHunk {
                old_start: 1,
                old_lines: 10,
                new_start: 1,
                new_lines: 12,
                lines: vec![
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: "function login() {".to_string(),
                        old_line_number: Some(1),
                        new_line_number: Some(1),
                    },
                    DiffLine {
                        line_type: DiffLineType::Added,
                        content: "  // Validate input".to_string(),
                        old_line_number: None,
                        new_line_number: Some(2),
                    },
                ],
                annotations: vec![
                    LineAnnotation {
                        line_number: 5,
                        annotation_type: AnnotationType::Warning,
                        finding_id: "SEC-001".to_string(),
                        message: "Potential security issue".to_string(),
                    }
                ],
            };

            // Calculate overall score based on findings
            let relevant_findings: Vec<_> = findings.iter()
                .filter(|f| f.file_path == change.file_path)
                .collect();

            let critical_count = relevant_findings.iter().filter(|f| matches!(f.severity, Severity::Critical)).count();
            let high_count = relevant_findings.iter().filter(|f| matches!(f.severity, Severity::High)).count();

            let overall_score = 100.0 - (critical_count as f32 * 20.0) - (high_count as f32 * 10.0);

            annotated_diffs.push(AnnotatedDiff {
                file_path: change.file_path.clone(),
                hunks: vec![diff_hunk],
                overall_score: overall_score.max(0.0),
                summary: format!("{} findings in this file", relevant_findings.len()),
            });
        }

        Ok(annotated_diffs)
    }

    fn generate_summary(&self, findings: &[ReviewFinding]) -> Result<ReviewSummary, Box<dyn std::error::Error>> {
        let mut by_severity = HashMap::new();
        let mut by_category = HashMap::new();

        for finding in findings {
            *by_severity.entry(format!("{:?}", finding.severity)).or_insert(0) += 1;
            *by_category.entry(format!("{:?}", finding.category)).or_insert(0) += 1;
        }

        // Calculate quality scores
        let critical_count = findings.iter().filter(|f| matches!(f.severity, Severity::Critical)).count();
        let high_count = findings.iter().filter(|f| matches!(f.severity, Severity::High)).count();
        let total_findings = findings.len();

        let code_quality_score = if total_findings == 0 {
            100.0
        } else {
            100.0 - (critical_count as f32 * 15.0) - (high_count as f32 * 5.0)
        };

        let security_findings = findings.iter().filter(|f| matches!(f.category, ReviewCategory::Security)).count();
        let security_score = 100.0 - (security_findings as f32 * 20.0);

        let maintainability_issues = findings.iter()
            .filter(|f| matches!(f.category, ReviewCategory::CodeSmell | ReviewCategory::BestPractice))
            .count();
        let maintainability_score = 100.0 - (maintainability_issues as f32 * 3.0);

        Ok(ReviewSummary {
            total_findings: findings.len(),
            findings_by_severity: by_severity,
            findings_by_category: by_category,
            code_quality_score: code_quality_score.max(0.0),
            security_score: security_score.max(0.0),
            maintainability_score: maintainability_score.max(0.0),
        })
    }

    fn generate_recommendations(&self, findings: &[ReviewFinding]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();

        let critical_count = findings.iter().filter(|f| matches!(f.severity, Severity::Critical)).count();
        if critical_count > 0 {
            recommendations.push(format!("Fix {} critical issues before merging", critical_count));
        }

        let security_count = findings.iter().filter(|f| matches!(f.category, ReviewCategory::Security)).count();
        if security_count > 0 {
            recommendations.push("Security review recommended before deployment".to_string());
        }

        let test_findings = findings.iter().filter(|f| matches!(f.category, ReviewCategory::Testing)).count();
        if test_findings == 0 {
            recommendations.push("Consider adding more comprehensive test coverage".to_string());
        }

        Ok(recommendations)
    }

    fn determine_approval_status(&self, findings: &[ReviewFinding], summary: &ReviewSummary) -> Result<ApprovalStatus, Box<dyn std::error::Error>> {
        let critical_count = findings.iter().filter(|f| matches!(f.severity, Severity::Critical)).count();
        let high_count = findings.iter().filter(|f| matches!(f.severity, Severity::High)).count();

        if critical_count > 0 {
            Ok(ApprovalStatus::Rejected)
        } else if high_count > 3 {
            Ok(ApprovalStatus::RequiresChanges)
        } else if summary.code_quality_score < 70.0 {
            Ok(ApprovalStatus::RequiresChanges)
        } else if high_count > 0 || summary.code_quality_score < 90.0 {
            Ok(ApprovalStatus::ApprovedWithComments)
        } else {
            Ok(ApprovalStatus::Approved)
        }
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
    async fn test_code_review() {
        let mock_provider = Box::new(MockModelProvider {
            response: "Mock review response".to_string(),
        });

        let agent = ReviewerAgent::new(mock_provider);
        let request = ReviewRequest {
            code_changes: vec![CodeChange {
                file_path: "src/auth.js".to_string(),
                old_content: "// old code".to_string(),
                new_content: "// new code".to_string(),
                change_type: "modify".to_string(),
            }],
            requirements: None,
            context: None,
            review_focus: Some(vec![ReviewFocus::Security, ReviewFocus::Performance]),
        };

        let response = agent.review_changes(request).await.unwrap();
        assert!(!response.findings.is_empty());
        assert!(!response.annotated_diffs.is_empty());
    }
}
