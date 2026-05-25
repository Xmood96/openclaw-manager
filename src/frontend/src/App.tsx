import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AnimatePresence, motion } from "framer-motion";
import {
  LayoutDashboard,
  Radio,
  Brain,
  Bot,
  ScrollText,
  Settings,
  Wifi,
  WifiOff,
  Loader2,
  AlertTriangle,
} from "lucide-react";
import SetupWizard from "./pages/SetupWizard";
import Dashboard from "./pages/Dashboard";
import Channels from "./pages/Channels";
import ModelMonitor from "./pages/ModelMonitor";
import AIAssistant from "./pages/AIAssistant";
import Logs from "./pages/Logs";
import SettingsPage from "./pages/Settings";

type Page = "dashboard" | "channels" | "models" | "assistant" | "logs" | "settings";

interface SystemStatus {
  overall_phase: string;
  wsl: { installed: boolean; version: string | null; details: string };
  ubuntu: { installed: boolean; version: string | null; details: string };
  openclaw: { installed: boolean; version: string | null; details: string };
  config: { installed: boolean; version: string | null; details: string };
}

const navItems: { id: Page; label: string; icon: React.ComponentType<any> }[] = [
  { id: "dashboard", label: "الرئيسية", icon: LayoutDashboard },
  { id: "channels", label: "القنوات", icon: Radio },
  { id: "models", label: "الموديلات", icon: Brain },
  { id: "assistant", label: "مساعد الصيانة", icon: Bot },
  { id: "logs", label: "السجلات", icon: ScrollText },
  { id: "settings", label: "الإعدادات", icon: Settings },
];

const pageVariants = {
  initial: { opacity: 0, x: -12 },
  animate: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: 12 },
};

const pageTransition = { duration: 0.2, ease: "easeOut" };

export default function App() {
  const [currentPage, setCurrentPage] = useState<Page>("dashboard");
  const [systemPhase, setSystemPhase] = useState<string>("checking");
  const [setupComplete, setSetupComplete] = useState(false);
  const [gatewayStatus, setGatewayStatus] = useState<"connected" | "disconnected" | "checking">("checking");

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

  const handleSetupComplete = useCallback(() => {
    setSetupComplete(true);
    setGatewayStatus("connected");
  }, []);

  const renderPage = () => {
    switch (currentPage) {
      case "dashboard":
        return <Dashboard onOpenAssistant={() => setCurrentPage("assistant")} />;
      case "channels":
        return <Channels />;
      case "models":
        return <ModelMonitor />;
      case "assistant":
        return <AIAssistant />;
      case "logs":
        return <Logs />;
      case "settings":
        return <SettingsPage />;
    }
  };

  if (systemPhase === "checking") {
    return (
      <div className="h-screen flex flex-col items-center justify-center gap-4 bg-bg">
        <Loader2 size={40} className="text-primary animate-spin" />
        <h2 className="text-2xl font-bold text-primary">🧠 مساعد شخصي</h2>
        <p className="text-muted">جاري فحص النظام...</p>
      </div>
    );
  }

  return (
    <div className="flex h-screen w-screen overflow-hidden">
      {/* Sidebar */}
      <aside className="w-[220px] flex flex-col gap-2 p-4 bg-sidebar text-sidebar-text flex-shrink-0">
        {/* Header */}
        <div className="text-center pb-4 border-b border-white/10 mb-2">
          <motion.div
            className="text-4xl mb-2"
            animate={{ scale: [1, 1.08, 1] }}
            transition={{ duration: 3, repeat: Infinity, ease: "easeInOut" }}
          >
            🧠
          </motion.div>
          <h1 className="text-lg font-bold mb-2">مساعد شخصي</h1>
          <motion.div
            className={`inline-flex items-center gap-1.5 text-xs px-3 py-1 rounded-full ${
              gatewayStatus === "connected"
                ? "bg-success/20 text-green-400"
                : gatewayStatus === "disconnected"
                ? "bg-error/20 text-red-400"
                : "bg-warning/20 text-yellow-400"
            }`}
            animate={{ opacity: gatewayStatus === "checking" ? [0.5, 1, 0.5] : 1 }}
            transition={{ duration: 1.5, repeat: Infinity }}
          >
            {gatewayStatus === "connected" ? (
              <Wifi size={12} />
            ) : gatewayStatus === "disconnected" ? (
              <WifiOff size={12} />
            ) : (
              <Loader2 size={12} className="animate-spin" />
            )}
            {gatewayStatus === "connected" ? "متصل" : gatewayStatus === "disconnected" ? "غير متصل" : "..."}
          </motion.div>
          {!setupComplete && (
            <div className="mt-2 text-[11px] text-warning px-2 py-1 bg-warning/15 rounded-lg">
              ⚠️ الإعداد غير مكتمل
            </div>
          )}
        </div>

        {/* Navigation */}
        <nav className="flex flex-col gap-1 flex-1">
          {navItems.map((item) => {
            const Icon = item.icon;
            return (
              <motion.button
                key={item.id}
                whileHover={{ x: -2 }}
                whileTap={{ scale: 0.97 }}
                onClick={() => setCurrentPage(item.id)}
                className={`flex items-center gap-2.5 px-3 py-2.5 rounded-xl text-sm text-right transition-colors ${
                  currentPage === item.id
                    ? "bg-sidebar-active text-white"
                    : "hover:bg-white/[0.06] text-sidebar-text"
                }`}
              >
                <Icon size={18} />
                <span>{item.label}</span>
              </motion.button>
            );
          })}
        </nav>

        {/* Footer */}
        <div className="pt-2 border-t border-white/10 text-[11px] text-muted text-center">
          <span>v0.4.0</span>
          {!setupComplete && (
            <div className="flex items-center justify-center gap-1 mt-1 text-warning text-[10px]">
              <AlertTriangle size={10} />
              يحتاج إعداد
            </div>
          )}
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-y-auto p-6">
        <AnimatePresence mode="wait">
          <motion.div
            key={setupComplete ? currentPage : "setup"}
            variants={pageVariants}
            initial="initial"
            animate="animate"
            exit="exit"
            transition={pageTransition}
            className="max-w-4xl mx-auto"
          >
            {!setupComplete ? (
              <SetupWizard onComplete={handleSetupComplete} />
            ) : (
              renderPage()
            )}
          </motion.div>
        </AnimatePresence>
      </main>
    </div>
  );
}
