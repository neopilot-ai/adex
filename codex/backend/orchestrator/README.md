# Codex Orchestrator Service

The Codex Orchestrator is a high-level service that coordinates multiple AI agents to automate software development tasks. It provides a unified API for interacting with the Codex Agent Suite.

## Features

- **Unified API**: Single endpoint for all agent operations
- **Workflow Orchestration**: Coordinate multiple agents in sequence
- **State Management**: Track active sessions and request history
- **Error Handling**: Comprehensive error reporting and recovery
- **Extensible**: Easy to add new agents and workflows

## Prerequisites

- Rust 1.60+ (https://rustup.rs/)
- Cargo (Rust's package manager)
- Node.js 16+ (for testing)

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/your-org/codex.git
   cd codex
   ```

2. Build the orchestrator service:
   ```bash
   cd codex/backend/orchestrator
   cargo build --release
   ```

## Running the Service

Start the orchestrator service:

```bash
# From the orchestrator directory
cargo run --release
```

The service will start on `http://localhost:3000` by default.

## API Endpoints

### POST /api/orchestrate

Process an orchestration request with the specified agents.

**Request Body:**
```json
{
  "prompt": "Create a simple HTTP server in Node.js",
  "context": {
    "language": "javascript",
    "framework": "express"
  },
  "agent_sequence": ["spec", "code", "reviewer"],
  "options": {
    "test_framework": "jest",
    "coverage_goals": "80%"
  }
}
```

**Response:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "completed",
  "result": {
    /* Agent-specific output */
  },
  "metadata": {
    "start_time": "2024-03-20T12:00:00Z",
    "end_time": "2024-03-20T12:00:30Z",
    "duration_ms": 30000,
    "agent_sequence": ["Spec", "Code", "Reviewer"],
    "success": true
  }
}
```

## Configuration

Configuration can be provided via environment variables:

- `PORT`: Port to run the server on (default: 3000)
- `LOG_LEVEL`: Logging level (default: info)
- `MAX_CONCURRENT_REQUESTS`: Maximum concurrent requests (default: 10)

## Testing

1. Start the orchestrator service:
   ```bash
   cd codex/backend/orchestrator
   cargo run
   ```

2. In a separate terminal, run the test script:
   ```bash
   node scripts/test-orchestrator.js
   ```

## Development

### Adding New Agents

1. Implement the agent in the `codex-agents` crate
2. Add the agent to the `AgentSuite` in `src/lib.rs`
3. Update the request/response types as needed
4. Add tests for the new agent integration

### Running Tests

```bash
cargo test
```

## Deployment

### Building for Production

```bash
cargo build --release
```

The optimized binary will be available at `target/release/codex-orchestrator`.

### Docker

A `Dockerfile` is provided for containerized deployment:

```bash
docker build -t codex-orchestrator .
docker run -p 3000:3000 codex-orchestrator
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
