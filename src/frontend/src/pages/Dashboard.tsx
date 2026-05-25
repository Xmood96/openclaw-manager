import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ChannelSnap {
  name: string;
  connected: boolean;
  status: string;
  health_state: string;
  last_event_at: string | null;
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

function Dashboard({ onOpenAssistant }: { onOpenAssistant: () => void }) {
  const [snap, setSnap] = useState<SystemSnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [actionResult, setActionResult] = useState<string | null>(null);
  const [operationLog, setOperationLog] = useState<OperationLogEntry[]>([]);

  const addOperation = (text: string) => {
    const entry: OperationLogEntry = {
      id: `${Date.now()}-${Math.random().toString(16).slice(2)}`,
      text,
      created_at: new Date().toLocaleTimeString("ar-SA"),
    };
    setOperationLog((prev) => [entry, ...prev].slice(0, 8));
  };

  const fetchStatus = async () => {
    try {
      const result = await invoke<SystemSnapshot>("take_snapshot_cmd");
      setSnap(result);
    } catch (e) {
      console.error("فشل:", e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 15000);
    return () => clearInterval(interval);
  }, []);

  const overallColor =
    snap?.gateway_ok ? "green" :
    snap?.wsl_ok ? "orange" : "red";

  const overallText =
    snap?.gateway_ok ? "✅ النظام يعمل بكفاءة" :
    snap?.wsl_ok ? "⚠️ Gateway غير شغال" : "❌ WSL غير متصل";

  if (loading) {
    return (
      <div className="page-loading">
        <div className="spinner" />
        <p>⏳ فحص سريع...</p>
      </div>
    );
  }

  return (
    <div className="page">
      <div className="page-header">
        <h2>لوحة التحكم</h2>
        <button className="btn btn-sm" onClick={fetchStatus}>🔄 تحديث</button>
      </div>

      <div className="status-bar" style={{ background: overallColor }}>
        {overallText}
      </div>

      <div className="cards-grid">
        {/* WSL */}
        <div className="card">
          <div className="card-header">
            <span className="card-icon">🐧</span><h3>WSL</h3>
          </div>
          <div className={`card-status ${snap?.wsl_ok ? "ok" : "error"}`}>
            {snap?.wsl_ok ? "🟢 شغال" : "🔴 موقف"}
          </div>
          {snap?.node_version && <div className="card-detail">Node {snap.node_version}</div>}
        </div>

        {/* Gateway */}
        <div className="card">
          <div className="card-header">
            <span className="card-icon">🔵</span><h3>Gateway</h3>
          </div>
          <div className={`card-status ${snap?.gateway_ok ? "ok" : "error"}`}>
            {snap?.gateway_ok ? "🟢 متصل" : "🔴 غير متصل"}
          </div>
          {snap?.gateway_version && <div className="card-detail">v{snap.gateway_version}</div>}
          {snap?.gateway_pid && snap.gateway_pid > 0 && (
            <div className="card-detail">PID: {snap.gateway_pid}</div>
          )}
        </div>

        {/* Sessions */}
        <div className="card">
          <div className="card-header">
            <span className="card-icon">💬</span><h3>الجلسات</h3>
          </div>
          <div className="card-status ok">{snap?.active_sessions ?? 0} نشطة</div>
          {snap?.agents && snap.agents.length > 0 && (
            <div className="card-detail">
              {snap.agents.map(a => (
                <span key={a.id} className="agent-badge">
                  {a.is_default ? "⭐" : "🤖"} {a.id}: {a.session_count}
                </span>
              ))}
            </div>
          )}
        </div>

        {/* OpenClaw */}
        <div className="card">
          <div className="card-header">
            <span className="card-icon">📦</span><h3>OpenClaw</h3>
          </div>
          <div className="card-status ok">
            {snap?.openclaw_version ? `v${snap.openclaw_version}` : "?"}
          </div>
          <div className="card-detail">
            {snap?.agents?.length ?? 0} وكلاء
          </div>
        </div>
      </div>

      <div className="section">
        <div className="page-header" style={{ marginBottom: 12 }}>
          <h3>🤖 مساعد الصيانة</h3>
          <button className="btn btn-sm" onClick={onOpenAssistant}>💬 فتح الدردشة</button>
        </div>

        <div
          className="status-bar"
          style={{ marginBottom: 12, background: snap?.gateway_ok ? "#0f766e" : "#b45309" }}
        >
          {snap?.gateway_ok ? "🟢 Gateway يعمل" : "⚠️ Gateway واقف"} · {snap?.active_sessions ?? 0} جلسات
        </div>

        <div className="action-buttons" style={{ flexWrap: "wrap" }}>
          <button className="btn btn-sm" onClick={fetchStatus}>🔄 تحديث</button>
          <button
            className="btn btn-sm"
            onClick={async () => {
              setActionResult("⏳ تشخيص...");
              addOperation("بدء تشخيص النظام");
              try {
                const d: any = await invoke("run_diagnosis");
                const msg = d.issues_found.length === 0
                  ? "✅ لا توجد مشاكل"
                  : `⚠️ مشاكل: ${d.issues_found.join("، ")}${d.fixes_applied.length ? `\n✅ تم إصلاح: ${d.fixes_applied.join("، ")}` : ""}`;
                setActionResult(msg);
                addOperation("اكتمل التشخيص");
              } catch (e) {
                setActionResult(`❌ ${e}`);
                addOperation("فشل التشخيص");
              }
            }}
          >
            🔍 تشخيص
          </button>
          {!snap?.gateway_ok && (
            <button
              className="btn btn-primary btn-sm"
              onClick={async () => {
                setActionResult("⏳ جاري تشغيل Gateway...");
                addOperation("تشغيل Gateway");
                const r = await invoke<string>("start_gateway_cmd");
                setActionResult(r);
                setTimeout(fetchStatus, 3000);
              }}
            >
              ▶️ تشغيل Gateway
            </button>
          )}
          <button
            className="btn btn-sm"
            onClick={async () => {
              setActionResult("⏳ جاري إعادة التشغيل...");
              addOperation("إعادة تشغيل Gateway");
              const r = await invoke<string>("restart_gateway_cmd");
              setActionResult(r);
              setTimeout(fetchStatus, 3000);
            }}
          >
            🔄 إعادة تشغيل
          </button>
          <button
            className="btn btn-sm"
            onClick={async () => {
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
            }}
          >
            📤 تصدير
          </button>
        </div>

        {actionResult && <div className="alert alert-info">{actionResult}</div>}

        <div style={{ marginTop: 12 }}>
          <h3 style={{ marginBottom: 8 }}>📋 سجل العمليات</h3>
          {operationLog.length === 0 ? (
            <p className="muted">لا توجد عمليات</p>
          ) : (
            <div className="log-list">
              {operationLog.map((entry) => (
                <div key={entry.id} className="log-entry">
                  <strong>{entry.created_at}</strong> - {entry.text}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Gateway Controls */}
      <div className="section">
        <h3>🎮 التحكم بـ Gateway</h3>
        <div className="action-buttons">
          {!snap?.gateway_ok && (
            <button className="btn btn-primary" onClick={async () => {
              setActionResult("⏳ جاري تشغيل Gateway...");
              const r = await invoke<string>("start_gateway_cmd");
              setActionResult(r);
              setTimeout(fetchStatus, 3000);
            }}>▶️ تشغيل Gateway</button>
          )}
          {snap?.gateway_ok && (
            <button className="btn btn-warning" onClick={async () => {
              setActionResult("⏳ جاري إيقاف Gateway...");
              const r = await invoke<string>("stop_gateway_cmd");
              setActionResult(r);
              setTimeout(fetchStatus, 2000);
            }}>⏹️ إيقاف Gateway</button>
          )}
          <button className="btn" onClick={async () => {
            setActionResult("⏳ جاري إعادة التشغيل...");
            const r = await invoke<string>("restart_gateway_cmd");
            setActionResult(r);
            setTimeout(fetchStatus, 3000);
          }}>🔄 إعادة تشغيل</button>
          <button className="btn btn-sm" onClick={async () => {
            setActionResult("⏳ تشخيص...");
            try {
              const d: any = await invoke("run_diagnosis");
              const msg = d.issues_found.length === 0
                ? "✅ لا توجد مشاكل"
                : `⚠️ مشاكل: ${d.issues_found.join("، ")}${d.fixes_applied.length ? `\n✅ تم إصلاح: ${d.fixes_applied.join("، ")}` : ""}`;
              setActionResult(msg);
            } catch (e) {
              setActionResult(`❌ ${e}`);
            }
          }}>🩺 تشخيص</button>
        </div>
        {actionResult && <div className="alert alert-info">{actionResult}</div>}
      </div>

      {/* Channels */}
      {snap?.channels && snap.channels.length > 0 && (
        <div className="section">
          <h3>📡 القنوات</h3>
          <div className="channel-list">
            {snap.channels.map(ch => (
              <div key={ch.name} className="channel-item">
                <span>{ch.connected ? "🟢" : "🔴"}</span>
                <span className="channel-name">{ch.name === "whatsapp" ? "WhatsApp" : ch.name}</span>
                <span className={`channel-status ${ch.connected ? "status-ok" : "status-err"}`}>
                  {ch.connected ? "متصل" : "غير متصل"}
                </span>
                {ch.status && ch.status !== "unknown" && (
                  <span className="channel-detail">{ch.health_state}</span>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {snap?.error && <div className="alert alert-warning">⚠️ {snap.error}</div>}
    </div>
  );
}

export default Dashboard;
