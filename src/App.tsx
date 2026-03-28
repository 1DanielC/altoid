import { useState } from 'react';
import Header from './components/Header';
import Content from './components/Content';
import Footer from './components/Footer';
import SettingsMenu from './components/SettingsMenu';
import './App.css';

function App() {
  const [settingsOpen, setSettingsOpen] = useState(false);

  return (
    <div id="app">
      <Header onOpenSettings={() => setSettingsOpen(true)} />
      <Content />
      <Footer />
      <SettingsMenu isOpen={settingsOpen} setIsOpen={setSettingsOpen} />
    </div>
  );
}

export default App;
