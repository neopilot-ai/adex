import React, { useState } from 'react';
import './App.css';
import PromptComposer from './components/PromptComposer';
import DiffViewer from './components/DiffViewer';
import StreamingDiffViewer from './components/StreamingDiffViewer';
import ErrorBoundary from './components/ErrorBoundary';
import SessionRecording from './components/SessionRecording';
import { SessionRecordingProvider, useSessionRecording } from './contexts/SessionRecordingContext';

function AppContent() {
  const [patches, setPatches] = useState([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isStreaming, setIsStreaming] = useState(false);
  const [streamingComplete, setStreamingComplete] = useState(false);
  const [error, setError] = useState(null);
  const [retryCount, setRetryCount] = useState(0);
  const { recordEvent } = useSessionRecording();

  const handlePromptSubmit = async (prompt, attachments) => {
    setIsLoading(true);
    setIsStreaming(true);
    setStreamingComplete(false);
    setPatches([]);
    setError(null);

    // Record the prompt submission
    recordEvent('prompt_submitted', {
      promptLength: prompt.length,
      attachmentCount: attachments.length,
      hasAttachments: attachments.length > 0
    });

    try {
      // Prepare the request payload for streaming
      const requestData = {
        prompt: prompt,
        context: attachments.length > 0 ? {
          attachments: attachments.map(att => ({
            name: att.name,
            type: att.type,
            content: att.type === 'url' ? att.content : 'file_content_placeholder'
          }))
        } : undefined,
        agent_sequence: ["spec", "code", "reviewer"],
        options: {
          streaming: true
        }
      };

      // Record the API request
      recordEvent('api_request_sent', {
        endpoint: '/api/v1/orchestrate',
        method: 'POST',
        hasContext: !!requestData.context
      });

      // For now, simulate streaming with regular API call
      // In a full implementation, this would connect to the streaming endpoint
      const response = await fetch('http://localhost:3000/api/v1/orchestrate', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(requestData)
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const result = await response.json();

      // Record successful response
      recordEvent('api_response_received', {
        status: response.status,
        hasResult: !!result.result
      });

      // Convert the backend response to the expected patch format
      if (result.result && result.result.patch) {
        const patches = parsePatchResponse(result.result.patch);
        setPatches(patches);

        // Record patch generation
        recordEvent('patches_generated', {
          patchCount: patches.length
        });
      } else {
        const mockPatches = [
          {
            filePath: 'src/components/Example.js',
            oldContent: 'console.log("old code");',
            newContent: JSON.stringify(result.result, null, 2),
            explanation: 'Generated response from orchestrator'
          }
        ];
        setPatches(mockPatches);

        // Record fallback patch generation
        recordEvent('fallback_patches_generated', {
          reason: 'No patch in response'
        });
      }

      setStreamingComplete(true);

    } catch (error) {
      console.error('Error submitting prompt:', error);
      setError(error);
      setRetryCount(prev => prev + 1);

      // Record the error
      recordEvent('prompt_error', {
        errorType: error.name,
        errorMessage: error.message,
        retryCount: retryCount + 1
      });
    } finally {
      setIsLoading(false);
      setIsStreaming(false);
    }
  };

  const handleRePrompt = (selectedPatches) => {
    // Record the re-prompt action
    recordEvent('reprompt_requested', {
      selectedPatchCount: selectedPatches.length,
      patchIndices: selectedPatches.map(p => p.patchIndex)
    });

    // For now, just log the selected patches for re-prompting
    console.log('Re-prompting with selected patches:', selectedPatches);

    // Extract the original prompt and add context about the selected changes
    const basePrompt = "Please refine the following code changes based on the selected sections:";
    const contextInfo = selectedPatches.map(({ patch, patchIndex, hunkIndex }) =>
      `File: ${patch.filePath}, Change ${patchIndex + 1}, Section ${hunkIndex + 1}`
    ).join('\n');

    const refinedPrompt = `${basePrompt}\n\nSelected changes:\n${contextInfo}\n\nOriginal request: ${prompt}`;

    // TODO: Open re-prompt dialog with refinedPrompt
    alert(`Re-prompt dialog would open with refined prompt for ${selectedPatches.length} selected change(s)`);
  };

  const handleRetry = () => {
    setError(null);
    setRetryCount(0);

    // Record retry action
    recordEvent('retry_requested', {
      previousRetryCount: retryCount
    });
  };

  const clearResults = () => {
    setPatches([]);
    setError(null);
    setStreamingComplete(false);

    // Record clear action
    recordEvent('results_cleared', {
      hadPatches: patches.length > 0,
      hadError: !!error
    });
  };

  const handleStreamingComplete = (result) => {
    setIsStreaming(false);
    setStreamingComplete(true);
    // Process the streaming result
    if (result && result.patch) {
      const patches = parsePatchResponse(result.patch);
      setPatches(patches);
    }
  };

  const handleStreamingError = (error) => {
    setIsStreaming(false);
    setIsLoading(false);
    console.error('Streaming error:', error);
    alert('Streaming error occurred. Please try again.');
  };

  // Helper function to parse patch response
  const parsePatchResponse = (patchContent) => {
    // This is a simplified patch parser - in a real implementation,
    // you'd want more sophisticated parsing
    try {
      const lines = patchContent.split('\n');
      const patches = [];
      let currentFile = null;
      let oldContent = '';
      let newContent = '';

      for (const line of lines) {
        if (line.startsWith('+++') || line.startsWith('---')) {
          // Skip diff headers
          continue;
        } else if (line.startsWith('@@')) {
          // New hunk
          if (currentFile && oldContent && newContent) {
            patches.push({
              filePath: currentFile,
              oldContent: oldContent.trim(),
              newContent: newContent.trim(),
              explanation: 'Code changes generated by orchestrator'
            });
          }
          // Extract filename from hunk header if possible
          const match = line.match(/@@.*?\+.*? (.*?)\s*@@/);
          if (match) {
            currentFile = match[1];
          }
          oldContent = '';
          newContent = '';
        } else if (line.startsWith('+') && !line.startsWith('+++')) {
          newContent += line.substring(1) + '\n';
        } else if (line.startsWith('-') && !line.startsWith('---')) {
          oldContent += line.substring(1) + '\n';
        } else if (line.startsWith(' ')) {
          oldContent += line.substring(1) + '\n';
          newContent += line.substring(1) + '\n';
        }
      }

      // Add the final patch if exists
      if (currentFile && (oldContent || newContent)) {
        patches.push({
          filePath: currentFile,
          oldContent: oldContent.trim(),
          newContent: newContent.trim(),
          explanation: 'Code changes generated by orchestrator'
        });
      }

      return patches.length > 0 ? patches : [{
        filePath: 'response.txt',
        oldContent: 'No file changes detected',
        newContent: patchContent,
        explanation: 'Orchestrator response'
      }];
    } catch (error) {
      // If parsing fails, return the raw response
      return [{
        filePath: 'response.txt',
        oldContent: 'Error parsing response',
        newContent: patchContent || 'No content received',
        explanation: 'Raw orchestrator response'
      }];
    }
  };

  return (
    <ErrorBoundary>
      <SessionRecordingProvider>
        <AppContent />
      </SessionRecordingProvider>
    </ErrorBoundary>
  );
}

export default App;
