import React, { useState } from 'react';
import ReactDiffViewer from 'react-diff-viewer';
import './DiffViewer.css';

const DiffViewer = ({ patches, onRePrompt }) => {
  const [acceptedPatches, setAcceptedPatches] = useState(new Set());
  const [selectedHunks, setSelectedHunks] = useState(new Set());
  const [rePromptMode, setRePromptMode] = useState(false);

  const handleAcceptPatch = (patchIndex) => {
    setAcceptedPatches(prev => new Set([...prev, patchIndex]));
    // TODO: Apply patch to actual files
    console.log('Accepting patch:', patches[patchIndex]);
  };

  const handleRejectPatch = (patchIndex) => {
    // TODO: Remove patch from consideration
    console.log('Rejecting patch:', patches[patchIndex]);
  };

  const handleHunkSelect = (patchIndex, hunkIndex) => {
    if (rePromptMode) {
      const hunkKey = `${patchIndex}-${hunkIndex}`;
      setSelectedHunks(prev => {
        const newSet = new Set(prev);
        if (newSet.has(hunkKey)) {
          newSet.delete(hunkKey);
        } else {
          newSet.add(hunkKey);
        }
        return newSet;
      });
    }
  };

  const handleRePromptModeToggle = () => {
    setRePromptMode(!rePromptMode);
    setSelectedHunks(new Set()); // Clear selections when toggling mode
  };

  const handleRePromptSubmit = () => {
    if (selectedHunks.size === 0) return;

    // Convert selected hunks to patch context
    const selectedPatches = Array.from(selectedHunks).map(hunkKey => {
      const [patchIndex, hunkIndex] = hunkKey.split('-').map(Number);
      return {
        patchIndex,
        patch: patches[patchIndex],
        hunkIndex
      };
    });

    onRePrompt(selectedPatches);
    setRePromptMode(false);
    setSelectedHunks(new Set());
  };

  const renderDiffContent = (patch, patchIndex) => {
    // For now, we'll use a simplified approach
    // In a real implementation, we'd need to parse the diff and identify individual hunks
    const lines = patch.newContent.split('\n');

    return (
      <div className="diff-content">
        {lines.map((line, lineIndex) => {
          const hunkKey = `${patchIndex}-${lineIndex}`;
          const isSelected = selectedHunks.has(hunkKey);
          const isAdded = line.startsWith('+');
          const isRemoved = line.startsWith('-');

          return (
            <div
              key={lineIndex}
              className={`diff-line ${isAdded ? 'added' : ''} ${isRemoved ? 'removed' : ''} ${isSelected && rePromptMode ? 'selected' : ''}`}
              onClick={() => handleHunkSelect(patchIndex, lineIndex)}
              style={{ cursor: rePromptMode ? 'pointer' : 'default' }}
            >
              <span className="line-number">{lineIndex + 1}</span>
              <span className="line-content">{line}</span>
            </div>
          );
        })}
      </div>
    );
  };

  return (
    <div className="diff-viewer">
      <div className="diff-viewer-header">
        <h2>Generated Patches</h2>
        <div className="diff-actions">
          <button
            onClick={handleRePromptModeToggle}
            className={`re-prompt-toggle ${rePromptMode ? 'active' : ''}`}
            title="Toggle re-prompt mode to select specific changes"
          >
            {rePromptMode ? '✅ Re-prompt Mode' : '🔄 Re-prompt'}
          </button>
          {selectedHunks.size > 0 && (
            <button
              onClick={handleRePromptSubmit}
              className="re-prompt-submit"
              title={`Re-prompt with ${selectedHunks.size} selected change(s)`}
            >
              Re-prompt Selected ({selectedHunks.size})
            </button>
          )}
        </div>
      </div>

      {patches.map((patch, index) => (
        <div key={index} className="patch-container">
          <div className="patch-header">
            <h3>{patch.filePath}</h3>
            <div className="patch-actions">
              <button
                onClick={() => handleRePrompt(index)}
                className="re-prompt-button"
                title="Re-prompt for this entire file"
              >
                🔄 File
              </button>
              <button
                onClick={() => handleRejectPatch(index)}
                className="reject-button"
                disabled={acceptedPatches.has(index)}
              >
                Reject
              </button>
              <button
                onClick={() => handleAcceptPatch(index)}
                className="accept-button"
                disabled={acceptedPatches.has(index)}
              >
                {acceptedPatches.has(index) ? '✓ Accepted' : 'Accept'}
              </button>
            </div>
          </div>

          {patch.explanation && (
            <div className="patch-explanation">
              <strong>Explanation:</strong> {patch.explanation}
            </div>
          )}

          {renderDiffContent(patch, index)}
        </div>
      ))}

      {patches.length > 0 && (
        <div className="bulk-actions">
          <button
            onClick={() => {
              // TODO: Apply all accepted patches
              console.log('Applying all accepted patches');
            }}
            className="apply-all-button"
          >
            Apply All Accepted Changes
          </button>
        </div>
      )}
    </div>
  );
};

export default DiffViewer;
