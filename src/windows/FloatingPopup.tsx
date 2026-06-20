import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { TitleBar } from '../components/TitleBar';

export const FloatingPopup: React.FC = () => {
  const [original, setOriginal] = useState('');
  const [translated, setTranslated] = useState('');
  const [loading, setLoading] = useState(false);

  // 1. Listen to background clipboard triggers and screenshot OCR triggers
  useEffect(() => {
    // Highlights trigger:
    const unlistenHighlight = listen<string>('translate-highlight', async (event) => {
      const sourceText = event.payload;
      setOriginal(sourceText);
      setTranslated('');
      setLoading(true);

      try {
        const config: any = await invoke('get_config');
        const res: string = await invoke('translate', {
          text: sourceText,
          source: config.source_lang,
          target: config.target_lang,
        });
        setTranslated(res);
      } catch (err) {
        console.error(err);
        setTranslated(`[Translation error: ${err}]`);
      } finally {
        setLoading(false);
      }
    });

    // Screenshot OCR triggers:
    const unlistenOcr = listen<{ original: string; translated: string }>('display-ocr-result', (event) => {
      setOriginal(event.payload.original);
      setTranslated(event.payload.translated);
      setLoading(false);
    });

    return () => {
      unlistenHighlight.then((f) => f());
      unlistenOcr.then((f) => f());
    };
  }, []);

  // 2. Auto-hide when user clicks away
  useEffect(() => {
    const handleBlur = async () => {
      try {
        await getCurrentWindow().hide();
      } catch (e) {
        console.error(e);
      }
    };
    
    window.addEventListener('blur', handleBlur);
    return () => window.removeEventListener('blur', handleBlur);
  }, []);

  const handleCopy = async () => {
    if (!translated) return;
    try {
      await navigator.clipboard.writeText(translated);
      await getCurrentWindow().hide(); // Dismiss popup on copy
    } catch (e) {
      console.error(e);
    }
  };

  const handleSpeak = () => {
    if (!translated) return;
    const utterance = new SpeechSynthesisUtterance(translated);
    window.speechSynthesis.speak(utterance);
  };

  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      height: '100vh',
      background: 'var(--bg-secondary)',
      border: '1px solid var(--border-color)',
      borderRadius: '8px',
      overflow: 'hidden',
      boxSizing: 'border-box',
      boxShadow: 'var(--shadow-lg)'
    }}>
      <TitleBar title="LangFlow Quick Translate" showMinimize={false} />

      <div style={{ flex: 1, padding: '44px 12px 12px 12px', overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '8px', boxSizing: 'border-box' }}>
        
        {/* Original Text Snippet */}
        <div style={{
          fontSize: '11px',
          color: 'var(--text-muted)',
          maxHeight: '40px',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap',
          borderBottom: '1px solid rgba(255,255,255,0.03)',
          paddingBottom: '4px'
        }}>
          {original || 'No text copied yet...'}
        </div>

        {/* Translation Output Box */}
        <div style={{ flex: 1, fontSize: '13px', lineHeight: '1.5', color: 'var(--text-primary)', overflowY: 'auto' }}>
          {loading ? (
            <div style={{ color: 'var(--text-muted)', fontStyle: 'italic' }}>Translating text...</div>
          ) : (
            translated || <div style={{ color: 'var(--text-muted)', fontSize: '12px' }}>Highlight text anywhere and press hotkey to translate.</div>
          )}
        </div>

        {/* Action Controls */}
        {translated && !loading && (
          <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '6px', borderTop: '1px solid var(--border-color)', paddingTop: '8px' }}>
            <button className="btn-secondary" style={{ padding: '3px 8px', fontSize: '11px' }} onClick={handleSpeak}>
              🔊 Speak
            </button>
            <button className="btn-primary" style={{ padding: '3px 8px', fontSize: '11px' }} onClick={handleCopy}>
              Copy & Close
            </button>
          </div>
        )}

      </div>
    </div>
  );
};
