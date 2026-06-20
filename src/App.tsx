import { useState, useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { MainWindow } from './windows/MainWindow';
import { ScreenshotOverlay } from './windows/ScreenshotOverlay';
import { FloatingPopup } from './windows/FloatingPopup';
import { SettingsWindow } from './windows/SettingsWindow';

export default function App() {
  const [label, setLabel] = useState<string | null>(null);

  useEffect(() => {
    // 1. Get the Tauri window label at runtime
    try {
      const windowLabel = getCurrentWindow().label;
      setLabel(windowLabel);
    } catch (e) {
      console.error("Failed to get Tauri window label:", e);
      setLabel('main'); // Fallback for standard browser preview
    }
  }, []);

  // 2. Render appropriate panel depending on window label
  if (label === null) {
    return null; // Don't flash layout during window detection
  }

  switch (label) {
    case 'main':
      return <MainWindow />;
    case 'screenshot_overlay':
      return <ScreenshotOverlay />;
    case 'floating_popup':
      return <FloatingPopup />;
    case 'settings':
      return <SettingsWindow />;
    default:
      return <MainWindow />;
  }
}
