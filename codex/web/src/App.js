import React, { useState } from 'react';
import './App.css';
import PromptComposer from './components/PromptComposer';
import DiffViewer from './components/DiffViewer';

function App() {
  const [patches, setPatches] = useState([]);
  const [isLoading, setIsLoading] = useState(false);

  const handlePromptSubmit = async (prompt, attachments) => {
    setIsLoading(true);
    try {
      // TODO: Connect to backend orchestrator
      console.log('Prompt submitted:', prompt, 'Attachments:', attachments);

      // Mock response for now
      const mockPatches = [
        {
          filePath: 'src/components/Example.js',
          oldContent: 'console.log("old code");',
          newContent: 'console.log("new code with improvements");',
          explanation: 'Updated the console.log statement with better messaging'
        }
      ];

      setPatches(mockPatches);
    } catch (error) {
      console.error('Error submitting prompt:', error);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="App">
      <header className="App-header">
        <h1>Codex - Agentic Development Environment</h1>
      </header>
      <main className="App-main">
        <div className="prompt-section">
          <PromptComposer onSubmit={handlePromptSubmit} isLoading={isLoading} />
        </div>
        {patches.length > 0 && (
          <div className="diff-section">
            <h2>Generated Patches</h2>
            <DiffViewer patches={patches} />
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
