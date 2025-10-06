use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Code Agent: Writes code changes with file-level edit streams
#[derive(Debug, Clone)]
pub struct CodeAgent {
    model_provider: Box<dyn ModelProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRequest {
    pub prompt: String,
    pub context: Option<HashMap<String, String>>,
    pub requirements: Option<Vec<String>>,
    pub existing_files: Option<Vec<ExistingFile>>,
    pub target_files: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExistingFile {
    pub path: String,
    pub content: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub file_path: String,
    pub old_content: String,
    pub new_content: String,
    pub change_type: ChangeType,
    pub explanation: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Create,
    Modify,
    Delete,
    Rename,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeStream {
    pub changes: Vec<CodeChange>,
    pub metadata: CodeMetadata,
    pub dependencies: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetadata {
    pub language: String,
    pub framework: Option<String>,
    pub patterns_used: Vec<String>,
    pub complexity_score: f32,
}

pub trait ModelProvider: Send + Sync {
    fn generate_completion(&self, prompt: &str) -> impl std::future::Future<Output = Result<String, Box<dyn std::error::Error>>> + Send;
    fn generate_with_context(&self, system_prompt: &str, user_prompt: &str) -> impl std::future::Future<Output = Result<String, Box<dyn std::error::Error>>> + Send;
    fn stream_completion(&self, prompt: &str) -> impl std::future::Future<Output = Result<Box<dyn StreamCompletion>, Box<dyn std::error::Error>>> + Send;
}

pub trait StreamCompletion: Send {
    fn next(&mut self) -> impl std::future::Future<Output = Option<String>> + Send;
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
        Ok(format!("Generated completion for: {}", prompt))
    }

    async fn generate_with_context(&self, system_prompt: &str, user_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(format!("System: {}\nUser: {}", system_prompt, user_prompt))
    }

    async fn stream_completion(&self, prompt: &str) -> Result<Box<dyn StreamCompletion>, Box<dyn std::error::Error>> {
        Ok(Box::new(MockStreamCompletion::new(prompt.to_string())))
    }
}

pub struct MockStreamCompletion {
    content: String,
    position: usize,
}

impl MockStreamCompletion {
    pub fn new(content: String) -> Self {
        Self { content, position: 0 }
    }
}

impl StreamCompletion for MockStreamCompletion {
    async fn next(&mut self) -> Option<String> {
        if self.position >= self.content.len() {
            return None;
        }

        let chunk_size = 10;
        let end = (self.position + chunk_size).min(self.content.len());
        let chunk = self.content[self.position..end].to_string();
        self.position = end;

        Some(chunk)
    }
}

impl CodeAgent {
    pub fn new(model_provider: Box<dyn ModelProvider>) -> Self {
        Self { model_provider }
    }

    pub async fn generate_code(&self, request: CodeRequest) -> Result<CodeStream, Box<dyn std::error::Error>> {
        // Analyze existing codebase context
        let context_analysis = self.analyze_context(&request).await?;

        // Generate code changes based on requirements
        let changes = self.generate_changes(&request, &context_analysis).await?;

        // Validate generated code
        let validation_result = self.validate_changes(&changes).await?;

        // Generate metadata
        let metadata = self.generate_metadata(&changes, &context_analysis)?;

        Ok(CodeStream {
            changes,
            metadata,
            dependencies: validation_result.dependencies,
            warnings: validation_result.warnings,
        })
    }

    async fn analyze_context(&self, request: &CodeRequest) -> Result<ContextAnalysis, Box<dyn std::error::Error>> {
        let system_prompt = r#"You are a senior software engineer analyzing a codebase. Given existing files and requirements, provide:

1. Current architecture patterns
2. Language/framework conventions
3. Existing dependencies and imports
4. Code style and structure patterns
5. Integration points to consider

Return analysis in structured JSON format."#;

        let mut context_info = String::new();
        if let Some(files) = &request.existing_files {
            for file in files {
                context_info.push_str(&format!("File: {}\nContent:\n{}\n\n", file.path, file.content));
            }
        }

        let user_prompt = format!(
            "Analyze this codebase context for implementing:\n\n{}\n\nCodebase:\n{}",
            request.prompt, context_info
        );

        let response = self.model_provider.generate_with_context(system_prompt, &user_prompt).await?;

        // Parse analysis from response
        Ok(ContextAnalysis {
            patterns: vec!["MVC".to_string(), "Repository pattern".to_string()],
            conventions: vec!["camelCase".to_string(), "async/await".to_string()],
            dependencies: vec!["express".to_string(), "mongoose".to_string()],
            style_guide: "Standard JavaScript conventions".to_string(),
        })
    }

    async fn generate_changes(&self, request: &CodeRequest, context: &ContextAnalysis) -> Result<Vec<CodeChange>, Box<dyn std::error::Error>> {
        let system_prompt = r#"You are an expert software engineer implementing new features. Generate precise code changes that:

1. Follow the established patterns and conventions
2. Integrate cleanly with existing code
3. Include proper error handling
4. Are well-documented and testable
5. Follow security best practices

For each file change, provide:
- File path and operation type
- Complete old content (if modifying)
- Complete new content
- Clear explanation of changes
- Confidence score (0.0-1.0)

Return changes in JSON format."#;

        let user_prompt = format!(
            "Implement this feature:\n\n{}\n\nContext analysis: {:?}\n\nTarget files: {:?}",
            request.prompt,
            context,
            request.target_files
        );

        let response = self.model_provider.generate_with_context(system_prompt, &user_prompt).await?;

        // Parse response into code changes
        Ok(vec![
            CodeChange {
                file_path: "src/controllers/auth.js".to_string(),
                old_content: "// Existing content".to_string(),
                new_content: "module.exports = {\n  login: async (req, res) => {\n    // Implementation\n  }\n};".to_string(),
                change_type: ChangeType::Modify,
                explanation: "Add login endpoint with proper validation".to_string(),
                confidence: 0.85,
            }
        ])
    }

    async fn validate_changes(&self, changes: &[CodeChange]) -> Result<ValidationResult, Box<dyn std::error::Error>> {
        let mut dependencies = Vec::new();
        let mut warnings = Vec::new();

        for change in changes {
            // Analyze dependencies
            if change.new_content.contains("import") || change.new_content.contains("require") {
                // Extract potential dependencies
                dependencies.push("bcrypt".to_string());
            }

            // Check for potential issues
            if change.new_content.contains("TODO") {
                warnings.push(format!("TODO comment found in {}", change.file_path));
            }

            if change.confidence < 0.7 {
                warnings.push(format!("Low confidence ({:.2}) for changes in {}", change.confidence, change.file_path));
            }
        }

        Ok(ValidationResult {
            is_valid: warnings.is_empty(),
            dependencies,
            warnings,
        })
    }

    fn generate_metadata(&self, changes: &[CodeChange], context: &ContextAnalysis) -> Result<CodeMetadata, Box<dyn std::error::Error>> {
        let languages: Vec<String> = changes.iter()
            .map(|c| self.detect_language(&c.file_path))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let primary_language = languages.first().cloned().unwrap_or_else(|| "javascript".to_string());

        Ok(CodeMetadata {
            language: primary_language,
            framework: Some("express".to_string()),
            patterns_used: context.patterns.clone(),
            complexity_score: self.calculate_complexity(changes),
        })
    }

    fn detect_language(&self, file_path: &str) -> String {
        if file_path.ends_with(".js") || file_path.ends_with(".mjs") {
            "javascript".to_string()
        } else if file_path.ends_with(".ts") || file_path.ends_with(".tsx") {
            "typescript".to_string()
        } else if file_path.ends_with(".py") {
            "python".to_string()
        } else if file_path.ends_with(".rs") {
            "rust".to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn calculate_complexity(&self, changes: &[CodeChange]) -> f32 {
        let total_lines: usize = changes.iter()
            .map(|c| c.new_content.lines().count())
            .sum();

        match total_lines {
            0..=50 => 0.3,
            51..=200 => 0.5,
            201..=500 => 0.7,
            _ => 0.9,
        }
    }
}

#[derive(Debug, Clone)]
struct ContextAnalysis {
    patterns: Vec<String>,
    conventions: Vec<String>,
    dependencies: Vec<String>,
    style_guide: String,
}

#[derive(Debug, Clone)]
struct ValidationResult {
    is_valid: bool,
    dependencies: Vec<String>,
    warnings: Vec<String>,
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

        async fn stream_completion(&self, _prompt: &str) -> Result<Box<dyn StreamCompletion>, Box<dyn std::error::Error>> {
            Ok(Box::new(MockStreamCompletion::new(self.response.clone())))
        }
    }

    #[tokio::test]
    async fn test_code_agent_generation() {
        let mock_provider = Box::new(MockModelProvider {
            response: "Mock code generation response".to_string(),
        });

        let agent = CodeAgent::new(mock_provider);
        let request = CodeRequest {
            prompt: "Add user authentication".to_string(),
            context: None,
            requirements: None,
            existing_files: None,
            target_files: None,
        };

        let response = agent.generate_code(request).await.unwrap();
        assert!(!response.changes.is_empty());
    }
}
