import React, { useState, useEffect, useRef } from 'react';
import './StreamingDiffViewer.css';

const StreamingDiffViewer = ({ isStreaming, onStreamingComplete, onStreamingError }) => {
  const [streamingContent, setStreamingContent] = useState('');
  const [isConnected, setIsConnected] = useState(false);
  const eventSourceRef = useRef(null);

  useEffect(() => {
    if (isStreaming && !eventSourceRef.current) {
      startStreaming();
    } else if (!isStreaming && eventSourceRef.current) {
      stopStreaming();
    }

    return () => {
      if (eventSourceRef.current) {
        eventSourceRef.current.close();
      }
    };
  }, [isStreaming]);

  const startStreaming = () => {
    try {
      // Connect to the streaming endpoint
      const eventSource = new EventSource('http://localhost:3000/api/v1/orchestrate/stream');
      eventSourceRef.current = eventSource;

      eventSource.onopen = () => {
        setIsConnected(true);
        console.log('Streaming connection established');
      };

      eventSource.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);

          if (data.type === 'chunk') {
            setStreamingContent(prev => prev + data.content);
          } else if (data.type === 'patch') {
            // Handle complete patch data
            setStreamingContent(data.content);
          } else if (data.type === 'complete') {
            // Streaming completed
            setIsConnected(false);
            if (onStreamingComplete) {
              onStreamingComplete(data.result);
            }
          }
        } catch (error) {
          console.error('Error parsing streaming data:', error);
        }
      };

      eventSource.onerror = (error) => {
        console.error('Streaming error:', error);
        setIsConnected(false);
        if (onStreamingError) {
          onStreamingError(error);
        }
      };

    } catch (error) {
      console.error('Failed to start streaming:', error);
      if (onStreamingError) {
        onStreamingError(error);
      }
    }
  };

  const stopStreaming = () => {
    if (eventSourceRef.current) {
      eventSourceRef.current.close();
      eventSourceRef.current = null;
      setIsConnected(false);
    }
  };

  const formatStreamingContent = (content) => {
    // Try to format as JSON if possible, otherwise show as plain text
    try {
      const parsed = JSON.parse(content);
      return JSON.stringify(parsed, null, 2);
    } catch {
      return content;
    }
  };

  return (
    <div className="streaming-diff-viewer">
      <div className="streaming-header">
        <h3>Live Generation</h3>
        <div className={`connection-status ${isConnected ? 'connected' : 'disconnected'}`}>
          {isConnected ? '🟢 Live' : '⚫ Idle'}
        </div>
      </div>

      {isStreaming && (
        <div className="streaming-progress">
          <div className="progress-bar">
            <div className="progress-fill"></div>
          </div>
          <span>Generating code...</span>
        </div>
      )}

      <div className="streaming-content">
        <pre className="content-display">
          <code>{formatStreamingContent(streamingContent)}</code>
        </pre>
      </div>

      {streamingContent && (
        <div className="streaming-actions">
          <button
            onClick={() => navigator.clipboard.writeText(streamingContent)}
            className="copy-button"
          >
            📋 Copy Content
          </button>
        </div>
      )}
    </div>
  );
};

export default StreamingDiffViewer;
