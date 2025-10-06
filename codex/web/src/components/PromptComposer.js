import React, { useState, useRef } from 'react';
import './PromptComposer.css';

const PromptComposer = ({ onSubmit, isLoading }) => {
  const [prompt, setPrompt] = useState('');
  const [attachments, setAttachments] = useState([]);
  const [dragActive, setDragActive] = useState(false);
  const fileInputRef = useRef(null);

  const handleSubmit = (e) => {
    e.preventDefault();
    if (prompt.trim() && !isLoading) {
      onSubmit(prompt, attachments);
      setPrompt('');
      setAttachments([]);
    }
  };

  const handleFileUpload = (files) => {
    const newAttachments = Array.from(files).map(file => ({
      id: Date.now() + Math.random(),
      name: file.name,
      type: file.type,
      content: file
    }));
    setAttachments(prev => [...prev, ...newAttachments]);
  };

  const handleDrag = (e) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.type === "dragenter" || e.type === "dragover") {
      setDragActive(true);
    } else if (e.type === "dragleave") {
      setDragActive(false);
    }
  };

  const handleDrop = (e) => {
    e.preventDefault();
    e.stopPropagation();
    setDragActive(false);
    if (e.dataTransfer.files && e.dataTransfer.files[0]) {
      handleFileUpload(e.dataTransfer.files);
    }
  };

  const removeAttachment = (id) => {
    setAttachments(prev => prev.filter(att => att.id !== id));
  };

  const handleUrlAdd = (url) => {
    if (url.trim()) {
      const newAttachment = {
        id: Date.now() + Math.random(),
        name: url,
        type: 'url',
        content: url
      };
      setAttachments(prev => [...prev, newAttachment]);
    }
  };

  return (
    <div className="prompt-composer">
      <form onSubmit={handleSubmit}>
        <div className="prompt-input-section">
          <textarea
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            placeholder="Describe the feature or change you want to implement..."
            disabled={isLoading}
            className="prompt-textarea"
          />
        </div>

        <div className="attachments-section">
          <div className="url-input">
            <input
              type="url"
              placeholder="Add URL context..."
              onKeyPress={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  handleUrlAdd(e.target.value);
                  e.target.value = '';
                }
              }}
              disabled={isLoading}
            />
          </div>

          <div
            className={`file-drop-zone ${dragActive ? 'drag-active' : ''}`}
            onDragEnter={handleDrag}
            onDragLeave={handleDrag}
            onDragOver={handleDrag}
            onDrop={handleDrop}
            onClick={() => fileInputRef.current?.click()}
          >
            <input
              ref={fileInputRef}
              type="file"
              multiple
              onChange={(e) => handleFileUpload(e.target.files)}
              style={{ display: 'none' }}
              disabled={isLoading}
            />
            <div className="drop-zone-content">
              <span>Drag & drop files here or click to browse</span>
            </div>
          </div>

          {attachments.length > 0 && (
            <div className="attachments-list">
              <h4>Attachments:</h4>
              {attachments.map(attachment => (
                <div key={attachment.id} className="attachment-item">
                  <span className="attachment-name">
                    {attachment.type === 'url' ? '🔗' : '📄'} {attachment.name}
                  </span>
                  <button
                    type="button"
                    onClick={() => removeAttachment(attachment.id)}
                    className="remove-attachment"
                    disabled={isLoading}
                  >
                    ×
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="submit-section">
          <button
            type="submit"
            disabled={!prompt.trim() || isLoading}
            className="submit-button"
          >
            {isLoading ? 'Generating...' : 'Generate Code'}
          </button>
        </div>
      </form>
    </div>
  );
};

export default PromptComposer;
