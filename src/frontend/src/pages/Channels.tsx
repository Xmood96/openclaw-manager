import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion, AnimatePresence } from "framer-motion";
import {
  Radio,
  Wifi,
  WifiOff,
  RefreshCw,
  Trash2,
  Plus,
  X,
  Loader2,
  CheckCircle2,
  AlertCircle,
  QrCode,
  Key,
  Shield,
  Users,
  Copy,
} from "lucide-react";

interface ChannelData {
  name: string;
  provider: string;
  connected: boolean;
  health_state: string;
  enabled: boolean;
}
interface AgentData {
  id: string;
  name: string;
  is_default: boolean;
  session_count: number;
}

export default function Channels() {
  const [agents, setAgents] = useState<AgentData[]>([]);
  const [channels, setChannels] = useState<Record<string, ChannelData>>({});
  const [activeAgent, setActiveAgent] = useState<string>("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Modals
  const [whatsappModal, setWhatsappModal] = useState(false);
  const [terminalOutput, setTerminalOutput] = useState("");
  const [terminalLoading, setTerminalLoading] = useState(false);
  const [telegramModal, setTelegramModal] = useState(false);
  const [telegramToken, setTelegramToken] = useState("");
  const [telegramLoading, setTelegramLoading] = useState(false);
  const [allowlistModal, setAllowlistModal] = useState(false);
  const [allowlist, setAllowlist] = useState<string[]>([]);
  const [allowlistInput, setAllowlistInput] = useState("");
  const [copied, setCopied] = useState(false);

  // Action feedback
  const [actionMsg, setActionMsg] = useState<string | null>(null);

  const showMsg = (msg: string) => {
    setActionMsg(msg);
    setTimeout(() => setActionMsg(null), 4000);
  };

  const fetchData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const raw = await invoke<string>("get_channels_detailed");
      // Parse raw health JSON
      const health = JSON.parse(raw);
      if (health.error) { setError(health.error); setAgents([]); setChannels({}); }
      else {
        // Extract channels
        const chs: Record<string, ChannelData> = {};
        const chObj = health.channels || {};
        for (const [name, ch] of Object.entries(chObj)) {
          const c = ch as any;
          chs[name] = {
            name,
            provider: c.provider || "",
            connected: c.connected || false,
            health_state: c.healthState || c.health_state || "unknown",
            enabled: c.enabled !== false,
          };
        }
        setChannels(chs);
        // Extract agents
        const ags = (health.agents || []).map((a: any) => ({
          id: a.agentId || a.id || "",
          name: a.name || a.agentId || "?",
          is_default: a.isDefault || false,
          session_count: (a.sessions?.count) || 0,
        }));
        setAgents(ags);
        if (!activeAgent && ags.length > 0) setActiveAgent(ags[0].id);
      }
    } catch (e) {
      setError(`فشل جلب البيانات: ${e}`);
    } finally { setLoading(false); }
  }, [activeAgent]);

  useEffect(() => { fetchData(); }, []);

  const handleReconnect = async (name: string) => {
    showMsg(`⏳ إعادة ربط ${name}...`);
    try {
      const r: any = await invoke("reconnect_channel", { channelName: name });
      showMsg(r.success ? `✅ تم إعادة ربط ${name}` : `❌ ${r.stderr}`);
    } catch (e) { showMsg(`❌ ${e}`); }
    setTimeout(fetchData, 2000);
  };

  const handleRemove = async (name: string) => {
    if (!confirm(`متأكد من حذف قناة "${name}"؟`)) return;
    showMsg(`⏳ حذف ${name}...`);
    try {
      const r: any = await invoke("remove_channel", { channelName: name });
      showMsg(r.success ? `✅ تم حذف ${name}` : `❌ ${r.stderr}`);
    } catch (e) { showMsg(`❌ ${e}`); }
    setTimeout(fetchData, 2000);
  };

  const handleWhatsAppPair = async () => {
    setWhatsappModal(true);
    setTerminalOutput("");
    setTerminalLoading(true);
    try {
      const r: any = await invoke("run_terminal_command", {
        command: "openclaw channels login --channel whatsapp",
      });
      setTerminalOutput(r.success ? r.stdout : `❌ ${r.stderr}`);
    } catch (e) {
      setTerminalOutput(`❌ ${e}`);
    }
    setTerminalLoading(false);
  };

  const handleTelegramConnect = async () => {
    if (!telegramToken.trim()) return;
    setTelegramLoading(true);
    try {
      const r: any = await invoke("login_telegram", { botToken: telegramToken.trim() });
      showMsg(r.success ? "✅ تم ربط تيليجرام" : `❌ ${r.stderr}`);
    } catch (e) { showMsg(`❌ ${e}`); }
    setTelegramLoading(false);
    setTelegramModal(false);
    setTelegramToken("");
    setTimeout(fetchData, 2000);
  };

  const loadAllowlist = async (agentId: string) => {
    try {
      const raw = await invoke<string>("get_agent_allowlist", { agentId });
      setAllowlist(JSON.parse(raw));
    } catch { setAllowlist([]); }
  };

  const openAllowlist = (agentId: string) => {
    setAllowlistModal(true);
    loadAllowlist(agentId);
  };

  const channelList = Object.values(channels);
  const connectedCount = channelList.filter((c) => c.connected).length;

  return (
    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.25 }}>
      {/* Header */}
      <div className="flex items-center justify-between mb-5">
        <div>
          <h2 className="text-2xl font-bold text-primary">القنوات</h2>
          <p className="text-sm text-muted mt-0.5">
            {connectedCount}/{channelList.length} متصلة
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={fetchData}
            disabled={loading}
            className="flex items-center gap-1.5 px-3 py-2 rounded-xl text-sm border border-border bg-surface hover:bg-bg transition-colors"
          >
            <RefreshCw size={14} className={loading ? "animate-spin" : ""} />
            تحديث
          </button>
          <button
            onClick={handleWhatsAppPair}
            className="flex items-center gap-1.5 px-3 py-2 rounded-xl text-sm bg-success text-white hover:bg-green-600 transition-colors"
          >
            <Plus size={14} /> ربط واتساب
          </button>
          <button
            onClick={() => setTelegramModal(true)}
            className="flex items-center gap-1.5 px-3 py-2 rounded-xl text-sm bg-primary text-white hover:bg-primary-dark transition-colors"
          >
            <Plus size={14} /> ربط تيليجرام
          </button>
        </div>
      </div>

      {/* Error */}
      {error && (
        <div className="flex items-center gap-2 p-3 rounded-xl bg-error/10 border border-error/20 text-error text-sm mb-4">
          <AlertCircle size={16} /> {error}
        </div>
      )}

      {/* Agent Tabs */}
      {agents.length > 1 && (
        <div className="flex gap-1.5 mb-4 overflow-x-auto pb-1">
          {agents.map((agent) => (
            <button
              key={agent.id}
              onClick={() => setActiveAgent(agent.id)}
              className={`flex items-center gap-1.5 px-4 py-2 rounded-xl text-sm font-medium whitespace-nowrap transition-all ${
                activeAgent === agent.id
                  ? "bg-primary text-white"
                  : "bg-surface border border-border hover:bg-bg"
              }`}
            >
              <Radio size={14} />
              {agent.name}
              {agent.is_default && <span className="text-[10px] opacity-70">(افتراضي)</span>}
            </button>
          ))}
          <button
            onClick={() => openAllowlist(activeAgent)}
            className="flex items-center gap-1 px-3 py-2 rounded-xl text-xs border border-border bg-surface hover:bg-bg transition-colors whitespace-nowrap ml-auto"
          >
            <Shield size={12} /> Allowlist
          </button>
        </div>
      )}

      {/* Loading */}
      {loading && (
        <div className="flex flex-col items-center justify-center h-40 gap-4 text-muted">
          <Loader2 size={32} className="animate-spin text-primary" />
          <p>جاري فحص القنوات...</p>
        </div>
      )}

      {/* Empty state */}
      {!loading && channelList.length === 0 && (
        <motion.div
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          className="bg-surface border border-dashed border-border rounded-2xl p-10 text-center"
        >
          <Radio size={48} className="mx-auto text-muted mb-4 opacity-40" />
          <h3 className="text-lg font-semibold mb-2">لا توجد قنوات</h3>
          <p className="text-sm text-muted mb-6">
            ابدأ بربط واتساب أو تيليجرام للمساعد
          </p>
          <div className="flex items-center justify-center gap-3">
            <button
              onClick={handleWhatsAppPair}
              className="flex items-center gap-2 px-5 py-3 rounded-2xl bg-success text-white font-semibold hover:bg-green-600 transition-all"
            >
              <QrCode size={18} /> ربط واتساب
            </button>
            <button
              onClick={() => setTelegramModal(true)}
              className="flex items-center gap-2 px-5 py-3 rounded-2xl bg-primary text-white font-semibold hover:bg-primary-dark transition-all"
            >
              <Key size={18} /> ربط تيليجرام
            </button>
          </div>
        </motion.div>
      )}

      {/* Channel List */}
      {!loading && channelList.length > 0 && (
        <div className="flex flex-col gap-2">
          <AnimatePresence>
            {channelList.map((ch, i) => (
              <motion.div
                key={ch.name}
                initial={{ opacity: 0, x: -8 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: i * 0.04 }}
                className={`flex items-center gap-4 p-4 rounded-2xl border ${
                  ch.connected
                    ? "bg-surface border-success/20"
                    : "bg-surface border-border"
                }`}
              >
                {/* Status icon */}
                <div
                  className={`flex items-center justify-center w-11 h-11 rounded-xl ${
                    ch.connected ? "bg-success/10" : "bg-error/10"
                  }`}
                >
                  {ch.connected ? (
                    <Wifi size={20} className="text-success" />
                  ) : (
                    <WifiOff size={20} className="text-error" />
                  )}
                </div>

                {/* Info */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <h3 className="font-bold text-sm capitalize">{ch.name}</h3>
                    <span
                      className={`text-[10px] font-bold px-1.5 py-0.5 rounded-md ${
                        ch.connected
                          ? "bg-success/10 text-success"
                          : "bg-error/10 text-error"
                      }`}
                    >
                      {ch.connected ? "متصل" : "منفصل"}
                    </span>
                    {ch.enabled && (
                      <span className="text-[10px] px-1.5 py-0.5 rounded-md bg-primary/5 text-primary">
                        مفعّل
                      </span>
                    )}
                  </div>
                  <div className="flex items-center gap-3 mt-1 text-xs text-muted">
                    <span>{ch.provider || ch.health_state}</span>
                    <span>الحالة: {ch.health_state || "—"}</span>
                  </div>
                </div>

                {/* Actions */}
                <div className="flex items-center gap-1.5 flex-shrink-0">
                  <button
                    onClick={() => handleReconnect(ch.name)}
                    className="p-2 rounded-lg hover:bg-bg transition-colors"
                    title="إعادة ربط"
                  >
                    <RefreshCw size={16} className="text-muted" />
                  </button>
                  <button
                    onClick={() => handleRemove(ch.name)}
                    className="p-2 rounded-lg hover:bg-error/10 transition-colors"
                    title="حذف"
                  >
                    <Trash2 size={16} className="text-error" />
                  </button>
                </div>
              </motion.div>
            ))}
          </AnimatePresence>
        </div>
      )}

      {/* Action feedback toast */}
      <AnimatePresence>
        {actionMsg && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 20 }}
            className="fixed bottom-6 left-1/2 -translate-x-1/2 bg-sidebar text-sidebar-text px-5 py-3 rounded-2xl shadow-lg text-sm z-50"
          >
            {actionMsg}
          </motion.div>
        )}
      </AnimatePresence>

      {/* WhatsApp Modal — Embedded Terminal */}
      <AnimatePresence>
        {whatsappModal && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/40 flex items-center justify-center z-50 p-4"
            onClick={() => { setWhatsappModal(false); setTerminalOutput(""); }}
          >
            <motion.div
              initial={{ scale: 0.95 }}
              animate={{ scale: 1 }}
              exit={{ scale: 0.95 }}
              className="bg-surface rounded-3xl p-5 max-w-2xl w-full shadow-2xl max-h-[85vh] flex flex-col"
              onClick={(e) => e.stopPropagation()}
            >
              <div className="flex items-center justify-between mb-3 flex-shrink-0">
                <div className="flex items-center gap-2">
                  <QrCode size={20} className="text-success" />
                  <h3 className="font-bold text-lg">ربط واتساب</h3>
                </div>
                <button
                  onClick={() => { setWhatsappModal(false); setTerminalOutput(""); }}
                  className="p-1.5 rounded-lg hover:bg-bg transition-colors"
                >
                  <X size={18} />
                </button>
              </div>

              {terminalLoading ? (
                <div className="flex flex-col items-center gap-4 py-12">
                  <Loader2 size={40} className="animate-spin text-primary" />
                  <p className="text-sm text-muted">جاري فتح الطرفية...</p>
                  <p className="text-xs text-muted">قد يأخذ دقيقة — تابع النافذة الطرفية للمسح</p>
                </div>
              ) : terminalOutput ? (
                <>
                  <div className="bg-[#0a0a0a] text-green-400 rounded-2xl p-4 overflow-y-auto flex-1 min-h-[300px] max-h-[55vh]">
                    <pre className="text-[11px] leading-relaxed font-mono whitespace-pre-wrap break-all log-viewer">
                      {terminalOutput}
                    </pre>
                  </div>
                  <div className="flex gap-2 mt-3 flex-shrink-0">
                    <button
                      onClick={() => {
                        navigator.clipboard.writeText(terminalOutput);
                        setCopied(true);
                        setTimeout(() => setCopied(false), 2000);
                      }}
                      className="flex items-center gap-1.5 px-4 py-2 rounded-xl border border-border text-sm hover:bg-bg"
                    >
                      {copied ? <CheckCircle2 size={14} className="text-success" /> : <Copy size={14} />}
                      {copied ? "تم" : "نسخ"}
                    </button>
                    <button
                      onClick={fetchData}
                      className="flex items-center gap-1.5 px-4 py-2 rounded-xl bg-primary text-white text-sm hover:bg-primary-dark"
                    >
                      <RefreshCw size={14} /> تحديث القنوات
                    </button>
                    <button
                      onClick={() => { setWhatsappModal(false); setTerminalOutput(""); }}
                      className="px-4 py-2 rounded-xl border border-border text-sm hover:bg-bg"
                    >
                      إغلاق
                    </button>
                  </div>
                </>
              ) : null}
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Telegram Modal */}
      <AnimatePresence>
        {telegramModal && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/40 flex items-center justify-center z-50 p-4"
            onClick={() => setTelegramModal(false)}
          >
            <motion.div
              initial={{ scale: 0.95 }}
              animate={{ scale: 1 }}
              exit={{ scale: 0.95 }}
              className="bg-surface rounded-3xl p-6 max-w-md w-full shadow-2xl"
              onClick={(e) => e.stopPropagation()}
            >
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                  <Key size={20} className="text-primary" />
                  <h3 className="font-bold text-lg">ربط تيليجرام</h3>
                </div>
                <button onClick={() => setTelegramModal(false)} className="p-1.5 rounded-lg hover:bg-bg transition-colors">
                  <X size={18} />
                </button>
              </div>

              <label className="block text-sm font-semibold mb-1.5">Bot Token</label>
              <div className="flex gap-2">
                <input
                  type="password"
                  value={telegramToken}
                  onChange={(e) => setTelegramToken(e.target.value)}
                  placeholder="123456:ABC-DEF..."
                  className="flex-1 px-4 py-2.5 rounded-xl border border-border bg-bg text-sm font-mono focus:outline-none focus:border-primary-light"
                  dir="ltr"
                  onKeyDown={(e) => e.key === "Enter" && handleTelegramConnect()}
                />
              </div>
              <p className="text-xs text-muted mt-2">
                احصل على التوكن من <code className="bg-primary/5 text-primary px-1 py-0.5 rounded text-xs">@BotFather</code> في تيليجرام
              </p>

              <div className="flex gap-2 mt-4">
                <button
                  onClick={() => setTelegramModal(false)}
                  className="flex-1 px-4 py-2.5 rounded-xl border border-border text-sm hover:bg-bg transition-colors"
                >
                  إلغاء
                </button>
                <button
                  onClick={handleTelegramConnect}
                  disabled={!telegramToken.trim() || telegramLoading}
                  className="flex-1 flex items-center justify-center gap-1.5 px-4 py-2.5 rounded-xl bg-primary text-white text-sm font-medium hover:bg-primary-dark transition-colors disabled:opacity-50"
                >
                  {telegramLoading ? <Loader2 size={14} className="animate-spin" /> : <Plus size={14} />}
                  ربط
                </button>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Allowlist Modal */}
      <AnimatePresence>
        {allowlistModal && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/40 flex items-center justify-center z-50 p-4"
            onClick={() => setAllowlistModal(false)}
          >
            <motion.div
              initial={{ scale: 0.95 }}
              animate={{ scale: 1 }}
              exit={{ scale: 0.95 }}
              className="bg-surface rounded-3xl p-6 max-w-lg w-full shadow-2xl"
              onClick={(e) => e.stopPropagation()}
            >
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                  <Users size={20} className="text-primary" />
                  <h3 className="font-bold text-lg">Allowlist — {activeAgent}</h3>
                </div>
                <button onClick={() => setAllowlistModal(false)} className="p-1.5 rounded-lg hover:bg-bg transition-colors">
                  <X size={18} />
                </button>
              </div>

              <p className="text-sm text-muted mb-3">
                الأرقام/المستخدمين المسموح لهم بالتفاعل مع هذا الـ agent
              </p>

              {allowlist.length > 0 ? (
                <div className="flex flex-col gap-1.5 mb-3 max-h-[200px] overflow-y-auto">
                  {allowlist.map((entry, i) => (
                    <div key={i} className="flex items-center gap-2 p-2 rounded-xl bg-bg border border-border text-sm">
                      <CheckCircle2 size={14} className="text-success flex-shrink-0" />
                      <code className="text-xs font-mono">{entry}</code>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-sm text-muted italic mb-3">لا يوجد allowlist — الكل مسموح</p>
              )}

              <div className="flex gap-2">
                <input
                  value={allowlistInput}
                  onChange={(e) => setAllowlistInput(e.target.value)}
                  placeholder="+9665XXXXXXXX"
                  className="flex-1 px-4 py-2.5 rounded-xl border border-border bg-bg text-sm font-mono focus:outline-none focus:border-primary-light"
                  dir="ltr"
                />
                <button
                  onClick={() => {
                    if (allowlistInput.trim()) {
                      setAllowlist([...allowlist, allowlistInput.trim()]);
                      setAllowlistInput("");
                    }
                  }}
                  className="px-4 py-2.5 rounded-xl bg-primary text-white text-sm hover:bg-primary-dark transition-colors"
                >
                  <Plus size={14} />
                </button>
              </div>
              <p className="text-[10px] text-muted mt-2">هذا للعرض فقط — الحفظ الفعلي يحتاج أمر OpenClaw</p>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}
