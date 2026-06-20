import React from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';

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
      // For utility application, close request hides the window rather than destroying it
      await getCurrentWindow().hide();
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
