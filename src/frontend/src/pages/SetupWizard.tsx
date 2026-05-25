import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion, AnimatePresence } from "framer-motion";
import {
  Monitor,
  Package,
  Settings,
  Play,
  CheckCircle2,
  ChevronDown,
  Loader2,
  RefreshCw,
  Trash2,
  Rocket,
  Server,
  Terminal,
  Wrench,
  Cpu,
} from "lucide-react";

interface CompStatus {
  installed: boolean;
  version: string | null;
  details: string;
}
interface SystemStatus {
  overall_phase: string;
  wsl: CompStatus;
  ubuntu: CompStatus;
  nodejs: CompStatus;
  openclaw: CompStatus;
  config: CompStatus;
}
interface SetupStep {
  step_id: number;
  title: string;
  description: string;
  explanation: string;
  recommendation: string;
  status: string;
  action_label: string;
}
interface InstallGuide {
  steps: SetupStep[];
  current_step: number;
  total_steps: number;
  overall_progress: number;
}
interface ModelRec {
  id: string;
  name: string;
  provider: string;
  cost_tier: string;
  speed: string;
  quality: string;
  best_for: string[];
  requires_api_key: boolean;
  recommendation_level: string;
  explanation_ar: string;
}

const phaseMeta: Record<string, { icon: React.ComponentType<{ size?: number }>; emoji: string; title: string; subtitle: string }> = {
  NoWSL: { icon: Monitor, emoji: "🪟", title: "مرحبًا بك في مساعدك الشخصي", subtitle: "لنبدأ بتجهيز البيئة" },
  WSLNoDistro: { icon: Server, emoji: "🐧", title: "جهز Linux", subtitle: "نحتاج توزيعة Ubuntu" },
  DistroNoOpenClaw: { icon: Package, emoji: "🧠", title: "نصب OpenClaw", subtitle: "القلب النابض للمساعد" },
  OpenClawNoConfig: { icon: Settings, emoji: "⚙️", title: "الإعدادات الأولية", subtitle: "لنضبط المساعد" },
  OpenClawStopped: { icon: Play, emoji: "▶️", title: "شغّل المساعد", subtitle: "كل شيء جاهز للتشغيل" },
};

const levelColor: Record<string, string> = {
  recommended: "border-primary bg-primary/5",
  good: "border-secondary bg-secondary/5",
  budget: "border-warning bg-warning/5",
  premium: "border-purple-500 bg-purple-500/5",
};
const levelLabel: Record<string, string> = {
  recommended: "⭐ موصى به",
  good: "👍 جيد",
  budget: "💰 اقتصادي",
  premium: "💎 احترافي",
};

