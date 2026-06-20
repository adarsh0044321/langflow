import React from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';

interface TitleBarProps {
  title: string;
  showMinimize?: boolean;
}

export const TitleBar: React.FC<TitleBarProps> = ({ title, showMinimize = true }) => {
  const handleMinimize = async () => {
    try {
      await getCurrentWindow().minimize();
    } catch (e) {
      console.error(e);
    }
  };

  const handleClose = async () => {
    try {
      const label = getCurrentWindow().label;
      await invoke('hide_window', { label });
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div className="window-drag-region" data-tauri-drag-region>
      <div className="window-drag-title">{title}</div>
      <div className="window-controls">
        {showMinimize && (
          <button className="window-control-btn" onClick={handleMinimize} title="Minimize">
            &#9472;
          </button>
        )}
        <button className="window-control-btn close" onClick={handleClose} title="Close to Tray">
          &#10005;
        </button>
      </div>
    </div>
  );
};
