import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";
import {
  Monitor,
  Cpu,
  Package,
  MessageSquare,
  Users,
  RefreshCw,
  Play,
  Square,
  RotateCw,
  Stethoscope,
  FileDown,
  Wifi,
  WifiOff,
  Loader2,
  Bot,
  CheckCircle2,
  AlertTriangle,
  Clock,
} from "lucide-react";

interface ChannelSnap {
  name: string;
  connected: boolean;
  status: string;
  health_state: string;
}

interface AgentSnap {
  id: string;
  name: string;
  is_default: boolean;
  session_count: number;
}

interface SystemSnapshot {
  wsl_ok: boolean;
  ubuntu_ok: boolean;
  gateway_ok: boolean;
  gateway_version: string | null;
  gateway_pid: number | null;
  channels: ChannelSnap[];
  active_sessions: number;
  agents: AgentSnap[];
  node_version: string | null;
  openclaw_version: string | null;
  error: string | null;
}

interface OperationLogEntry {
  id: string;
  text: string;
  created_at: string;
}

const cardVariants = {
  hidden: { opacity: 0, y: 12 },
  visible: (i: number) => ({
    opacity: 1,
    y: 0,
    transition: { delay: i * 0.06, duration: 0.3, ease: "easeOut" },
  }),
};

export default function Dashboard({ onOpenAssistant }: { onOpenAssistant: () => void }) {
  const [snap, setSnap] = useState<SystemSnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [actionResult, setActionResult] = useState<string | null>(null);
  const [operationLog, setOperationLog] = useState<OperationLogEntry[]>([]);
  const [busy, setBusy] = useState(false);

  const addOperation = useCallback((text: string) => {
    const entry: OperationLogEntry = {
      id: `${Date.now()}-${Math.random().toString(16).slice(2)}`,
      text,
      created_at: new Date().toLocaleTimeString("ar-SA"),
    };
    setOperationLog((prev) => [entry, ...prev].slice(0, 8));
  }, []);

  const fetchStatus = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<SystemSnapshot>("take_snapshot_cmd");
      setSnap(data);
    } catch {
      setSnap(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 30_000);
    return () => clearInterval(interval);
  }, [fetchStatus]);

  const overallOk = snap?.gateway_ok && snap?.wsl_ok;

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.25 }}
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-5">
        <div>
          <h2 className="text-2xl font-bold text-primary">لوحة التحكم</h2>
          <p className="text-sm text-muted mt-0.5">حالة النظام في الوقت الفعلي</p>
        </div>
        <button
          onClick={fetchStatus}
          disabled={loading}
          className="flex items-center gap-1.5 px-3 py-2 rounded-xl text-sm border border-border bg-surface hover:bg-bg transition-colors disabled:opacity-50"
        >
          <RefreshCw size={14} className={loading ? "animate-spin" : ""} />
          تحديث
        </button>
      </div>

      {/* Status Bar */}
      <motion.div
        className={`px-4 py-3 rounded-2xl text-white font-semibold text-sm mb-5 ${
          overallOk ? "bg-secondary" : "bg-[#b45309]"
        }`}
        initial={{ scaleX: 0.95, opacity: 0 }}
        animate={{ scaleX: 1, opacity: 1 }}
        transition={{ duration: 0.3 }}
      >
        {overallOk ? (
          <span className="flex items-center gap-2">
            <CheckCircle2 size={16} className="text-green-300" />
            النظام يعمل بكفاءة · {snap?.active_sessions ?? 0} جلسة نشطة
          </span>
        ) : (
          <span className="flex items-center gap-2">
            <AlertTriangle size={16} className="text-yellow-300" />
            {snap?.error ?? "Gateway غير شغال"} · {snap?.active_sessions ?? 0} جلسة
          </span>
        )}
      </motion.div>

      {/* Cards Grid */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-5">
        {/* WSL */}
        <motion.div
          custom={0}
          variants={cardVariants}
          initial="hidden"
          animate="visible"
          className="bg-surface border border-border rounded-2xl p-4"
        >
          <div className="flex items-center gap-2 mb-3">
            <Monitor size={20} className="text-primary-light" />
            <h3 className="font-semibold text-sm">WSL</h3>
          </div>
          <div className={`text-lg font-bold ${snap?.wsl_ok ? "text-success" : "text-error"}`}>
            {snap?.wsl_ok ? "🟢 شغال" : "🔴 متوقف"}
          </div>
          <div className="text-xs text-muted mt-1">
            {snap?.node_version ? `Node ${snap.node_version}` : "—"}
          </div>
        </motion.div>

        {/* Gateway */}
        <motion.div
          custom={1}
          variants={cardVariants}
          initial="hidden"
          animate="visible"
          className="bg-surface border border-border rounded-2xl p-4"
        >
          <div className="flex items-center gap-2 mb-3">
            {snap?.gateway_ok ? (
              <Wifi size={20} className="text-success" />
            ) : (
              <WifiOff size={20} className="text-error" />
            )}
            <h3 className="font-semibold text-sm">Gateway</h3>
          </div>
          <div className={`text-lg font-bold ${snap?.gateway_ok ? "text-success" : "text-error"}`}>
            {snap?.gateway_ok ? "🟢 متصل" : "🔴 غير متصل"}
          </div>
          <div className="text-xs text-muted mt-1">
            {snap?.gateway_pid ? `PID: ${snap.gateway_pid}` : "—"}
          </div>
        </motion.div>

        {/* Sessions */}
        <motion.div
          custom={2}
          variants={cardVariants}
          initial="hidden"
          animate="visible"
          className="bg-surface border border-border rounded-2xl p-4"
        >
          <div className="flex items-center gap-2 mb-3">
            <MessageSquare size={20} className="text-primary-light" />
            <h3 className="font-semibold text-sm">الجلسات</h3>
          </div>
          <div className="text-lg font-bold">{snap?.active_sessions ?? 0}</div>
          <div className="text-xs text-muted mt-1">نشطة</div>
        </motion.div>

        {/* OpenClaw */}
        <motion.div
          custom={3}
          variants={cardVariants}
          initial="hidden"
          animate="visible"
          className="bg-surface border border-border rounded-2xl p-4"
        >
          <div className="flex items-center gap-2 mb-3">
            <Package size={20} className="text-primary-light" />
            <h3 className="font-semibold text-sm">OpenClaw</h3>
          </div>
          <div className="text-lg font-bold">{snap?.openclaw_version ?? "?"}</div>
          <div className="text-xs text-muted mt-1">
            {snap?.agents?.length ?? 0} وكلاء
          </div>
        </motion.div>
      </div>

      {/* Maintenance Assistant + Controls */}
      <div className="bg-surface border border-border rounded-2xl p-4 mb-4">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <Bot size={18} className="text-primary" />
            <h3 className="font-semibold text-sm">🎮 التحكم بـ Gateway</h3>
          </div>
          <button
            onClick={onOpenAssistant}
            className="flex items-center gap-1.5 text-xs px-3 py-1.5 rounded-lg border border-border hover:bg-bg transition-colors"
          >
            <Bot size={14} />
            فتح الدردشة
          </button>
        </div>

        <div className="flex flex-wrap gap-2 mb-3">
          <button
            onClick={fetchStatus}
            className="flex items-center gap-1 px-3 py-2 rounded-xl text-sm border border-border bg-surface hover:bg-bg transition-colors"
          >
            <RefreshCw size={14} />
            تحديث
          </button>

          {!snap?.gateway_ok && (
            <button
              disabled={busy}
              onClick={async () => {
                setBusy(true);
                setActionResult("⏳ جاري تشغيل Gateway...");
                addOperation("تشغيل Gateway");
                try {
                  const r = await invoke<string>("start_gateway_cmd");
                  setActionResult(r);
                  addOperation("تم تشغيل Gateway");
                } catch (e) {
                  setActionResult(`❌ ${e}`);
                  addOperation("فشل تشغيل Gateway");
                }
                setBusy(false);
                setTimeout(fetchStatus, 3000);
              }}
              className="flex items-center gap-1 px-3 py-2 rounded-xl text-sm bg-primary text-white hover:bg-primary-dark transition-colors disabled:opacity-50"
            >
              <Play size={14} />
              تشغيل Gateway
            </button>
          )}

          {snap?.gateway_ok && (
            <button
              disabled={busy}
              onClick={async () => {
                setBusy(true);
                setActionResult("⏳ جاري إيقاف Gateway...");
                addOperation("إيقاف Gateway");
                try {
                  const r = await invoke<string>("stop_gateway_cmd");
                  setActionResult(r);
                  addOperation("تم إيقاف Gateway");
                } catch (e) {
                  setActionResult(`❌ ${e}`);
                  addOperation("فشل إيقاف Gateway");
                }
                setBusy(false);
                setTimeout(fetchStatus, 3000);
              }}
              className="flex items-center gap-1 px-3 py-2 rounded-xl text-sm bg-warning text-white hover:bg-amber-600 transition-colors disabled:opacity-50"
            >
              <Square size={14} />
              إيقاف Gateway
            </button>
          )}

          <button
            disabled={busy}
            onClick={async () => {
              setBusy(true);
              setActionResult("⏳ جاري إعادة التشغيل...");
              addOperation("إعادة تشغيل Gateway");
              try {
                const r = await invoke<string>("restart_gateway_cmd");
                setActionResult(r);
                addOperation("تمت إعادة التشغيل");
              } catch (e) {
                setActionResult(`❌ ${e}`);
                addOperation("فشل إعادة التشغيل");
              }
              setBusy(false);
              setTimeout(fetchStatus, 3000);
            }}
            className="flex items-center gap-1 px-3 py-2 rounded-xl text-sm border border-border bg-surface hover:bg-bg transition-colors disabled:opacity-50"
          >
            <RotateCw size={14} />
            إعادة تشغيل
          </button>

          <button
            disabled={busy}
            onClick={async () => {
              setBusy(true);
              setActionResult("⏳ تشخيص...");
              addOperation("بدء تشخيص النظام");
              try {
                const d: any = await invoke("run_diagnosis");
                const msg =
                  d.issues_found.length === 0
                    ? "✅ لا توجد مشاكل"
                    : `⚠️ مشاكل: ${d.issues_found.join("، ")}${
                        d.fixes_applied.length
                          ? `\n✅ تم إصلاح: ${d.fixes_applied.join("، ")}`
                          : ""
                      }`;
                setActionResult(msg);
                addOperation("اكتمل التشخيص");
              } catch (e) {
                setActionResult(`❌ ${e}`);
                addOperation("فشل التشخيص");
              }
              setBusy(false);
            }}
            className="flex items-center gap-1 px-3 py-2 rounded-xl text-sm border border-border bg-surface hover:bg-bg transition-colors disabled:opacity-50"
          >
            <Stethoscope size={14} />
            تشخيص
          </button>

          <button
            disabled={busy}
            onClick={async () => {
              setBusy(true);
              setActionResult("⏳ جاري التصدير...");
              addOperation("تصدير تقرير التشخيص");
              try {
                const report: any = await invoke("export_diagnostics");
                setActionResult(`✅ تم التصدير: ${report.timestamp}`);
                addOperation("تم تصدير تقرير التشخيص");
              } catch (e) {
                setActionResult(`❌ ${e}`);
                addOperation("فشل تصدير التشخيص");
              }
              setBusy(false);
            }}
            className="flex items-center gap-1 px-3 py-2 rounded-xl text-sm border border-border bg-surface hover:bg-bg transition-colors disabled:opacity-50"
          >
            <FileDown size={14} />
            تصدير
          </button>
        </div>

        {busy && (
          <div className="flex items-center gap-2 text-sm text-muted">
            <Loader2 size={14} className="animate-spin" />
            جاري التنفيذ...
          </div>
        )}

        {/* Action Result */}
        <AnimatedResult result={actionResult} />
      </div>

      {/* Agents List */}
      {snap?.agents && snap.agents.length > 0 && (
        <motion.div
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          className="bg-surface border border-border rounded-2xl p-4 mb-4"
        >
          <div className="flex items-center gap-2 mb-3">
            <Users size={18} className="text-primary" />
            <h3 className="font-semibold text-sm">الوكلاء</h3>
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
            {snap.agents.map((agent) => (
              <div
                key={agent.id}
                className="flex items-center justify-between p-3 rounded-xl bg-bg border border-border"
              >
                <div className="flex items-center gap-2">
                  <Cpu size={16} className={agent.is_default ? "text-primary" : "text-muted"} />
                  <div>
                    <div className="text-sm font-semibold">{agent.name}</div>
                    <div className="text-[11px] text-muted">
                      {agent.is_default ? "افتراضي" : ""}
                    </div>
                  </div>
                </div>
                <div className="text-sm font-bold text-primary">{agent.session_count}</div>
              </div>
            ))}
          </div>
        </motion.div>
      )}

      {/* Operation Log */}
      {operationLog.length > 0 && (
        <motion.div
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4 }}
          className="bg-surface border border-border rounded-2xl p-4"
        >
          <div className="flex items-center gap-2 mb-3">
            <Clock size={16} className="text-muted" />
            <h3 className="font-semibold text-sm text-muted">سجل العمليات</h3>
          </div>
          <div className="flex flex-col gap-1.5 max-h-[200px] overflow-y-auto">
            {operationLog.map((entry) => (
              <div key={entry.id} className="flex items-center gap-3 text-[13px] py-1 px-2 rounded-lg hover:bg-bg transition-colors">
                <span className="text-muted text-xs font-mono">{entry.created_at}</span>
                <span>{entry.text}</span>
              </div>
            ))}
          </div>
        </motion.div>
      )}
    </motion.div>
  );
}

function AnimatedResult({ result }: { result: string | null }) {
  if (!result) return null;
  return (
    <motion.div
      initial={{ opacity: 0, height: 0 }}
      animate={{ opacity: 1, height: "auto" }}
      exit={{ opacity: 0, height: 0 }}
      className="mt-3 p-3 rounded-xl text-sm overflow-auto max-h-[200px] font-mono log-viewer bg-sidebar text-sidebar-text"
    >
      {result}
    </motion.div>
  );
}
