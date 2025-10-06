use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Main Agents module that orchestrates all agent types
pub mod spec;
pub mod code;
pub mod test_generator;
pub mod reviewer;
pub mod debug;

pub use spec::{SpecAgent, SpecRequest, SpecResponse};
pub use code::{CodeAgent, CodeRequest as CodeGenerationRequest, CodeStream};
pub use test_generator::{TestGeneratorAgent, TestRequest as TestGenerationRequest, TestSuite};
pub use reviewer::{ReviewerAgent, ReviewRequest, ReviewReport};
pub use debug::{DebugAgent, DebugRequest, DebugReport};

/// Agent orchestration and coordination
#[derive(Debug, Clone)]
pub struct AgentSuite {
    spec_agent: Arc<SpecAgent>,
    code_agent: Arc<CodeAgent>,
    test_generator: Arc<TestGeneratorAgent>,
    reviewer: Arc<ReviewerAgent>,
    debug_agent: Arc<DebugAgent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOrchestrationRequest {
    pub prompt: String,
    pub context: Option<HashMap<String, String>>,
    pub agent_sequence: Option<Vec<AgentType>>,
    pub options: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    Spec,
    Code,
    TestGenerator,
    Reviewer,
    Debug,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOrchestrationResponse {
    pub execution_order: Vec<AgentExecution>,
    pub final_result: Option<serde_json::Value>,
    pub metadata: OrchestrationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecution {
    pub agent_type: AgentType,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub success: bool,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationMetadata {
    pub total_execution_time_ms: u64,
    pub agents_executed: usize,
    pub success_rate: f32,
    pub warnings: Vec<String>,
}

impl AgentSuite {
    pub fn new(
        spec_agent: SpecAgent,
        code_agent: CodeAgent,
        test_generator: TestGeneratorAgent,
        reviewer: ReviewerAgent,
        debug_agent: DebugAgent,
    ) -> Self {
        Self {
            spec_agent: Arc::new(spec_agent),
            code_agent: Arc::new(code_agent),
            test_generator: Arc::new(test_generator),
            reviewer: Arc::new(reviewer),
            debug_agent: Arc::new(debug_agent),
        }
    }

    pub async fn orchestrate(&self, request: AgentOrchestrationRequest) -> Result<AgentOrchestrationResponse, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut executions = Vec::new();
        let mut warnings = Vec::new();

        // Determine execution order
        let agent_sequence = request.agent_sequence.unwrap_or_else(|| {
            vec![
                AgentType::Spec,      // Generate requirements first
                AgentType::Code,      // Generate code based on spec
                AgentType::Reviewer,   // Review the generated code
                AgentType::TestGenerator, // Generate tests for the code
            ]
        });

        let mut previous_output = None;

        for agent_type in agent_sequence {
            let execution_start = std::time::Instant::now();

            let (input, output, success, error) = match self.execute_agent(&agent_type, &request, previous_output.as_ref()).await {
                Ok((input, output)) => {
                    previous_output = Some(output.clone());
                    (input, output, true, None)
                }
                Err(e) => {
                    warnings.push(format!("Agent {:?} failed: {}", agent_type, e));
                    let error_output = serde_json::json!({
                        "error": e.to_string(),
                        "agent": format!("{:?}", agent_type)
                    });
                    (serde_json::Value::Null, error_output, false, Some(e.to_string()))
                }
            };

            executions.push(AgentExecution {
                agent_type,
                input,
                output,
                success,
                execution_time_ms: execution_start.elapsed().as_millis() as u64,
                error_message: error,
            });
        }

        let total_time = start_time.elapsed().as_millis() as u64;
        let success_count = executions.iter().filter(|e| e.success).count();
        let success_rate = if executions.is_empty() {
            0.0
        } else {
            (success_count as f32 / executions.len() as f32) * 100.0
        };

        Ok(AgentOrchestrationResponse {
            execution_order: executions,
            final_result: previous_output,
            metadata: OrchestrationMetadata {
                total_execution_time_ms: total_time,
                agents_executed: agent_sequence.len(),
                success_rate,
                warnings,
            },
        })
    }

    async fn execute_agent(
        &self,
        agent_type: &AgentType,
        request: &AgentOrchestrationRequest,
        previous_output: Option<&serde_json::Value>,
    ) -> Result<(serde_json::Value, serde_json::Value), Box<dyn std::error::Error>> {
        let input = self.prepare_agent_input(agent_type, request, previous_output)?;

        match agent_type {
            AgentType::Spec => {
                let spec_request: SpecRequest = serde_json::from_value(input.clone())?;
                let spec_response = self.spec_agent.generate_spec(spec_request).await?;
                let output = serde_json::to_value(spec_response)?;
                Ok((input, output))
            }
            AgentType::Code => {
                let code_request: code::CodeRequest = serde_json::from_value(input.clone())?;
                let code_response = self.code_agent.generate_code(code_request).await?;
                let output = serde_json::to_value(code_response)?;
                Ok((input, output))
            }
            AgentType::TestGenerator => {
                let test_request: test_generator::TestRequest = serde_json::from_value(input.clone())?;
                let test_response = self.test_generator.generate_tests(test_request).await?;
                let output = serde_json::to_value(test_response)?;
                Ok((input, output))
            }
            AgentType::Reviewer => {
                let review_request: reviewer::ReviewRequest = serde_json::from_value(input.clone())?;
                let review_response = self.reviewer.review_changes(review_request).await?;
                let output = serde_json::to_value(review_response)?;
                Ok((input, output))
            }
            AgentType::Debug => {
                let debug_request: debug::DebugRequest = serde_json::from_value(input.clone())?;
                let debug_response = self.debug_agent.analyze_logs(debug_request).await?;
                let output = serde_json::to_value(debug_response)?;
                Ok((input, output))
            }
        }
    }

    fn prepare_agent_input(
        &self,
        agent_type: &AgentType,
        request: &AgentOrchestrationRequest,
        previous_output: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        match agent_type {
            AgentType::Spec => {
                // Spec agent takes the original prompt and context
                Ok(serde_json::json!({
                    "prompt": request.prompt,
                    "context": request.context,
                    "project_type": request.options.as_ref().and_then(|o| o.get("project_type")),
                    "existing_requirements": request.options.as_ref().and_then(|o| o.get("existing_requirements")),
                }))
            }
            AgentType::Code => {
                // Code agent needs spec output and context
                if let Some(spec_output) = previous_output {
                    Ok(serde_json::json!({
                        "prompt": request.prompt,
                        "context": request.context,
                        "requirements": spec_output.get("requirements"),
                        "existing_files": request.options.as_ref().and_then(|o| o.get("existing_files")),
                        "target_files": request.options.as_ref().and_then(|o| o.get("target_files")),
                    }))
                } else {
                    Ok(serde_json::json!({
                        "prompt": request.prompt,
                        "context": request.context,
                    }))
                }
            }
            AgentType::TestGenerator => {
                // Test generator needs code changes and requirements
                if let Some(code_output) = previous_output {
                    Ok(serde_json::json!({
                        "code_changes": code_output.get("changes"),
                        "requirements": code_output.get("requirements"),
                        "test_framework": request.options.as_ref().and_then(|o| o.get("test_framework")),
                        "coverage_goals": request.options.as_ref().and_then(|o| o.get("coverage_goals")),
                    }))
                } else {
                    Ok(serde_json::json!({
                        "prompt": request.prompt,
                    }))
                }
            }
            AgentType::Reviewer => {
                // Reviewer needs code changes for review
                if let Some(code_output) = previous_output {
                    Ok(serde_json::json!({
                        "code_changes": code_output.get("changes"),
                        "requirements": code_output.get("requirements"),
                        "review_focus": request.options.as_ref().and_then(|o| o.get("review_focus")),
                    }))
                } else {
                    Ok(serde_json::json!({
                        "prompt": request.prompt,
                    }))
                }
            }
            AgentType::Debug => {
                // Debug agent needs logs and context
                Ok(serde_json::json!({
                    "logs": request.options.as_ref().and_then(|o| o.get("logs")),
                    "error_context": request.context,
                    "codebase_files": request.options.as_ref().and_then(|o| o.get("codebase_files")),
                    "debug_focus": request.options.as_ref().and_then(|o| o.get("debug_focus")),
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockModelProvider {
        response: String,
    }

    impl spec::ModelProvider for MockModelProvider {
        async fn generate_completion(&self, _prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
            Ok(self.response.clone())
        }

        async fn generate_with_context(&self, _system_prompt: &str, _user_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
            Ok(self.response.clone())
        }
    }

    impl code::ModelProvider for MockModelProvider {
        async fn generate_completion(&self, _prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
            Ok(self.response.clone())
        }

        async fn generate_with_context(&self, _system_prompt: &str, _user_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
            Ok(self.response.clone())
        }

        async fn stream_completion(&self, _prompt: &str) -> Result<Box<dyn code::StreamCompletion>, Box<dyn std::error::Error>> {
            Ok(Box::new(crate::code::MockStreamCompletion::new(self.response.clone())))
        }
    }

    impl test_generator::ModelProvider for MockModelProvider {
        async fn generate_completion(&self, _prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
            Ok(self.response.clone())
        }

        async fn generate_with_context(&self, _system_prompt: &str, _user_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
            Ok(self.response.clone())
        }
    }

    impl reviewer::ModelProvider for MockModelProvider {
        async fn generate_completion(&self, _prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
            Ok(self.response.clone())
        }

        async fn generate_with_context(&self, _system_prompt: &str, _user_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
            Ok(self.response.clone())
        }
    }

    impl debug::ModelProvider for MockModelProvider {
        async fn generate_completion(&self, _prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
            Ok(self.response.clone())
        }

        async fn generate_with_context(&self, _system_prompt: &str, _user_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
            Ok(self.response.clone())
        }
    }

    #[tokio::test]
    async fn test_agent_orchestration() {
        let mock_provider = Box::new(MockModelProvider {
            response: "Mock response".to_string(),
        });

        let suite = AgentSuite::new(
            SpecAgent::new(mock_provider.clone()),
            CodeAgent::new(mock_provider.clone()),
            TestGeneratorAgent::new(mock_provider.clone()),
            ReviewerAgent::new(mock_provider.clone()),
            DebugAgent::new(mock_provider),
        );

        let request = AgentOrchestrationRequest {
            prompt: "Add user authentication".to_string(),
            context: None,
            agent_sequence: Some(vec![AgentType::Spec, AgentType::Code]),
            options: None,
        };

        let response = suite.orchestrate(request).await.unwrap();
        assert_eq!(response.execution_order.len(), 2);
        assert!(response.metadata.total_execution_time_ms > 0);
    }
}
