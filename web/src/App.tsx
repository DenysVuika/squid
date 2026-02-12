import { BrowserRouter, Routes, Route, Link, useLocation } from 'react-router-dom';
import ChatBot from './components/app/chatbot';
import Logs from './components/app/logs';
import { Button } from './components/ui/button';
import { MessageSquare, FileText } from 'lucide-react';

function Navigation() {
  const location = useLocation();

  return (
    <nav className="bg-white border-b border-gray-200 px-4 py-3 flex items-center gap-4">
      <div className="flex items-center gap-2 mr-8">
        <span className="text-2xl">🦑</span>
        <span className="font-bold text-xl">Squid</span>
      </div>
      <div className="flex gap-2">
        <Link to="/">
          <Button variant={location.pathname === '/' ? 'default' : 'ghost'} className="flex items-center gap-2">
            <MessageSquare className="h-4 w-4" />
            Chat
          </Button>
        </Link>
        <Link to="/logs">
          <Button variant={location.pathname === '/logs' ? 'default' : 'ghost'} className="flex items-center gap-2">
            <FileText className="h-4 w-4" />
            Logs
          </Button>
        </Link>
      </div>
    </nav>
  );
}

function AppContent() {
  return (
    <div className="flex flex-col h-screen w-screen">
      <Navigation />
      <div className="flex-1 overflow-hidden">
        <Routes>
          <Route path="/" element={<ChatBot />} />
          <Route path="/logs" element={<Logs />} />
        </Routes>
      </div>
    </div>
  );
}

function App() {
  return (
    <BrowserRouter>
      <AppContent />
    </BrowserRouter>
  );
}

export default App;
