import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";
import {
  Radio,
  CheckCircle2,
  XCircle,
  Activity,
  Users,
  Loader2,
  AlertCircle,
} from "lucide-react";

interface ChannelInfo {
  name: string;
  connected: boolean;
  status: string;
}
interface AgentInfo {
  id: string;
  name: string;
  is_default: boolean;
  session_count: number;
}
interface HealthSummary {
  overall: string;
  wsl: { running: boolean; distro: string };
  gateway: { reachable: boolean; version: string | null; uptime: string | null };
  channels: ChannelInfo[];
  agents: AgentInfo[];
  active_sessions: number;
  diagnosis: string | null;
  recommended_action: string | null;
}

export default function Channels() {
  const [health, setHealth] = useState<HealthSummary | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const h: HealthSummary = await invoke("get_health_summary");
        setHealth(h);
      } catch (e) {
        console.error("فشل جلب القنوات:", e);
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  const channels = health?.channels ?? [];
  const connectedCount = channels.filter((c) => c.connected).length;

  return (
    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.25 }}>
      <div className="flex items-center justify-between mb-5">
        <div>
          <h2 className="text-2xl font-bold text-primary">القنوات</h2>
          <p className="text-sm text-muted mt-0.5">إدارة قنوات الاتصال</p>
        </div>
        {health && (
          <div
            className={`flex items-center gap-1.5 text-xs px-3 py-1.5 rounded-full ${
              health.overall === "good"
                ? "bg-success/10 text-success"
                : health.overall === "degraded"
                ? "bg-warning/10 text-warning"
                : "bg-error/10 text-error"
            }`}
          >
            <Activity size={12} />
            {health.overall === "good" ? "النظام تمام" : health.overall === "degraded" ? "بعض المشاكل" : "عطل"}
          </div>
        )}
      </div>

      {loading ? (
        <div className="flex flex-col items-center justify-center h-60 gap-4 text-muted">
          <Loader2 size={32} className="animate-spin text-primary" />
          <p>جاري فحص القنوات...</p>
        </div>
      ) : channels.length === 0 ? (
        <motion.div
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          className="bg-surface border border-dashed border-border rounded-2xl p-10 text-center"
        >
          <Radio size={48} className="mx-auto text-muted mb-4 opacity-40" />
          <h3 className="text-lg font-semibold mb-2">لا توجد قنوات</h3>
          <p className="text-sm text-muted mb-4">
            لإضافة قناة، افتح إعدادات OpenClaw وأضف WhatsApp أو Telegram
          </p>
        </motion.div>
      ) : (
        <>
          {/* Channel cards */}
          <div className="grid gap-3">
            {channels.map((ch, i) => (
              <motion.div
                key={ch.name}
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: i * 0.05 }}
                className="flex items-center gap-4 p-4 rounded-2xl bg-surface border border-border"
              >
                <div
                  className={`flex items-center justify-center w-10 h-10 rounded-xl ${
                    ch.connected ? "bg-success/10" : "bg-error/10"
                  }`}
                >
                  {ch.connected ? (
                    <CheckCircle2 size={20} className="text-success" />
                  ) : (
                    <XCircle size={20} className="text-error" />
                  )}
                </div>
                <div className="flex-1 min-w-0">
                  <h3 className="font-semibold text-sm capitalize">
                    {ch.name === "whatsapp" ? "WhatsApp" : ch.name}
                  </h3>
                  <p className={`text-xs font-medium ${ch.connected ? "text-success" : "text-error"}`}>
                    {ch.connected ? "متصل" : "غير متصل"}
                  </p>
                  {ch.status && <p className="text-xs text-muted truncate mt-0.5">{ch.status}</p>}
                </div>
                {!ch.connected && (
                  <button
                    onClick={async () => {
                      try {
                        const r = await invoke("run_playbook", { playbookId: "run-doctor" });
                        alert(r);
                      } catch (e) {
                        alert(`خطأ: ${e}`);
                      }
                    }}
                    className="text-xs px-3 py-1.5 rounded-lg bg-warning text-white hover:bg-amber-600 transition-colors flex-shrink-0"
                  >
                    إعادة ربط
                  </button>
                )}
              </motion.div>
            ))}
          </div>

          {/* Stats */}
          <motion.div
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 }}
            className="grid grid-cols-4 gap-3 mt-4"
          >
            <StatCard icon={Activity} label="جلسات نشطة" value={health?.active_sessions ?? 0} />
            <StatCard icon={Users} label="وكلاء" value={health?.agents?.length ?? 0} />
            <StatCard icon={Radio} label="قنوات" value={channels.length} />
            <StatCard icon={CheckCircle2} label="متصلة" value={connectedCount} color="text-success" />
          </motion.div>
        </>
      )}

      {health?.recommended_action && (
        <div className="mt-4 p-3 rounded-xl bg-warning/10 border border-warning/20 text-warning text-sm flex items-start gap-2">
          <AlertCircle size={16} className="flex-shrink-0 mt-0.5" />
          {health.recommended_action}
        </div>
      )}
    </motion.div>
  );
}

function StatCard({
  icon: Icon,
  label,
  value,
  color = "text-primary",
}: {
  icon: React.ComponentType<any>;
  label: string;
  value: number;
  color?: string;
}) {
  return (
    <div className="bg-surface border border-border rounded-xl p-3 text-center">
      <Icon size={16} className={`mx-auto mb-1 ${color}`} />
      <div className="text-lg font-bold">{value}</div>
      <div className="text-[11px] text-muted">{label}</div>
    </div>
  );
}