export default function SetupWizard({ onComplete }: { onComplete: () => void }) {
  const [status, setStatus] = useState<SystemStatus | null>(null);
  const [guide, setGuide] = useState<InstallGuide | null>(null);
  const [models, setModels] = useState<ModelRec[]>([]);
  const [loading, setLoading] = useState(true);
  const [actionOutput, setActionOutput] = useState("");
  const [runningAction, setRunningAction] = useState(false);
  const [expandedStep, setExpandedStep] = useState<number | null>(null);

  useEffect(() => { checkSystem(); }, []);

  const checkSystem = async () => {
    setLoading(true);
    try {
      const s = await invoke<SystemStatus>("check_full_system");
      setStatus(s);
      const g = await invoke<InstallGuide>("get_setup_guide", { phase: s.overall_phase });
      setGuide(g);
      const m = await invoke<ModelRec[]>("get_model_recommendations");
      setModels(m);
    } catch (e) {
      console.error("فحص فاشل:", e);
    } finally { setLoading(false); }
  };

  const runAction = async (label: string, command: string, useWindows = false) => {
    setRunningAction(true);
    setActionOutput(`🔄 ${label}...`);
    try {
      const fn = useWindows ? "run_windows_command" : "run_install_command";
      const result = await invoke<string>(fn, { command });
      setActionOutput(result);
    } catch (e) {
      setActionOutput(`❌ ${e}`);
    }
    setRunningAction(false);
    setTimeout(checkSystem, 2000);
  };

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center h-60 gap-4">
        <Loader2 size={32} className="animate-spin text-primary" />
        <p className="text-muted">جاري فحص النظام...</p>
      </div>
    );
  }

  const phase = status?.overall_phase || "";
  const meta = phaseMeta[phase] || { icon: Terminal, emoji: "❓", title: "جاري الفحص...", subtitle: "" };
  const isComplete = phase === "OpenClawRunning";

  // Success screen
  if (isComplete) {
    return (
      <motion.div initial={{ scale: 0.95, opacity: 0 }} animate={{ scale: 1, opacity: 1 }} className="text-center py-16 max-w-md mx-auto">
        <motion.div animate={{ scale: [1, 1.1, 1] }} transition={{ duration: 0.5 }} className="text-7xl mb-4">🎉</motion.div>
        <h2 className="text-3xl font-bold mb-2">مساعدك الشخصي جاهز!</h2>
        <p className="text-muted mb-6">OpenClaw يعمل وكل القنوات جاهزة</p>
        <div className="flex flex-col gap-2 mb-8">
          {[
            `WSL ${status?.wsl.version ? `(${status.wsl.version})` : ""}`,
            "Ubuntu مثبت",
            `OpenClaw ${status?.openclaw.version ? `(${status.openclaw.version})` : ""}`,
            "الإعدادات موجودة",
          ].map((text, i) => (
            <motion.div
              key={i}
              initial={{ opacity: 0, x: -8 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: 0.2 + i * 0.1 }}
              className="flex items-center gap-2 p-2.5 rounded-xl bg-success/5 border border-success/20 text-success text-sm font-semibold"
            >
              <CheckCircle2 size={16} /> {text}
            </motion.div>
          ))}
        </div>
        <button
          onClick={onComplete}
          className="flex items-center gap-2 mx-auto px-8 py-3.5 rounded-2xl bg-primary text-white font-bold text-lg hover:bg-primary-dark transition-all hover:scale-105"
        >
          <Rocket size={20} /> فتح لوحة التحكم
        </button>
      </motion.div>
    );
  }

  return (
    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="max-w-2xl mx-auto">
      {/* Header */}
      <div className="text-center py-8">
        <div className="text-6xl mb-3">{meta.emoji}</div>
        <h2 className="text-3xl font-bold mb-1">{meta.title}</h2>
        <p className="text-muted text-sm">{meta.subtitle}</p>
      </div>

      {/* Progress */}
      {guide && (
        <div className="flex items-center gap-3 mb-6 px-2">
          <div className="flex-1 h-2 bg-border rounded-full overflow-hidden">
            <motion.div
              className="h-full bg-gradient-to-r from-primary to-secondary rounded-full"
              initial={{ width: 0 }}
              animate={{ width: `${guide.overall_progress * 100}%` }}
              transition={{ duration: 0.5, ease: "easeOut" }}
            />
          </div>
          <span className="text-xs text-muted whitespace-nowrap font-mono">
            {Math.round(guide.overall_progress * 100)}% · {guide.current_step}/{guide.total_steps}
          </span>
        </div>
      )}

      {/* Steps */}
      <div className="flex flex-col gap-2 mb-4">
        <AnimatePresence>
          {guide?.steps.map((step, i) => {
            const isActive = step.status === "Current";
            const isDone = step.status === "Done";
            const hasError = step.status === "Error";
            const isExpanded = expandedStep === step.step_id;

            return (
              <motion.div
                key={step.step_id}
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: i * 0.04 }}
                className={`rounded-2xl border-2 transition-all cursor-pointer ${
                  isActive
                    ? "border-primary bg-primary/[0.03] shadow-[0_0_0_3px_rgba(7,89,133,0.06)]"
                    : isDone
                    ? "border-success/30 opacity-70"
                    : hasError
                    ? "border-error/30"
                    : "border-border bg-surface"
                }`}
                onClick={() => step.description && setExpandedStep(isExpanded ? null : step.step_id)}
              >
                <div className="flex items-start gap-3 p-4">
                  <div className="text-2xl pt-0.5 flex-shrink-0">
                    {isDone ? "✅" : isActive ? "▶️" : hasError ? "❌" : "⏳"}
                  </div>
                  <div className="flex-1 min-w-0">
                    <h3 className="font-semibold text-sm">{step.title}</h3>
                    {step.description && <p className="text-xs text-muted mt-0.5">{step.description}</p>}

                    <AnimatePresence>
                      {isExpanded && (step.explanation || step.recommendation) && (
                        <motion.div
                          initial={{ opacity: 0, height: 0 }}
                          animate={{ opacity: 1, height: "auto" }}
                          exit={{ opacity: 0, height: 0 }}
                          className="mt-3 flex flex-col gap-2"
                        >
                          {step.explanation && (
                            <div className="p-2.5 rounded-xl bg-primary/5 border border-primary/10 text-sm text-primary">
                              <span className="font-bold block mb-1">💡 شرح:</span>
                              {step.explanation}
                            </div>
                          )}
                          {step.recommendation && (
                            <div className="p-2.5 rounded-xl bg-success/5 border border-success/10 text-sm text-secondary">
                              <span className="font-bold block mb-1">⭐ أنصح بـ:</span>
                              {step.recommendation}
                            </div>
                          )}
                        </motion.div>
                      )}
                    </AnimatePresence>
                  </div>

                  {step.description && (
                    <ChevronDown
                      size={16}
                      className={`flex-shrink-0 mt-1 text-muted transition-transform ${isExpanded ? "rotate-180" : ""}`}
                    />
                  )}

                  {isActive && step.action_label && (
                    <button
                      disabled={runningAction}
                      onClick={async (e) => {
                        e.stopPropagation();
                        const p = status?.overall_phase;
                        let cmd = ""; let win = false;
                        switch (p) {
                          case "NoWSL": cmd = "wsl --install"; win = true; break;
                          case "WSLNoDistro": cmd = "wsl --install -d Ubuntu-24.04"; win = true; break;
                          case "DistroNoOpenClaw": cmd = "npm install -g openclaw"; break;
                          case "OpenClawNoConfig": cmd = "openclaw onboard --non-interactive --accept-risk --skip-channels --skip-skills --skip-search --skip-ui --auth-choice skip 2>&1"; break;
                          case "OpenClawStopped": cmd = "openclaw gateway start 2>&1"; break;
                          default: cmd = "openclaw doctor --fix --non-interactive 2>&1";
                        }
                        await runAction(step.action_label, cmd, win);
                      }}
                      className="flex-shrink-0 px-4 py-2 rounded-xl bg-primary text-white text-sm font-medium hover:bg-primary-dark transition-colors disabled:opacity-50"
                    >
                      {runningAction ? (
                        <Loader2 size={14} className="animate-spin" />
                      ) : (
                        step.action_label
                      )}
                    </button>
                  )}
                </div>
              </motion.div>
            );
          })}
        </AnimatePresence>
      </div>

      {/* Action output */}
      <AnimatePresence>
        {actionOutput && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: "auto" }}
            exit={{ opacity: 0, height: 0 }}
            className="mb-4"
          >
            <pre className="bg-sidebar text-sidebar-text p-4 rounded-2xl text-xs font-mono max-h-[300px] overflow-y-auto log-viewer">
              {actionOutput}
            </pre>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Model recommendations */}
      {phase === "OpenClawNoConfig" && models.length > 0 && (
        <div className="bg-surface border border-border rounded-2xl p-5 mb-4">
          <div className="flex items-center gap-2 mb-1">
            <Cpu size={18} className="text-primary" />
            <h3 className="font-semibold">اختر موديل المساعد</h3>
          </div>
          <p className="text-sm text-muted mb-4">كل خيار مع توصيتنا:</p>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            {models.map((model) => (
              <div
                key={model.id}
                className={`rounded-2xl border-2 p-4 ${levelColor[model.recommendation_level] || "border-border"}`}
              >
                <span
                  className={`inline-block text-[10px] font-bold px-2 py-0.5 rounded-full text-white mb-2 ${
                    model.recommendation_level === "recommended" ? "bg-primary" :
                    model.recommendation_level === "budget" ? "bg-warning" :
                    model.recommendation_level === "premium" ? "bg-purple-600" : "bg-secondary"
                  }`}
                >
                  {levelLabel[model.recommendation_level]}
                </span>
                <h4 className="font-bold text-sm">{model.name}</h4>
                <p className="text-xs text-muted mb-2">{model.provider}</p>
                <div className="flex flex-wrap gap-1 mb-2">
                  {[
                    [model.speed === "very_fast" ? "⚡ سريع" : "🚀 عادي", "bg-blue-50 text-blue-700"],
                    [model.cost_tier === "free" ? "💰 مجاني" : "💵 مدفوع", "bg-green-50 text-green-700"],
                    [model.quality === "excellent" ? "🏆 ممتاز" : "👍 جيد", "bg-purple-50 text-purple-700"],
                  ].map(([label, cls], i) => (
                    <span key={i} className={`text-[10px] font-bold px-1.5 py-0.5 rounded-md ${cls}`}>{label}</span>
                  ))}
                </div>
                <p className="text-xs leading-relaxed mb-2">{model.explanation_ar}</p>
                <div className="flex flex-wrap gap-1">
                  {model.best_for.map((use) => (
                    <span key={use} className="text-[10px] px-1.5 py-0.5 rounded-md bg-bg border border-border text-muted">{use}</span>
                  ))}
                </div>
                {model.requires_api_key && (
                  <p className="text-[10px] text-warning font-semibold mt-2">🔑 يتطلب API key</p>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Footer */}
      <div className="flex gap-2 pb-4">
        <button
          onClick={checkSystem}
          className="flex items-center gap-1.5 px-3 py-2 rounded-xl text-sm border border-border hover:bg-bg transition-colors"
        >
          <RefreshCw size={14} /> إعادة فحص
        </button>
        {actionOutput && (
          <button
            onClick={() => setActionOutput("")}
            className="flex items-center gap-1.5 px-3 py-2 rounded-xl text-sm border border-border hover:bg-bg transition-colors"
          >
            <Trash2 size={14} /> مسح المخرجات
          </button>
        )}
      </div>
    </motion.div>
  );
}
