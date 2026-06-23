import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { TitleBar } from '../components/TitleBar';

interface HistoryEntry {
  id: string;
  source_lang: string;
  target_lang: string;
  original_text: string;
  translated_text: string;
  is_favorite: boolean;
  created_at: string;
}

const LANGUAGES = [
  { code: 'en', name: 'English' },
  { code: 'ja', name: 'Japanese' },
  { code: 'zh', name: 'Chinese' },
  { code: 'ko', name: 'Korean' },
  { code: 'de', name: 'German' },
  { code: 'fr', name: 'French' },
  { code: 'es', name: 'Spanish' },
  { code: 'ru', name: 'Russian' }
];

const detectLanguage = (text: string): string => {
  if (!text) return 'en';
  // Check for Japanese characters (Hiragana, Katakana, Kanji)
  if (/[\u3040-\u309F\u30A0-\u30FF\u4E00-\u9FBF]/.test(text)) {
    return 'ja';
  }
  // Check for Korean characters (Hangul)
  if (/[\uAC00-\uD7A3]/.test(text)) {
    return 'ko';
  }
  // Check for Cyrillic characters (Russian)
  if (/[\u0400-\u04FF]/.test(text)) {
    return 'ru';
  }
  // Check for Chinese characters (Hanzi)
  if (/[\u4E00-\u9FFF]/.test(text)) {
    return 'zh';
  }
  return 'en'; // Default fallback
};

