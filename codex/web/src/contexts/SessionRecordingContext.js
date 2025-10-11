import React, { createContext, useContext, useState, useCallback, useRef } from 'react';

const SessionRecordingContext = createContext();

export const useSessionRecording = () => {
  const context = useContext(SessionRecordingContext);
  if (!context) {
    throw new Error('useSessionRecording must be used within a SessionRecordingProvider');
  }
  return context;
};

export const SessionRecordingProvider = ({ children }) => {
  const [isRecording, setIsRecording] = useState(false);
  const [sessionData, setSessionData] = useState([]);
  const [sessionId, setSessionId] = useState(null);
  const startTimeRef = useRef(null);

  const startRecording = useCallback(() => {
    const newSessionId = `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    setSessionId(newSessionId);
    setSessionData([]);
    setIsRecording(true);
    startTimeRef.current = Date.now();

    // Record session start
    recordEvent('session_start', {
      sessionId: newSessionId,
      timestamp: new Date().toISOString(),
      userAgent: navigator.userAgent
    });

    console.log('Session recording started:', newSessionId);
  }, []);

  const stopRecording = useCallback(() => {
    setIsRecording(false);

    // Record session end
    recordEvent('session_end', {
      sessionId,
      timestamp: new Date().toISOString(),
      duration: Date.now() - startTimeRef.current,
      totalEvents: sessionData.length
    });

    console.log('Session recording stopped:', sessionId);
  }, [sessionId, sessionData.length]);

  const recordEvent = useCallback((eventType, data) => {
    if (!isRecording) return;

    const event = {
      id: `${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      sessionId,
      eventType,
      timestamp: new Date().toISOString(),
      data,
      relativeTime: startTimeRef.current ? Date.now() - startTimeRef.current : 0
    };

    setSessionData(prev => [...prev, event]);
  }, [isRecording, sessionId]);

  const exportSession = useCallback(() => {
    if (sessionData.length === 0) return null;

    return {
      sessionId,
      startTime: startTimeRef.current,
      duration: Date.now() - startTimeRef.current,
      events: sessionData,
      metadata: {
        exportedAt: new Date().toISOString(),
        totalEvents: sessionData.length,
        userAgent: navigator.userAgent
      }
    };
  }, [sessionId, sessionData]);

  const clearSession = useCallback(() => {
    setSessionData([]);
    setSessionId(null);
    setIsRecording(false);
    startTimeRef.current = null;
  }, []);

  const value = {
    isRecording,
    sessionId,
    sessionData,
    startRecording,
    stopRecording,
    recordEvent,
    exportSession,
    clearSession
  };

  return (
    <SessionRecordingContext.Provider value={value}>
      {children}
    </SessionRecordingContext.Provider>
  );
};
