import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import SetupWizard from "./pages/SetupWizard";
import Dashboard from "./pages/Dashboard";
import Channels from "./pages/Channels";
import ModelMonitor from "./pages/ModelMonitor";
import AIAssistant from "./pages/AIAssistant";
import Logs from "./pages/Logs";
import Settings from "./pages/Settings";

type Page = "dashboard" | "channels" | "models" | "assistant" | "logs" | "settings";

interface SystemStatus {
  overall_phase: string;
  wsl: { installed: boolean; version: string | null; details: string };
  ubuntu: { installed: boolean; version: string | null; details: string };
  openclaw: { installed: boolean; version: string | null; details: string };
  config: { installed: boolean; version: string | null; details: string };
}

interface NavItem {
  id: Page;
  label: string;
  icon: string;
}

const navItems: NavItem[] = [
  { id: "dashboard", label: "الرئيسية", icon: "🏠" },
  { id: "channels", label: "القنوات", icon: "📡" },
  { id: "models", label: "الموديلات", icon: "🧠" },
  { id: "assistant", label: "المساعد الذكي", icon: "🫀" },
  { id: "logs", label: "السجلات", icon: "📋" },
  { id: "settings", label: "الإعدادات", icon: "⚙️" },
];

function App() {
  const [currentPage, setCurrentPage] = useState<Page>("dashboard");
  const [systemPhase, setSystemPhase] = useState<string>("checking");
  const [setupComplete, setSetupComplete] = useState(false);
  const [gatewayStatus, setGatewayStatus] = useState<
    "connected" | "disconnected" | "checking"
  >("checking");

  // فحص النظام عند البداية
  useEffect(() => {
    const check = async () => {
      try {
        const s = await invoke<SystemStatus>("check_full_system");
        setSystemPhase(s.overall_phase);
        if (s.overall_phase === "OpenClawRunning") {
          setSetupComplete(true);
          setGatewayStatus("connected");
        } else if (s.overall_phase === "OpenClawStopped") {
          setSetupComplete(true);
          setGatewayStatus("disconnected");
        } else {
          setSetupComplete(false);
          setGatewayStatus("disconnected");
        }
      } catch {
        setSystemPhase("error");
        setGatewayStatus("disconnected");
      }
    };
    check();
  }, []);

  const handleSetupComplete = () => {
    setSetupComplete(true);
    setGatewayStatus("connected");
  };

  const renderPage = () => {
    switch (currentPage) {
      case "dashboard":
        return <Dashboard />;
      case "channels":
        return <Channels />;
      case "models":
        return <ModelMonitor />;
      case "assistant":
        return <AIAssistant />;
      case "logs":
        return <Logs />;
      case "settings":
        return <Settings />;
    }
  };

  // شاشة التحميل الأولي
  if (systemPhase === "checking") {
    return (
      <div className="fullscreen-loading">
        <div className="spinner" />
        <h2>🧠 مساعد شخصي</h2>
        <p>جاري فحص النظام...</p>
      </div>
    );
  }

  return (
    <div className="app-container">
      {/* الشريط الجانبي — يظهر دائمًا */}
      <aside className="sidebar">
        <div className="sidebar-header">
          <div className="logo">🧠</div>
          <h1 className="app-title">مساعد شخصي</h1>
          <div className={`status-badge ${gatewayStatus}`}>
            <span className="status-dot" />
            {gatewayStatus === "connected"
              ? "متصل"
              : gatewayStatus === "disconnected"
              ? "غير متصل"
              : "..."}
          </div>
          {!setupComplete && (
            <div className="setup-notice">
              ⚠️ الإعداد غير مكتمل
            </div>
          )}
        </div>
        <nav className="nav-menu">
          {navItems.map((item) => (
            <button
              key={item.id}
              className={`nav-item ${currentPage === item.id ? "active" : ""}`}
              onClick={() => setCurrentPage(item.id)}
            >
              <span className="nav-icon">{item.icon}</span>
              <span className="nav-label">{item.label}</span>
            </button>
          ))}
        </nav>
        <div className="sidebar-footer">
          <span className="version">v0.2.0</span>
          {!setupComplete && (
            <div className="setup-indicator">
              <span className="setup-dot" />
              يحتاج إعداد
            </div>
          )}
        </div>
      </aside>

      {/* المحتوى الرئيسي */}
      <main className="main-content">
        {!setupComplete ? (
          <SetupWizard onComplete={handleSetupComplete} />
        ) : (
          renderPage()
        )}
      </main>
    </div>
  );
}

export default App;
