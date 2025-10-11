use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use anyhow::Result;

use codex_agents::{
    AgentSuite, AgentOrchestrationRequest, AgentType,
    spec::SpecAgent,
    code::CodeAgent,
    test_generator::TestGeneratorAgent,
    reviewer::ReviewerAgent,
    debug::DebugAgent,
};

/// Orchestrator service for managing agent workflows
pub struct AgentOrchestrator {
    agent_suite: Arc<AgentSuite>,
    state: Mutex<OrchestratorState>,
}

#[derive(Debug, Default)]
struct OrchestratorState {
    active_sessions: usize,
    total_requests: u64,
}

/// Request for agent orchestration
#[derive(Debug, Serialize, Deserialize)]
pub struct OrchestrationRequest {
    pub prompt: String,
    pub context: Option<serde_json::Value>,
    pub agent_sequence: Option<Vec<String>>,
    pub options: Option<serde_json::Value>,
}

/// Response from agent orchestration
#[derive(Debug, Serialize, Deserialize)]
pub struct OrchestrationResponse {
    pub request_id: String,
    pub status: OrchestrationStatus,
    pub result: Option<serde_json::Value>,
    pub metadata: OrchestrationMetadata,
}

/// Status of an orchestration request
#[derive(Debug, Serialize, Deserialize)]
pub enum OrchestrationStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

/// Metadata about the orchestration
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OrchestrationMetadata {
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_ms: Option<i64>,
    pub agent_sequence: Vec<String>,
    pub success: bool,
    pub error: Option<String>,
}

impl AgentOrchestrator {
    /// Create a new orchestrator with default agents
    pub fn new() -> Result<Self> {
        // Initialize model provider (should be injected in production)
        let model_provider = Arc::new(DefaultModelProvider::default());
        
        let agent_suite = AgentSuite::new(
            SpecAgent::new(model_provider.clone()),
            CodeAgent::new(model_provider.clone()),
            TestGeneratorAgent::new(model_provider.clone()),
            ReviewerAgent::new(model_provider.clone()),
            DebugAgent::new(model_provider),
        );

        Ok(Self {
            agent_suite: Arc::new(agent_suite),
            state: Mutex::new(OrchestratorState::default()),
        })
    }

    /// Process an orchestration request
    pub async fn process_request(
        &self,
        request: OrchestrationRequest,
    ) -> Result<OrchestrationResponse> {
        let start_time = chrono::Utc::now();
        let request_id = uuid::Uuid::new_v4().to_string();
        
        // Update state
        {
            let mut state = self.state.lock().await;
            state.active_sessions += 1;
            state.total_requests += 1;
        }

        // Map request to agent suite format
        let agent_sequence = request.agent_sequence
            .unwrap_or_else(|| vec!["spec".to_string(), "code".to_string(), "reviewer".to_string()])
            .into_iter()
            .filter_map(|agent| match agent.to_lowercase().as_str() {
                "spec" => Some(AgentType::Spec),
                "code" => Some(AgentType::Code),
                "test" | "test_generator" => Some(AgentType::TestGenerator),
                "review" | "reviewer" => Some(AgentType::Reviewer),
                "debug" => Some(AgentType::Debug),
                _ => None,
            })
            .collect::<Vec<_>>();

        let agent_request = AgentOrchestrationRequest {
            prompt: request.prompt,
            context: request.context.map(|ctx| {
                ctx.as_object()
                    .map(|map| map.iter().map(|(k, v)| (k.clone(), v.to_string())).collect())
                    .unwrap_or_default()
            }),
            agent_sequence: Some(agent_sequence.clone()),
            options: request.options.map(|opts| {
                opts.as_object()
                    .map(|map| map.iter().map(|(k, v)| (k.clone(), v.to_string())).collect())
                    .unwrap_or_default()
            }),
        };

        // Execute agent sequence
        let response = match self.agent_suite.orchestrate(agent_request).await {
            Ok(response) => {
                let end_time = chrono::Utc::now();
                let duration = end_time.signed_duration_since(start_time);
                
                OrchestrationResponse {
                    request_id: request_id.clone(),
                    status: OrchestrationStatus::Completed,
                    result: Some(serde_json::to_value(response.final_result)?),
                    metadata: OrchestrationMetadata {
                        start_time: Some(start_time),
                        end_time: Some(end_time),
                        duration_ms: Some(duration.num_milliseconds()),
                        agent_sequence: agent_sequence.into_iter().map(|a| format!("{:?}", a)).collect(),
                        success: true,
                        error: None,
                    },
                }
            }
            Err(e) => {
                OrchestrationResponse {
                    request_id: request_id.clone(),
                    status: OrchestrationStatus::Failed(e.to_string()),
                    result: None,
                    metadata: OrchestrationMetadata {
                        start_time: Some(start_time),
                        end_time: Some(chrono::Utc::now()),
                        duration_ms: Some(chrono::Utc::now().signed_duration_since(start_time).num_milliseconds()),
                        agent_sequence: agent_sequence.into_iter().map(|a| format!("{:?}", a)).collect(),
                        success: false,
                        error: Some(e.to_string()),
                    },
                }
            }
        };

        // Update state
        {
            let mut state = self.state.lock().await;
            state.active_sessions -= 1;
        }

        Ok(response)
    }
}

/// Default model provider implementation
#[derive(Clone)]
struct DefaultModelProvider;

#[async_trait]
impl codex_agents::ModelProvider for DefaultModelProvider {
    async fn generate_completion(&self, _prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, this would call the actual model provider API
        Ok("Generated completion".to_string())
    }

    async fn generate_with_context(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, this would call the actual model provider API
        Ok("Generated completion with context".to_string())
    }
}

#[async_trait]
impl codex_agents::code::ModelProvider for DefaultModelProvider {
    async fn stream_completion(
        &self,
        _prompt: &str,
    ) -> Result<Box<dyn codex_agents::code::StreamCompletion + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, this would return a streaming response
        Ok(Box::new(MockStreamCompletion::new("".to_string())))
    }
}

struct MockStreamCompletion {
    content: String,
    position: usize,
}

impl MockStreamCompletion {
    fn new(content: String) -> Self {
        Self { content, position: 0 }
    }
}

#[async_trait::async_trait]
impl codex_agents::code::StreamCompletion for MockStreamCompletion {
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
