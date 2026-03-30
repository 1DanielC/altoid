import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useQueryClient } from '@tanstack/react-query';
import { USER_QUERY_KEY } from './hooks/queries/useUserQuery';
import Header from './components/Header';
import Content from './components/Content';
import Footer from './components/Footer';
import SettingsMenu from './components/SettingsMenu';
import './App.css';

function App() {
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [configLoaded, setConfigLoaded] = useState(false);
  const [configError, setConfigError] = useState<string | null>(null);
  const queryClient = useQueryClient();

  useEffect(() => {
    invoke('load_config')
      .then(async (config) => {
        if (!config) {
          // No bootstrap config available — app runs without auth
          console.log('No OAuth config available, running without auth');
          setConfigLoaded(true);
          return;
        }
        // Check if user is already authenticated
        try {
          const user = await invoke('check_user');
          if (user) {
            queryClient.setQueryData(USER_QUERY_KEY, user);
          }
        } catch (e) {
          console.log('User not authenticated:', e);
        }
        setConfigLoaded(true);
      })
      .catch((e) => {
        console.error('Failed to load config:', e);
        setConfigError(typeof e === 'string' ? e : JSON.stringify(e));
        setConfigLoaded(true);
      });
  }, [queryClient]);

  if (!configLoaded) {
    return (
      <div id="app">
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100vh', color: '#666' }}>
          Loading configuration...
        </div>
      </div>
    );
  }

  return (
    <div id="app">
      {configError && (
        <div style={{ background: '#fff3cd', color: '#856404', padding: '8px 12px', fontSize: 12, textAlign: 'center' }}>
          Config error: {configError}
        </div>
      )}
      <Header onOpenSettings={() => setSettingsOpen(true)} />
      <Content />
      <Footer />
      <SettingsMenu isOpen={settingsOpen} setIsOpen={setSettingsOpen} />
    </div>
  );
}

export default App;