export const MainWindow: React.FC = () => {
  const [sourceText, setSourceText] = useState('');
  const [translatedText, setTranslatedText] = useState('');
  const [sourceLang, setSourceLang] = useState('Auto');
  const [targetLang, setTargetLang] = useState('ja');
  const [isTranslating, setIsTranslating] = useState(false);
  const [showHistory, setShowHistory] = useState(false);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [historySearch, setHistorySearch] = useState('');
  
  const typingTimeoutRef = useRef<any>(null);

  // 1. Debounced translation trigger
  useEffect(() => {
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
    }

    if (!sourceText.trim()) {
      setTranslatedText('');
      return;
    }

    setIsTranslating(true);
    typingTimeoutRef.current = setTimeout(() => {
      handleTranslate();
    }, 450); // 450ms debounce

    return () => {
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
      }
    };
  }, [sourceText, sourceLang, targetLang]);

  const handleTranslate = async () => {
    try {
      const result: string = await invoke('translate', {
        text: sourceText,
        source: sourceLang,
        target: targetLang,
      });
      setTranslatedText(result);
    } catch (error) {
      console.error(error);
      setTranslatedText(`[Error: ${error}]`);
    } finally {
      setIsTranslating(false);
    }
  };

  const handleSwap = () => {
    let currentSource = sourceLang;
    if (currentSource === 'Auto') {
      currentSource = detectLanguage(sourceText);
    }

    const newSource = targetLang;
    let newTarget = currentSource;
    
    // Ensure source and target are not identical
    if (newSource === newTarget) {
      newTarget = newSource === 'en' ? 'ja' : 'en';
    }

    setSourceLang(newSource);
    setTargetLang(newTarget);
    
    // Swap texts instantly for a snappy UI experience
    const oldSourceText = sourceText;
    setSourceText(translatedText);
    setTranslatedText(oldSourceText);
  };

  const handleClear = () => {
    setSourceText('');
    setTranslatedText('');
  };

  const handleCopy = async () => {
    if (!translatedText) return;
    try {
      await navigator.clipboard.writeText(translatedText);
    } catch (e) {
      console.error(e);
    }
  };

  const handleSpeak = (text: string) => {
    if (!text) return;
    const utterance = new SpeechSynthesisUtterance(text);
    // Auto-detect voice if possible
    window.speechSynthesis.speak(utterance);
  };

  const loadHistory = async () => {
    try {
      const list: HistoryEntry[] = await invoke('get_history', { search: historySearch || null });
      setHistory(list);
    } catch (e) {
      console.error(e);
    }
  };

  // Reload history when drawer opens or search query changes
  useEffect(() => {
    if (showHistory) {
      loadHistory();
    }
  }, [showHistory, historySearch]);

  const handleToggleFavorite = async (id: string) => {
    try {
      await invoke('toggle_favorite', { id });
      loadHistory();
    } catch (e) {
      console.error(e);
    }
  };

  const handleDeleteHistory = async (id: string) => {
    try {
      await invoke('delete_history', { id });
      loadHistory();
    } catch (e) {
      console.error(e);
    }
  };

  const handleClearAllHistory = async () => {
    if (confirm('Clear all translation history?')) {
      try {
        await invoke('clear_history');
        loadHistory();
      } catch (e) {
        console.error(e);
      }
    }
  };

  const handleOpenSettings = async () => {
    try {
      await invoke('request_memory_trim'); // Reclaim memory before opening another window
      await invoke('show_window', { label: 'settings' });
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', position: 'relative' }}>
      <TitleBar title="LangFlow" />

      {/* Main Content Area */}
      <div style={{ display: 'flex', flex: 1, padding: '48px 12px 12px 12px', gap: '8px', boxSizing: 'border-box', overflow: 'hidden' }}>
        
        {/* Left Side: Source */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', borderRadius: '6px', overflow: 'hidden' }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '6px 8px', borderBottom: '1px solid var(--border-color)', background: 'rgba(255,255,255,0.01)' }}>
            <select value={sourceLang} onChange={(e) => setSourceLang(e.target.value)}>
              <option value="Auto">Auto Detect</option>
              {LANGUAGES.map((l) => (
                <option key={l.code} value={l.code}>{l.name}</option>
              ))}
            </select>
            <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
              {sourceText.length} chars
            </div>
          </div>
          <div style={{ flex: 1 }}>
            <textarea
              placeholder="Type or paste text to translate..."
              value={sourceText}
              onChange={(e) => setSourceText(e.target.value)}
            />
          </div>
          <div style={{ display: 'flex', gap: '4px', padding: '6px 8px', borderTop: '1px solid var(--border-color)', background: 'rgba(255,255,255,0.01)' }}>
            <button className="btn-secondary" style={{ padding: '3px 8px', fontSize: '11px' }} onClick={handleClear} disabled={!sourceText}>
              Clear
            </button>
            <button className="btn-secondary" style={{ padding: '3px 8px', fontSize: '11px' }} onClick={() => handleSpeak(sourceText)} disabled={!sourceText}>
              🔊 Speak
            </button>
          </div>
        </div>

        {/* Swap Button */}
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          <button
            className="btn-secondary"
            style={{ width: '28px', height: '28px', padding: 0, display: 'flex', alignItems: 'center', justifyContent: 'center', borderRadius: '50%', border: '1px solid var(--border-color)' }}
            onClick={handleSwap}
            title="Swap Languages"
          >
            ⇄
          </button>
        </div>

        {/* Right Side: Target */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', borderRadius: '6px', overflow: 'hidden' }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '6px 8px', borderBottom: '1px solid var(--border-color)', background: 'rgba(255,255,255,0.01)' }}>
            <select value={targetLang} onChange={(e) => setTargetLang(e.target.value)}>
              {LANGUAGES.map((l) => (
                <option key={l.code} value={l.code}>{l.name}</option>
              ))}
            </select>
            {isTranslating && (
              <span style={{ fontSize: '11px', color: 'var(--accent-color)', animation: 'pulse 1.5s infinite' }}>
                Translating...
              </span>
            )}
          </div>
          <div style={{ flex: 1 }}>
            <textarea
              placeholder="Translation will appear here..."
              value={translatedText}
              readOnly
            />
          </div>
          <div style={{ display: 'flex', gap: '4px', padding: '6px 8px', borderTop: '1px solid var(--border-color)', background: 'rgba(255,255,255,0.01)' }}>
            <button className="btn-primary" style={{ padding: '3px 8px', fontSize: '11px' }} onClick={handleCopy} disabled={!translatedText}>
              Copy
            </button>
            <button className="btn-secondary" style={{ padding: '3px 8px', fontSize: '11px' }} onClick={() => handleSpeak(translatedText)} disabled={!translatedText}>
              🔊 Speak
            </button>
          </div>
        </div>

      </div>

      {/* Bottom Control Bar */}
      <div style={{ display: 'flex', justifyContent: 'space-between', padding: '0 12px 12px 12px', boxSizing: 'border-box' }}>
        <button className="btn-secondary" style={{ display: 'flex', alignItems: 'center', gap: '4px' }} onClick={() => setShowHistory(!showHistory)}>
          📜 {showHistory ? 'Hide History' : 'History'}
        </button>
        <button className="btn-secondary" onClick={handleOpenSettings}>
          ⚙️ Settings
        </button>
      </div>

      {/* Sliding History Drawer */}
      {showHistory && (
        <div style={{
          position: 'absolute',
          top: '36px',
          right: 0,
          bottom: 0,
          width: '320px',
          background: 'var(--bg-secondary)',
          borderLeft: '1px solid var(--border-color)',
          display: 'flex',
          flexDirection: 'column',
          zIndex: 1000,
          boxShadow: 'var(--shadow-lg)'
        }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '8px 12px', borderBottom: '1px solid var(--border-color)' }}>
            <div style={{ fontWeight: 600, fontSize: '13px' }}>Translation History</div>
            <button style={{ background: 'transparent', border: 'none', color: 'var(--text-muted)', cursor: 'pointer', fontSize: '13px' }} onClick={() => setShowHistory(false)}>
              ✕
            </button>
          </div>
          
          {/* Search History */}
          <div style={{ padding: '8px' }}>
            <input
              type="text"
              placeholder="Search history..."
              value={historySearch}
              onChange={(e) => setHistorySearch(e.target.value)}
              style={{
                width: '100%',
                background: 'var(--bg-tertiary)',
                border: '1px solid var(--border-color)',
                borderRadius: '4px',
                padding: '6px 8px',
                color: 'var(--text-primary)',
                outline: 'none',
                boxSizing: 'border-box',
                fontSize: '12px'
              }}
            />
          </div>

          {/* History List */}
          <div style={{ flex: 1, overflowY: 'auto', padding: '8px', display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {history.length === 0 ? (
              <div style={{ color: 'var(--text-muted)', textAlign: 'center', fontSize: '12px', marginTop: '24px' }}>
                No history entries found.
              </div>
            ) : (
              history.map((entry) => (
                <div key={entry.id} style={{
                  background: 'var(--bg-tertiary)',
                  border: '1px solid var(--border-color)',
                  borderRadius: '4px',
                  padding: '8px',
                  fontSize: '12px',
                  position: 'relative'
                }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', color: 'var(--text-muted)', fontSize: '10px', marginBottom: '4px' }}>
                    <span>{entry.source_lang.toUpperCase()} ➔ {entry.target_lang.toUpperCase()}</span>
                    <span>{entry.created_at.split(' ')[1]}</span>
                  </div>
                  <div style={{ color: 'var(--text-primary)', fontWeight: 500, whiteSpace: 'pre-wrap', marginBottom: '4px' }}>{entry.original_text}</div>
                  <div style={{ color: 'var(--text-secondary)', whiteSpace: 'pre-wrap' }}>{entry.translated_text}</div>
                  
                  <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '8px', marginTop: '6px', borderTop: '1px solid rgba(255,255,255,0.03)', paddingTop: '4px' }}>
                    <button style={{ background: 'transparent', border: 'none', cursor: 'pointer', color: entry.is_favorite ? 'gold' : 'var(--text-muted)' }} onClick={() => handleToggleFavorite(entry.id)}>
                      ★
                    </button>
                    <button style={{ background: 'transparent', border: 'none', cursor: 'pointer', color: 'var(--danger-color)' }} onClick={() => handleDeleteHistory(entry.id)}>
                      🗑
                    </button>
                  </div>
                </div>
              ))
            )}
          </div>

          {/* Clear History Button */}
          {history.length > 0 && (
            <div style={{ padding: '8px', borderTop: '1px solid var(--border-color)', display: 'flex' }}>
              <button className="btn-secondary" style={{ width: '100%', padding: '4px', fontSize: '11px', color: 'var(--danger-color)' }} onClick={handleClearAllHistory}>
                Clear All History
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
