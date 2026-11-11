import { useEffect, useState } from 'react';
import { BrowserRouter, Navigate, Route, Routes, useLocation } from 'react-router-dom';
import { ThemeProvider } from './theme/theme';
import { GradientBackground } from './components/GradientBackground';
import { SiteHeader } from './components/SiteHeader';
import { About } from './pages/About';
import { Home } from './pages/Home';
import { Docs } from './pages/Docs';

const WIDE_VIEWPORT = '(min-width: 960px)';

function ScrollToTop() {
  const { pathname } = useLocation();
  useEffect(() => {
    window.scrollTo(0, 0);
  }, [pathname]);
  return null;
}

function Shell() {
  const { pathname } = useLocation();
  const onDocs = pathname === '/docs';

  // Docs sidebar state: visible by default on wide screens, hidden (drawer)
  // by default on mobile. The header's menu button toggles it. `Docs` mounts
  // fresh on every route change, so this initial value also resets per page.
  const [sidebarOpen, setSidebarOpen] = useState(() =>
    window.matchMedia(WIDE_VIEWPORT).matches,
  );

  useEffect(() => {
    const mq = window.matchMedia(WIDE_VIEWPORT);
    const onChange = () => setSidebarOpen(mq.matches);
    mq.addEventListener('change', onChange);
    return () => mq.removeEventListener('change', onChange);
  }, []);

  return (
    <>
      <GradientBackground />
      <SiteHeader
        onToggleDocs={onDocs ? () => setSidebarOpen((open) => !open) : undefined}
      />
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/about" element={<About />} />
        <Route
          path="/docs"
          element={
            <Docs
              open={sidebarOpen}
              closeSidebar={() => setSidebarOpen(false)}
            />
          }
        />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </>
  );
}

export default function App() {
  return (
    <ThemeProvider>
      <BrowserRouter basename={import.meta.env.BASE_URL}>
        <ScrollToTop />
        <Shell />
      </BrowserRouter>
    </ThemeProvider>
  );
}
