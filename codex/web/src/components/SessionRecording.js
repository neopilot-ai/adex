import React from 'react';
import { useSessionRecording } from '../contexts/SessionRecordingContext';
import './SessionRecording.css';

const SessionRecording = () => {
  const {
    isRecording,
    sessionId,
    sessionData,
    startRecording,
    stopRecording,
    exportSession,
    clearSession
  } = useSessionRecording();

  const handleExport = () => {
    const session = exportSession();
    if (session) {
      const blob = new Blob([JSON.stringify(session, null, 2)], {
        type: 'application/json'
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `codex-session-${sessionId}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    }
  };

  const handlePlayback = () => {
    // TODO: Implement session playback interface
    alert('Session playback interface would open here');
  };

  return (
    <div className="session-recording">
      <div className="recording-controls">
        {!isRecording ? (
          <button
            onClick={startRecording}
            className="start-recording-btn"
            title="Start recording session for demos and audits"
          >
            🔴 Start Recording
          </button>
        ) : (
          <button
            onClick={stopRecording}
            className="stop-recording-btn"
            title="Stop recording session"
          >
            ⏹️ Stop Recording
          </button>
        )}

        {isRecording && (
          <div className="recording-status">
            <span className="recording-indicator">● REC</span>
            <span className="session-info">
              Session: {sessionId} ({sessionData.length} events)
            </span>
          </div>
        )}
      </div>

      {sessionData.length > 0 && !isRecording && (
        <div className="session-actions">
          <button
            onClick={handlePlayback}
            className="playback-btn"
            title="Playback recorded session"
          >
            ▶️ Playback Session
          </button>
          <button
            onClick={handleExport}
            className="export-btn"
            title="Export session data as JSON"
          >
            📥 Export Session
          </button>
          <button
            onClick={clearSession}
            className="clear-btn"
            title="Clear recorded session data"
          >
            🗑️ Clear Session
          </button>
        </div>
      )}

      {sessionData.length > 0 && (
        <div className="session-stats">
          <div className="stat-item">
            <span className="stat-label">Events:</span>
            <span className="stat-value">{sessionData.length}</span>
          </div>
          <div className="stat-item">
            <span className="stat-label">Duration:</span>
            <span className="stat-value">
              {Math.round((Date.now() - sessionData[0]?.timestamp ? new Date(sessionData[0].timestamp).getTime() : Date.now()) / 1000)}s
            </span>
          </div>
        </div>
      )}
    </div>
  );
};

export default SessionRecording;
