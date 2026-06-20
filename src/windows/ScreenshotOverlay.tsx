import React, { useState, useEffect, useRef } from 'react';
import { PhysicalPosition } from '@tauri-apps/api/dpi';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';


export const ScreenshotOverlay: React.FC = () => {
  const [isDrawing, setIsDrawing] = useState(false);
  const [startPos, setStartPos] = useState({ x: 0, y: 0 });
  const [currentPos, setCurrentPos] = useState({ x: 0, y: 0 });
  const [scaleFactor, setScaleFactor] = useState(1);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  // 1. Fetch DPI scale factor for coordinate scaling on mount
  useEffect(() => {
    const fetchScale = async () => {
      try {
        const factor = await getCurrentWindow().scaleFactor();
        setScaleFactor(factor);
      } catch (e) {
        console.error(e);
      }
    };
    fetchScale();
  }, []);

  // 2. Clear draw buffer on escape key
  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setIsDrawing(false);
        await getCurrentWindow().hide();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  // 3. Render selection box on canvas
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Resize canvas to window size
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;

    // Clear and draw dim background
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = 'rgba(0, 0, 0, 0.45)';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    if (isDrawing) {
      const x = Math.min(startPos.x, currentPos.x);
      const y = Math.min(startPos.y, currentPos.y);
      const w = Math.abs(startPos.x - currentPos.x);
      const h = Math.abs(startPos.y - currentPos.y);

      // Clear the selected rectangle (transparent hole)
      ctx.clearRect(x, y, w, h);

      // Draw dashed border around selection
      ctx.strokeStyle = '#3b82f6';
      ctx.lineWidth = 2;
      ctx.setLineDash([6, 3]);
      ctx.strokeRect(x, y, w, h);
    }
  }, [isDrawing, startPos, currentPos]);

  const handleMouseDown = (e: React.MouseEvent) => {
    setIsDrawing(true);
    setStartPos({ x: e.clientX, y: e.clientY });
    setCurrentPos({ x: e.clientX, y: e.clientY });
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!isDrawing) return;
    setCurrentPos({ x: e.clientX, y: e.clientY });
  };

  const handleMouseUp = async (e: React.MouseEvent) => {
    if (!isDrawing) return;
    setIsDrawing(false);

    const x = Math.min(startPos.x, e.clientX);
    const y = Math.min(startPos.y, e.clientY);
    const w = Math.abs(startPos.x - e.clientX);
    const h = Math.abs(startPos.y - e.clientY);

    // Hide overlay window immediately to avoid flashing
    await getCurrentWindow().hide();

    if (w < 10 || h < 10) {
      // Too small selection, cancel
      return;
    }

    try {
      // Load current config to know the target language
      const config: any = await invoke('get_config');
      
      // Perform crop & OCR translation (coordinates scaled by DPI factor)
      const physicalX = Math.round(x * scaleFactor);
      const physicalY = Math.round(y * scaleFactor);
      const physicalW = Math.round(w * scaleFactor);
      const physicalH = Math.round(h * scaleFactor);

      const ocrResultJson: string = await invoke('ocr_translate', {
        x: physicalX,
        y: physicalY,
        w: physicalW,
        h: physicalH,
        lang: config.source_lang === 'Auto' ? null : config.source_lang,
        targetLang: config.target_lang,
      });

      const data = JSON.parse(ocrResultJson);

      // Show result in floating popup window
      const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
      const popup = await WebviewWindow.getByLabel('floating_popup');
      if (popup) {
        // Move popup close to selection area
        await popup.setPosition(new PhysicalPosition(physicalX, physicalY + physicalH + 10));
        await popup.show();
        await popup.setFocus();
        await popup.emit('display-ocr-result', {
          original: data.original,
          translated: data.translated,
        });
      }
    } catch (err) {
      console.error('OCR translation failed:', err);
      // Show error in popup
      const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
      const popup = await WebviewWindow.getByLabel('floating_popup');
      if (popup) {
        await popup.show();
        await popup.setFocus();
        await popup.emit('display-ocr-result', {
          original: '[OCR Error]',
          translated: `Failed to translate screenshot: ${err}`,
        });
      }
    }
  };

  return (
    <div
      style={{
        width: '100vw',
        height: '100vh',
        margin: 0,
        padding: 0,
        overflow: 'hidden',
        cursor: 'crosshair',
        background: 'transparent',
      }}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
    >
      <canvas ref={canvasRef} style={{ display: 'block' }} />
    </div>
  );
};
