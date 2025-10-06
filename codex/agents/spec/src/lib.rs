use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Spec Agent: Generates requirements, tests, and acceptance criteria from prompts
#[derive(Debug, Clone)]
pub struct SpecAgent {
    model_provider: Box<dyn ModelProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecRequest {
    pub prompt: String,
    pub context: Option<HashMap<String, String>>,
    pub project_type: Option<String>,
    pub existing_requirements: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub category: RequirementCategory,
    pub acceptance_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequirementCategory {
    Functional,
    NonFunctional,
    Technical,
    Security,
    Performance,
    Usability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,
    pub title: String,
    pub description: String,
    pub test_type: TestType,
    pub steps: Vec<String>,
    pub expected_result: String,
    pub requirement_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestType {
    Unit,
    Integration,
    E2E,
    Manual,
    Regression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecResponse {
    pub requirements: Vec<Requirement>,
    pub test_cases: Vec<TestCase>,
    pub user_stories: Vec<UserStory>,
    pub acceptance_criteria: HashMap<String, Vec<String>>,
    pub metadata: SpecMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStory {
    pub id: String,
    pub title: String,
    pub description: String,
    pub role: String,
    pub goal: String,
    pub benefit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecMetadata {
    pub estimated_effort: String,
    pub complexity: Complexity,
    pub dependencies: Vec<String>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Complexity {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
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
        // Implementation would use OpenAI API
        // For now, return a placeholder
        Ok(format!("Generated completion for: {}", prompt))
    }

    async fn generate_with_context(&self, system_prompt: &str, user_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Implementation would use OpenAI API with system/user messages
        Ok(format!("System: {}\nUser: {}", system_prompt, user_prompt))
    }
}

impl SpecAgent {
    pub fn new(model_provider: Box<dyn ModelProvider>) -> Self {
        Self { model_provider }
    }

    pub async fn generate_spec(&self, request: SpecRequest) -> Result<SpecResponse, Box<dyn std::error::Error>> {
        // Generate requirements from prompt
        let requirements = self.generate_requirements(&request).await?;

        // Generate test cases for requirements
        let test_cases = self.generate_test_cases(&requirements).await?;

        // Generate user stories
        let user_stories = self.generate_user_stories(&request, &requirements).await?;

        // Generate acceptance criteria
        let acceptance_criteria = self.generate_acceptance_criteria(&requirements).await?;

        // Generate metadata
        let metadata = self.generate_metadata(&request, &requirements).await?;

        Ok(SpecResponse {
            requirements,
            test_cases,
            user_stories,
            acceptance_criteria,
            metadata,
        })
    }

    async fn generate_requirements(&self, request: &SpecRequest) -> Result<Vec<Requirement>, Box<dyn std::error::Error>> {
        let system_prompt = r#"You are a senior product manager and systems analyst. Given a feature request, generate detailed, actionable requirements that:

1. Cover functional, non-functional, and technical aspects
2. Include clear acceptance criteria
3. Are prioritized appropriately
4. Are testable and measurable

Return requirements in JSON format with proper categorization."#;

        let user_prompt = format!(
            "Generate requirements for this feature request:\n\n{}\n\n{}",
            request.prompt,
            request.context.as_ref()
                .map(|ctx| format!("Additional context: {:?}", ctx))
                .unwrap_or_default()
        );

        let response = self.model_provider.generate_with_context(system_prompt, &user_prompt).await?;

        // Parse the JSON response into Requirements
        // For now, return placeholder requirements
        Ok(vec![
            Requirement {
                id: "REQ-001".to_string(),
                title: "Implement user authentication".to_string(),
                description: "Users should be able to authenticate using email/password".to_string(),
                priority: Priority::High,
                category: RequirementCategory::Functional,
                acceptance_criteria: vec![
                    "User can register with email and password".to_string(),
                    "User can login with correct credentials".to_string(),
                    "User cannot login with incorrect credentials".to_string(),
                ],
            }
        ])
    }

    async fn generate_test_cases(&self, requirements: &[Requirement]) -> Result<Vec<TestCase>, Box<dyn std::error::Error>> {
        let mut test_cases = Vec::new();

        for requirement in requirements {
            let system_prompt = "Generate comprehensive test cases for this requirement. Include unit, integration, and end-to-end tests where applicable.";

            let user_prompt = format!(
                "Generate test cases for requirement: {} - {}",
                requirement.id, requirement.title
            );

            let response = self.model_provider.generate_with_context(system_prompt, &user_prompt).await?;

            // Parse response into test cases
            // For now, create placeholder test cases
            test_cases.push(TestCase {
                id: format!("TC-{}-001", requirement.id),
                title: format!("Test {}", requirement.title),
                description: format!("Verify {}", requirement.description),
                test_type: TestType::Unit,
                steps: vec![
                    "Navigate to login page".to_string(),
                    "Enter valid credentials".to_string(),
                    "Click login button".to_string(),
                ],
                expected_result: "User should be redirected to dashboard".to_string(),
                requirement_id: requirement.id.clone(),
            });
        }

        Ok(test_cases)
    }

    async fn generate_user_stories(&self, request: &SpecRequest, requirements: &[Requirement]) -> Result<Vec<UserStory>, Box<dyn std::error::Error>> {
        let system_prompt = r#"As a product owner, break down this feature into user stories. Each story should follow the format:
"As a [type of user], I want [some goal] so that [some reason]"

Focus on user value and outcomes."#;

        let user_prompt = format!(
            "Create user stories for this feature:\n\n{}",
            request.prompt
        );

        let response = self.model_provider.generate_with_context(system_prompt, &user_prompt).await?;

        // Parse response into user stories
        Ok(vec![
            UserStory {
                id: "US-001".to_string(),
                title: "User login functionality".to_string(),
                description: "As a user, I want to log into the application so that I can access my personalized dashboard".to_string(),
                role: "User".to_string(),
                goal: "Access personalized dashboard".to_string(),
                benefit: "Secure access to user-specific content".to_string(),
            }
        ])
    }

    async fn generate_acceptance_criteria(&self, requirements: &[Requirement]) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
        let mut criteria = HashMap::new();

        for requirement in requirements {
            criteria.insert(
                requirement.id.clone(),
                requirement.acceptance_criteria.clone(),
            );
        }

        Ok(criteria)
    }

    async fn generate_metadata(&self, request: &SpecRequest, requirements: &[Requirement]) -> Result<SpecMetadata, Box<dyn std::error::Error>> {
        let critical_count = requirements.iter().filter(|r| matches!(r.priority, Priority::Critical)).count();
        let complexity = match requirements.len() {
            0..=3 => Complexity::Simple,
            4..=7 => Complexity::Moderate,
            8..=12 => Complexity::Complex,
            _ => Complexity::VeryComplex,
        };

        let risk_level = if critical_count > 2 {
            RiskLevel::High
        } else if complexity == Complexity::VeryComplex {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        Ok(SpecMetadata {
            estimated_effort: "2-3 weeks".to_string(),
            complexity,
            dependencies: vec!["Authentication service".to_string()],
            risk_level,
        })
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
    async fn test_spec_agent_generation() {
        let mock_provider = Box::new(MockModelProvider {
            response: "Mock response".to_string(),
        });

        let agent = SpecAgent::new(mock_provider);
        let request = SpecRequest {
            prompt: "Implement user authentication".to_string(),
            context: None,
            project_type: Some("web-app".to_string()),
            existing_requirements: None,
        };

        let response = agent.generate_spec(request).await.unwrap();
        assert!(!response.requirements.is_empty());
    }
}
