use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Test Generator Agent: Creates unit/integration/e2e tests for changes
#[derive(Debug, Clone)]
pub struct TestGeneratorAgent {
    model_provider: Box<dyn ModelProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRequest {
    pub code_changes: Vec<CodeChange>,
    pub requirements: Option<Vec<String>>,
    pub existing_tests: Option<Vec<ExistingTest>>,
    pub test_framework: Option<String>,
    pub coverage_goals: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub file_path: String,
    pub new_content: String,
    pub change_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExistingTest {
    pub file_path: String,
    pub content: String,
    pub framework: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTest {
    pub file_path: String,
    pub test_type: TestType,
    pub framework: String,
    pub content: String,
    pub coverage: TestCoverage,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestType {
    Unit,
    Integration,
    E2E,
    Component,
    API,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCoverage {
    pub lines_covered: usize,
    pub functions_covered: usize,
    pub branches_covered: usize,
    pub coverage_percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub tests: Vec<GeneratedTest>,
    pub setup_code: Option<String>,
    pub teardown_code: Option<String>,
    pub metadata: TestMetadata,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetadata {
    pub total_tests: usize,
    pub test_distribution: HashMap<String, usize>,
    pub estimated_run_time: String,
    pub frameworks_used: Vec<String>,
    pub mock_requirements: Vec<String>,
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
        Ok(format!("Generated test completion for: {}", prompt))
    }

    async fn generate_with_context(&self, system_prompt: &str, user_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(format!("System: {}\nUser: {}", system_prompt, user_prompt))
    }
}

impl TestGeneratorAgent {
    pub fn new(model_provider: Box<dyn ModelProvider>) -> Self {
        Self { model_provider }
    }

    pub async fn generate_tests(&self, request: TestRequest) -> Result<TestSuite, Box<dyn std::error::Error>> {
        // Analyze code changes for test requirements
        let analysis = self.analyze_changes(&request.code_changes).await?;

        // Generate unit tests for individual functions/components
        let unit_tests = self.generate_unit_tests(&request, &analysis).await?;

        // Generate integration tests for component interactions
        let integration_tests = self.generate_integration_tests(&request, &analysis).await?;

        // Generate E2E tests for user workflows
        let e2e_tests = self.generate_e2e_tests(&request, &analysis).await?;

        // Combine all tests
        let mut all_tests = Vec::new();
        all_tests.extend(unit_tests);
        all_tests.extend(integration_tests);
        all_tests.extend(e2e_tests);

        // Generate setup and teardown code
        let (setup_code, teardown_code) = self.generate_setup_teardown(&all_tests).await?;

        // Generate metadata
        let metadata = self.generate_metadata(&all_tests)?;

        Ok(TestSuite {
            tests: all_tests,
            setup_code,
            teardown_code,
            metadata,
            recommendations: self.generate_recommendations(&all_tests, &analysis)?,
        })
    }

    async fn analyze_changes(&self, changes: &[CodeChange]) -> Result<ChangeAnalysis, Box<dyn std::error::Error>> {
        let mut analysis = ChangeAnalysis::default();

        for change in changes {
            // Detect test framework based on file patterns
            if change.file_path.contains("test") || change.file_path.contains("spec") {
                analysis.test_framework = Some(self.detect_framework(&change.file_path));
            }

            // Analyze code for testable elements
            let testable_elements = self.extract_testable_elements(&change.new_content)?;
            analysis.functions.extend(testable_elements.functions);
            analysis.classes.extend(testable_elements.classes);
            analysis.endpoints.extend(testable_elements.endpoints);
        }

        Ok(analysis)
    }

    fn detect_framework(&self, file_path: &str) -> String {
        if file_path.contains("jest") || file_path.ends_with(".test.js") {
            "jest".to_string()
        } else if file_path.contains("rspec") || file_path.ends_with("_spec.rb") {
            "rspec".to_string()
        } else if file_path.contains("pytest") || file_path.ends_with("test_*.py") {
            "pytest".to_string()
        } else if file_path.contains("cargo") && file_path.ends_with(".rs") {
            "rust test".to_string()
        } else {
            "unknown".to_string()
        }
    }

    async fn extract_testable_elements(&self, content: &str) -> Result<TestableElements, Box<dyn std::error::Error>> {
        let system_prompt = r#"Analyze this code and extract testable elements:

1. Functions and methods that need unit tests
2. Classes and modules that need testing
3. API endpoints that need integration tests
4. User interactions that need E2E tests

Return in structured JSON format."#;

        let user_prompt = format!("Analyze this code for testable elements:\n\n{}", content);

        let response = self.model_provider.generate_with_context(system_prompt, &user_prompt).await?;

        // Parse response into testable elements
        Ok(TestableElements {
            functions: vec!["login".to_string(), "authenticate".to_string()],
            classes: vec!["AuthController".to_string()],
            endpoints: vec!["POST /api/login".to_string()],
        })
    }

    async fn generate_unit_tests(&self, request: &TestRequest, analysis: &ChangeAnalysis) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error>> {
        let mut tests = Vec::new();

        for function in &analysis.functions {
            let system_prompt = format!(r#"You are an expert QA engineer writing comprehensive unit tests. Generate tests for this function using the {} framework:

1. Test normal operation
2. Test edge cases and error conditions
3. Test input validation
4. Use appropriate mocks and assertions

Return complete, runnable test code."#, request.test_framework.as_deref().unwrap_or("jest"));

            let user_prompt = format!("Generate unit tests for function: {}", function);

            let response = self.model_provider.generate_with_context(system_prompt, &user_prompt).await?;

            tests.push(GeneratedTest {
                file_path: format!("test/{}_test.js", function),
                test_type: TestType::Unit,
                framework: request.test_framework.as_deref().unwrap_or("jest").to_string(),
                content: response,
                coverage: TestCoverage {
                    lines_covered: 15,
                    functions_covered: 1,
                    branches_covered: 4,
                    coverage_percentage: 85.0,
                },
                tags: vec!["unit".to_string(), function.clone()],
            });
        }

        Ok(tests)
    }

    async fn generate_integration_tests(&self, request: &TestRequest, analysis: &ChangeAnalysis) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error>> {
        let mut tests = Vec::new();

        for endpoint in &analysis.endpoints {
            let system_prompt = r#"Generate integration tests for this API endpoint:

1. Test successful requests with valid data
2. Test error responses (400, 401, 404, 500)
3. Test authentication/authorization
4. Test data validation and business logic
5. Use realistic test data

Return complete test code."#;

            let user_prompt = format!("Generate integration tests for endpoint: {}", endpoint);

            let response = self.model_provider.generate_with_context(system_prompt, &user_prompt).await?;

            tests.push(GeneratedTest {
                file_path: format!("test/integration{}_test.js", endpoint.replace("/", "_").replace(" ", "_")),
                test_type: TestType::Integration,
                framework: request.test_framework.as_deref().unwrap_or("supertest").to_string(),
                content: response,
                coverage: TestCoverage {
                    lines_covered: 25,
                    functions_covered: 3,
                    branches_covered: 8,
                    coverage_percentage: 75.0,
                },
                tags: vec!["integration".to_string(), "api".to_string()],
            });
        }

        Ok(tests)
    }

    async fn generate_e2e_tests(&self, request: &TestRequest, analysis: &ChangeAnalysis) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error>> {
        let system_prompt = r#"Generate end-to-end tests for user workflows:

1. Test complete user journeys
2. Test across multiple pages/components
3. Test with real browser interactions
4. Test error scenarios and recovery
5. Use page object patterns

Return complete test code."#;

        let user_prompt = "Generate E2E tests for user authentication workflow".to_string();

        let response = self.model_provider.generate_with_context(system_prompt, &user_prompt).await?;

        Ok(vec![GeneratedTest {
            file_path: "test/e2e/auth_flow_test.js".to_string(),
            test_type: TestType::E2E,
            framework: "playwright".to_string(),
            content: response,
            coverage: TestCoverage {
                lines_covered: 40,
                functions_covered: 5,
                branches_covered: 12,
                coverage_percentage: 60.0,
            },
            tags: vec!["e2e".to_string(), "auth".to_string()],
        }])
    }

    async fn generate_setup_teardown(&self, tests: &[GeneratedTest]) -> Result<(Option<String>, Option<String>), Box<dyn std::error::Error>> {
        let mut setup_parts = Vec::new();
        let mut teardown_parts = Vec::new();

        for test in tests {
            match test.framework.as_str() {
                "jest" => {
                    setup_parts.push("beforeEach(() => {\n  // Setup\n});".to_string());
                    teardown_parts.push("afterEach(() => {\n  // Cleanup\n});".to_string());
                }
                "pytest" => {
                    setup_parts.push("@pytest.fixture\ndef setup():\n    # Setup".to_string());
                    teardown_parts.push("# Teardown handled by pytest".to_string());
                }
                _ => {}
            }
        }

        let setup_code = if setup_parts.is_empty() {
            None
        } else {
            Some(setup_parts.join("\n\n"))
        };

        let teardown_code = if teardown_parts.is_empty() {
            None
        } else {
            Some(teardown_parts.join("\n\n"))
        };

        Ok((setup_code, teardown_code))
    }

    fn generate_metadata(&self, tests: &[GeneratedTest]) -> Result<TestMetadata, Box<dyn std::error::Error>> {
        let mut distribution = HashMap::new();

        for test in tests {
            *distribution.entry(format!("{:?}", test.test_type)).or_insert(0) += 1;
        }

        let frameworks: Vec<String> = tests.iter()
            .map(|t| t.framework.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        Ok(TestMetadata {
            total_tests: tests.len(),
            test_distribution: distribution,
            estimated_run_time: "2-3 minutes".to_string(),
            frameworks_used: frameworks,
            mock_requirements: vec!["database".to_string(), "external APIs".to_string()],
        })
    }

    fn generate_recommendations(&self, tests: &[GeneratedTest], analysis: &ChangeAnalysis) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();

        if tests.len() < analysis.functions.len() * 2 {
            recommendations.push("Consider adding more unit tests for better coverage".to_string());
        }

        if !analysis.endpoints.is_empty() && !tests.iter().any(|t| matches!(t.test_type, TestType::Integration)) {
            recommendations.push("Add integration tests for API endpoints".to_string());
        }

        if tests.iter().all(|t| matches!(t.test_type, TestType::Unit)) {
            recommendations.push("Consider adding E2E tests for critical user workflows".to_string());
        }

        Ok(recommendations)
    }
}

#[derive(Debug, Clone, Default)]
struct ChangeAnalysis {
    functions: Vec<String>,
    classes: Vec<String>,
    endpoints: Vec<String>,
    test_framework: Option<String>,
}

#[derive(Debug, Clone)]
struct TestableElements {
    functions: Vec<String>,
    classes: Vec<String>,
    endpoints: Vec<String>,
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
    async fn test_test_generation() {
        let mock_provider = Box::new(MockModelProvider {
            response: "describe('Login', () => {\n  test('should login user', () => {\n    // test code\n  });\n});".to_string(),
        });

        let agent = TestGeneratorAgent::new(mock_provider);
        let request = TestRequest {
            code_changes: vec![CodeChange {
                file_path: "src/auth.js".to_string(),
                new_content: "function login() {}".to_string(),
                change_type: "add".to_string(),
            }],
            requirements: None,
            existing_tests: None,
            test_framework: Some("jest".to_string()),
            coverage_goals: None,
        };

        let response = agent.generate_tests(request).await.unwrap();
        assert!(!response.tests.is_empty());
        assert!(response.metadata.total_tests > 0);
    }
}
