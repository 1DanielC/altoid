import Header from './components/Header';
import Content from './components/Content';
import Footer from './components/Footer';
import SettingsMenu from './components/SettingsMenu';
import './App.css';

function App() {
  return (
    <div id="app">
      <Header />
      <Content />
      <Footer />
      <SettingsMenu />
    </div>
  );
}

export default App;
