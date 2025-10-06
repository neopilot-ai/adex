import React, { useState } from 'react';
import ReactDiffViewer from 'react-diff-viewer';
import './DiffViewer.css';

const DiffViewer = ({ patches }) => {
  const [acceptedPatches, setAcceptedPatches] = useState(new Set());

  const handleAcceptPatch = (patchIndex) => {
    setAcceptedPatches(prev => new Set([...prev, patchIndex]));
    // TODO: Apply patch to actual files
    console.log('Accepting patch:', patches[patchIndex]);
  };

  const handleRejectPatch = (patchIndex) => {
    // TODO: Remove patch from consideration
    console.log('Rejecting patch:', patches[patchIndex]);
  };

  const handleRePrompt = (patchIndex) => {
    // TODO: Open re-prompt dialog scoped to this patch
    console.log('Re-prompt for patch:', patches[patchIndex]);
  };

  return (
    <div className="diff-viewer">
      {patches.map((patch, index) => (
        <div key={index} className="patch-container">
          <div className="patch-header">
            <h3>{patch.filePath}</h3>
            <div className="patch-actions">
              <button
                onClick={() => handleRePrompt(index)}
                className="re-prompt-button"
                title="Re-prompt for this specific change"
              >
                🔄
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

          <div className="diff-content">
            <ReactDiffViewer
              oldValue={patch.oldContent}
              newValue={patch.newContent}
              splitView={true}
              disableWordDiff={false}
              hideLineNumbers={false}
            />
          </div>
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
