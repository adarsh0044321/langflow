import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { TitleBar } from '../components/TitleBar';

interface AppConfig {
  source_lang: string;
  target_lang: string;
  translation_mode: string;
  online_provider: string;
  api_key: string;
  hotkey_translate: string;
  hotkey_ocr: string;
  hotkey_typing: string;
  inline_typing_enabled: boolean;
  replace_selection_directly: boolean;
  run_on_startup: boolean;
  idle_unload_timeout_secs: number;
}

interface LanguagePackInfo {
  lang_code: string;
  lang_name: string;
  version: string;
  status: string; // 'INSTALLED', 'DOWNLOADING', 'NOT_INSTALLED'
  local_path: string | null;
  model_size_bytes: number;
}

const AVAILABLE_PACKS = [
  { code: 'ja', name: 'Japanese (MarianMT Model - FP16)' },
  { code: 'zh', name: 'Chinese (MarianMT Model - FP16)' },
  { code: 'ko', name: 'Korean (MarianMT Model - FP16)' },
  { code: 'de', name: 'German (MarianMT Model - FP16)' },
  { code: 'fr', name: 'French (MarianMT Model - FP16)' },
  { code: 'es', name: 'Spanish (MarianMT Model - FP16)' },
  { code: 'ru', name: 'Russian (MarianMT Model - FP16)' }
];

