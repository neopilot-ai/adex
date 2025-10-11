// test-orchestrator.js
const axios = require('axios');

async function testOrchestrator() {
  try {
    const response = await axios.post('http://localhost:3000/api/orchestrate', {
      prompt: 'Create a simple HTTP server in Node.js',
      context: {
        language: 'javascript',
        framework: 'express',
      },
      agent_sequence: ['spec', 'code', 'reviewer'],
      options: {
        test_framework: 'jest',
        coverage_goals: '80%',
      },
    });

    console.log('Orchestration successful!');
    console.log('Request ID:', response.data.request_id);
    console.log('Status:', response.data.status);
    console.log('Result:', JSON.stringify(response.data.result, null, 2));
  } catch (error) {
    console.error('Error:', error.response?.data || error.message);
  }
}

testOrchestrator();