export const SettingsWindow: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'general' | 'languages' | 'hotkeys' | 'typing'>('general');
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [packs, setPacks] = useState<LanguagePackInfo[]>([]);
  const [downloadingStatus, setDownloadingStatus] = useState<Record<string, number>>({});
  


  // 1. Load config and language packs
  const loadData = async () => {
    try {
      const cfg: AppConfig = await invoke('get_config');
      setConfig(cfg);

      const installedList: LanguagePackInfo[] = await invoke('get_installed_packs');
      setPacks(installedList);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    loadData();

    // 2. Listen to background language pack download progress
    const unlistenProgress = listen<{ lang_code: string; progress: number; status: string }>(
      'download-progress',
      (event) => {
        const { lang_code, progress, status } = event.payload;
        setDownloadingStatus((prev) => ({
          ...prev,
          [lang_code]: progress,
        }));
        
        if (progress >= 100 || status === 'INSTALLED') {
          loadData(); // Reload pack registry values
        }
      }
    );

    // 3. Listen to global IME status updates
    const unlistenIme = listen<boolean>('ime-status-changed', (event) => {
      const activeState = event.payload;
      setConfig((prev) => prev ? { ...prev, inline_typing_enabled: activeState } : null);
    });

    return () => {
      unlistenProgress.then((f) => f());
      unlistenIme.then((f) => f());
    };
  }, []);

  const handleSaveConfig = async (updatedConfig: AppConfig) => {
    try {
      await invoke('update_config', { config: updatedConfig });
      setConfig(updatedConfig);
    } catch (e) {
      console.error(e);
    }
  };

  const handleInstallPack = async (code: string, name: string) => {
    try {
      setDownloadingStatus((prev) => ({ ...prev, [code]: 0 }));
      // Run background download task (will send progress events)
      invoke('download_pack', { code, name });
    } catch (e) {
      console.error(e);
    }
  };

  const handleUninstallPack = async (code: string) => {
    if (confirm(`Uninstall language pack for ${code.toUpperCase()}?`)) {
      try {
        await invoke('uninstall_pack', { code });
        setDownloadingStatus((prev) => {
          const cpy = { ...prev };
          delete cpy[code];
          return cpy;
        });
        loadData();
      } catch (e) {
        console.error(e);
      }
    }
  };

  const handleToggleIme = async () => {
    try {
      const activeState: boolean = await invoke('toggle_ime_mode');
      if (config) {
        setConfig({ ...config, inline_typing_enabled: activeState });
      }
    } catch (e) {
      console.error(e);
    }
  };



  if (!config) {
    return <div style={{ color: 'var(--text-muted)', display: 'flex', height: '100vh', alignItems: 'center', justifyContent: 'center' }}>Loading Settings...</div>;
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', background: 'var(--bg-primary)' }}>
      <TitleBar title="LangFlow Settings & Language Manager" />

      {/* Main Container */}
      <div style={{ display: 'flex', flex: 1, padding: '48px 12px 12px 12px', gap: '12px', boxSizing: 'border-box', overflow: 'hidden' }}>
        
        {/* Sidebar Nav */}
        <div style={{ width: '160px', display: 'flex', flexDirection: 'column', gap: '4px' }}>
          <button
            className={`btn-secondary ${activeTab === 'general' ? 'btn-primary' : ''}`}
            style={{ textAlign: 'left', background: activeTab === 'general' ? 'var(--accent-color)' : 'transparent', border: 'none' }}
            onClick={() => setActiveTab('general')}
          >
            ⚙️ General
          </button>
          <button
            className={`btn-secondary ${activeTab === 'languages' ? 'btn-primary' : ''}`}
            style={{ textAlign: 'left', background: activeTab === 'languages' ? 'var(--accent-color)' : 'transparent', border: 'none' }}
            onClick={() => setActiveTab('languages')}
          >
            🌐 Language Packs
          </button>
          <button
            className={`btn-secondary ${activeTab === 'hotkeys' ? 'btn-primary' : ''}`}
            style={{ textAlign: 'left', background: activeTab === 'hotkeys' ? 'var(--accent-color)' : 'transparent', border: 'none' }}
            onClick={() => setActiveTab('hotkeys')}
          >
            ⌨️ Hotkeys
          </button>
          <button
            className={`btn-secondary ${activeTab === 'typing' ? 'btn-primary' : ''}`}
            style={{ textAlign: 'left', background: activeTab === 'typing' ? 'var(--accent-color)' : 'transparent', border: 'none' }}
            onClick={() => setActiveTab('typing')}
          >
            ✍️ Typing Assistant
          </button>
        </div>

        {/* Tab Content Panel */}
        <div style={{ flex: 1, background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', borderRadius: '6px', padding: '16px', overflowY: 'auto', boxSizing: 'border-box' }}>
          
          {/* General Tab */}
          {activeTab === 'general' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
              <h3 style={{ margin: '0 0 8px 0', fontFamily: 'var(--font-display)', fontWeight: 600 }}>General Preferences</h3>
              
              <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                <label style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>Default Translation Mode</label>
                <select
                  value={config.translation_mode}
                  onChange={(e) => handleSaveConfig({ ...config, translation_mode: e.target.value })}
                >
                  <option value="Hybrid">Hybrid Mode (Uses Local Model, falls back to API)</option>
                  <option value="Offline">Offline Mode (Strictly Local Models)</option>
                  <option value="Online">Online Mode (Strictly Cloud API)</option>
                </select>
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                <label style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>Online Provider</label>
                <select
                  value={config.online_provider}
                  onChange={(e) => handleSaveConfig({ ...config, online_provider: e.target.value })}
                >
                  <option value="Google">Google Translate (Free, No Key Required)</option>
                  <option value="DeepL">DeepL API</option>
                  <option value="Gemini">Gemini API</option>
                </select>
              </div>

              {config.online_provider !== 'Google' && (
                <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                  <label style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>API Secret Key</label>
                  <input
                    type="password"
                    placeholder="Enter your API Key..."
                    value={config.api_key}
                    onChange={(e) => handleSaveConfig({ ...config, api_key: e.target.value })}
                    style={{
                      background: 'var(--bg-tertiary)',
                      border: '1px solid var(--border-color)',
                      borderRadius: '4px',
                      padding: '6px 8px',
                      color: 'var(--text-primary)',
                      outline: 'none',
                      fontSize: '12px'
                    }}
                  />
                </div>
              )}

              <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '12px' }}>
                  <label style={{ color: 'var(--text-secondary)' }}>Local Model Idle Unload Timer</label>
                  <span>{config.idle_unload_timeout_secs} seconds</span>
                </div>
                <input
                  type="range"
                  min="60"
                  max="600"
                  step="30"
                  value={config.idle_unload_timeout_secs}
                  onChange={(e) => handleSaveConfig({ ...config, idle_unload_timeout_secs: parseInt(e.target.value) })}
                  style={{ cursor: 'pointer', accentColor: 'var(--accent-color)' }}
                />
                <span style={{ fontSize: '10px', color: 'var(--text-muted)' }}>
                  Unloads local ONNX models from RAM after inactivity to save resource footprint.
                </span>
              </div>
            </div>
          )}

          {/* Languages Tab */}
          {activeTab === 'languages' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
              <h3 style={{ margin: '0 0 4px 0', fontFamily: 'var(--font-display)', fontWeight: 600 }}>Language Packs Manager</h3>
              <p style={{ fontSize: '11px', color: 'var(--text-muted)', margin: '0 0 8px 0' }}>
                Modular packs store local translation models in your user profile. This keeps the initial installation size tiny.
              </p>

              <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                {AVAILABLE_PACKS.map((pack) => {
                  const dbRecord = packs.find((p) => p.lang_code === pack.code);
                  const isInstalled = dbRecord?.status === 'INSTALLED';
                  const progress = downloadingStatus[pack.code];
                  const isDownloading = progress !== undefined && progress < 100;

                  return (
                    <div
                      key={pack.code}
                      style={{
                        display: 'flex',
                        flexDirection: 'column',
                        background: 'var(--bg-tertiary)',
                        border: '1px solid var(--border-color)',
                        borderRadius: '4px',
                        padding: '10px 12px',
                        gap: '6px'
                      }}
                    >
                      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                        <div>
                          <div style={{ fontSize: '13px', fontWeight: 500 }}>{pack.name}</div>
                          <div style={{ fontSize: '10px', color: 'var(--text-muted)' }}>
                            {isInstalled ? `Installed (${(dbRecord.model_size_bytes / (1024 * 1024)).toFixed(1)} MB)` : 'Not installed (Uses Online Mode fallback)'}
                          </div>
                        </div>

                        <div>
                          {isInstalled ? (
                            <button
                              className="btn-secondary"
                              style={{ padding: '3px 8px', fontSize: '11px', color: 'var(--danger-color)', borderColor: 'var(--danger-color)' }}
                              onClick={() => handleUninstallPack(pack.code)}
                            >
                              Uninstall
                            </button>
                          ) : isDownloading ? (
                            <button className="btn-secondary" style={{ padding: '3px 8px', fontSize: '11px' }} disabled>
                              Downloading...
                            </button>
                          ) : (
                            <button
                              className="btn-primary"
                              style={{ padding: '3px 8px', fontSize: '11px' }}
                              onClick={() => handleInstallPack(pack.code, pack.name)}
                            >
                              Install Pack
                            </button>
                          )}
                        </div>
                      </div>

                      {/* Download Progress Bar */}
                      {isDownloading && (
                        <div style={{ width: '100%', background: 'rgba(255,255,255,0.05)', borderRadius: '3px', height: '6px', overflow: 'hidden' }}>
                          <div style={{ width: `${progress}%`, background: 'var(--accent-color)', height: '100%', transition: 'width 0.1s ease' }} />
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Hotkeys Tab */}
          {activeTab === 'hotkeys' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
              <h3 style={{ margin: '0 0 8px 0', fontFamily: 'var(--font-display)', fontWeight: 600 }}>System-Wide Hotkeys</h3>
              <p style={{ fontSize: '12px', color: 'var(--text-secondary)', margin: 0 }}>
                LangFlow runs a native Win32 keyboard monitor that operates in the background, working while you browse or play fullscreen games.
              </p>

              {/* Direct Selection Replacement Toggle Card */}
              <div style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '12px 14px',
                background: 'var(--bg-tertiary)',
                border: '1px solid var(--border-color)',
                borderRadius: '6px',
                marginTop: '6px'
              }}>
                <div>
                  <div style={{ fontSize: '13px', fontWeight: 600 }}>Direct Selection Replacement</div>
                  <div style={{ fontSize: '10px', color: 'var(--text-muted)', marginTop: '2px' }}>
                    Directly type/replace the translation over the selected text instead of showing the floating popup.
                  </div>
                </div>
                <label className="switch" style={{ position: 'relative', display: 'inline-block', width: '40px', height: '22px' }}>
                  <input
                    type="checkbox"
                    checked={config.replace_selection_directly}
                    onChange={() => handleSaveConfig({ ...config, replace_selection_directly: !config.replace_selection_directly })}
                    style={{ opacity: 0, width: 0, height: 0 }}
                  />
                  <span className={`slider ${config.replace_selection_directly ? 'active' : ''}`} style={{
                    position: 'absolute',
                    cursor: 'pointer',
                    top: 0, left: 0, right: 0, bottom: 0,
                    backgroundColor: config.replace_selection_directly ? 'var(--success-color)' : '#4b5563',
                    transition: '0.3s',
                    borderRadius: '34px'
                  }}>
                    <span style={{
                      position: 'absolute',
                      content: '""',
                      height: '16px',
                      width: '16px',
                      left: config.replace_selection_directly ? '20px' : '3px',
                      bottom: '3px',
                      backgroundColor: 'white',
                      transition: '0.3s',
                      borderRadius: '50%'
                    }} />
                  </span>
                </label>
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginTop: '8px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', padding: '10px', background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', borderRadius: '4px' }}>
                  <div>
                    <div style={{ fontSize: '13px', fontWeight: 500 }}>Translate Highlighted Text</div>
                    <div style={{ fontSize: '10px', color: 'var(--text-muted)' }}>
                      {config.replace_selection_directly 
                        ? 'Copies highlighted text, translates it, and replaces the selection in-place.' 
                        : 'Copies highlighted text, pops up quick translation tooltip.'}
                    </div>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center' }}>
                    <code style={{ background: 'black', color: 'var(--accent-hover)', padding: '4px 8px', borderRadius: '4px', fontSize: '12px' }}>
                      Ctrl + Shift + T
                    </code>
                  </div>
                </div>

                <div style={{ display: 'flex', justifyContent: 'space-between', padding: '10px', background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', borderRadius: '4px' }}>
                  <div>
                    <div style={{ fontSize: '13px', fontWeight: 500 }}>Screenshot OCR Translate</div>
                    <div style={{ fontSize: '10px', color: 'var(--text-muted)' }}>Dims screen, lets you select area, outputs floating translation.</div>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center' }}>
                    <code style={{ background: 'black', color: 'var(--accent-hover)', padding: '4px 8px', borderRadius: '4px', fontSize: '12px' }}>
                      Ctrl + Shift + S
                    </code>
                  </div>
                </div>

                <div style={{ display: 'flex', justifyContent: 'space-between', padding: '10px', background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', borderRadius: '4px' }}>
                  <div>
                    <div style={{ fontSize: '13px', fontWeight: 500 }}>Inline Typing Assistant Mode</div>
                    <div style={{ fontSize: '10px', color: 'var(--text-muted)' }}>Brings up floating typing assistant near your cursor.</div>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center' }}>
                    <code style={{ background: 'black', color: 'var(--accent-hover)', padding: '4px 8px', borderRadius: '4px', fontSize: '12px' }}>
                      Ctrl + Shift + I
                    </code>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Typing Assistant Tab */}
          {activeTab === 'typing' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
              <h3 style={{ margin: '0 0 4px 0', fontFamily: 'var(--font-display)', fontWeight: 600 }}>Real-Time Typing Translation (IME Mode)</h3>
              <p style={{ fontSize: '11px', color: 'var(--text-muted)', margin: 0 }}>
                Like Google Input Tools, this mode lets you type in your native language directly inside any game, chat box, or app, and automatically translates it on-the-fly.
              </p>

              {/* Status Toggle Card */}
              <div style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '12px 14px',
                background: 'var(--bg-tertiary)',
                border: '1px solid var(--border-color)',
                borderRadius: '6px',
                marginTop: '6px'
              }}>
                <div>
                  <div style={{ fontSize: '13px', fontWeight: 600 }}>Enable Real-Time Typing Translation</div>
                  <div style={{ fontSize: '10px', color: 'var(--text-muted)', marginTop: '2px' }}>
                    Globally translates text as you type. Toggle with <code style={{ color: 'var(--accent-hover)' }}>Ctrl+Shift+I</code>.
                  </div>
                </div>
                <label className="switch" style={{ position: 'relative', display: 'inline-block', width: '40px', height: '22px' }}>
                  <input
                    type="checkbox"
                    checked={config.inline_typing_enabled}
                    onChange={handleToggleIme}
                    style={{ opacity: 0, width: 0, height: 0 }}
                  />
                  <span className={`slider ${config.inline_typing_enabled ? 'active' : ''}`} style={{
                    position: 'absolute',
                    cursor: 'pointer',
                    top: 0, left: 0, right: 0, bottom: 0,
                    backgroundColor: config.inline_typing_enabled ? 'var(--success-color)' : '#4b5563',
                    transition: '0.3s',
                    borderRadius: '34px'
                  }}>
                    <span style={{
                      position: 'absolute',
                      content: '""',
                      height: '16px',
                      width: '16px',
                      left: config.inline_typing_enabled ? '20px' : '3px',
                      bottom: '3px',
                      backgroundColor: 'white',
                      transition: '0.3s',
                      borderRadius: '50%'
                    }} />
                  </span>
                </label>
              </div>

              {/* Test Sandbox */}
              <div style={{ marginTop: '8px' }}>
                <label style={{ fontSize: '12px', color: 'var(--text-secondary)', display: 'block', marginBottom: '6px' }}>
                  ✍️ Test Sandbox (Type here to test real-time translation):
                </label>
                <input
                  type="text"
                  placeholder={config.inline_typing_enabled 
                    ? "Type normally, e.g. 'hello world.' and see it translate on Space or Period..." 
                    : "Enable the toggle above to start testing..."
                  }
                  disabled={!config.inline_typing_enabled}
                  style={{
                    width: '100%',
                    background: 'var(--bg-tertiary)',
                    border: '1px solid var(--border-color)',
                    borderRadius: '4px',
                    padding: '8px 10px',
                    color: 'var(--text-primary)',
                    outline: 'none',
                    fontSize: '13px',
                    boxSizing: 'border-box'
                  }}
                />
              </div>

              {/* How it works card */}
              <div style={{
                background: 'rgba(255, 255, 255, 0.02)',
                border: '1px solid var(--border-color)',
                borderRadius: '6px',
                padding: '12px',
                fontSize: '11px',
                lineHeight: '1.5',
                color: 'var(--text-secondary)',
                display: 'flex',
                flexDirection: 'column',
                gap: '8px'
              }}>
                <div style={{ fontWeight: 600, color: 'var(--text-primary)', fontSize: '12px' }}>How does it work?</div>
                <div>
                  <strong>1. Word Translation (on Space):</strong> As you type a word and press <kbd style={{ background: '#374151', padding: '2px 4px', borderRadius: '3px' }}>Space</kbd>, LangFlow immediately translates that word and types the translation.
                </div>
                <div>
                  <strong>2. Full Sentence Context (on Punctuation/Enter):</strong> When you finish your sentence by pressing <kbd style={{ background: '#374151', padding: '2px 4px', borderRadius: '3px' }}>.</kbd> <kbd style={{ background: '#374151', padding: '2px 4px', borderRadius: '3px' }}>?</kbd> <kbd style={{ background: '#374151', padding: '2px 4px', borderRadius: '3px' }}>!</kbd> or <kbd style={{ background: '#374151', padding: '2px 4px', borderRadius: '3px' }}>Enter</kbd>, LangFlow automatically takes the entire sentence, re-translates it to ensure proper context-aware grammar, erases the draft, and types the final result!
                </div>
              </div>
            </div>
          )}

        </div>

      </div>
    </div>
  );
};
