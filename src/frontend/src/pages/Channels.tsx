import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

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

function Channels() {
  const [health, setHealth] = useState<HealthSummary | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchChannels = async () => {
      try {
        const h: HealthSummary = await invoke("get_health_summary");
        setHealth(h);
      } catch (e) {
        console.error("فشل جلب القنوات:", e);
      } finally {
        setLoading(false);
      }
    };
    fetchChannels();
  }, []);

  const channels = health?.channels ?? [];

  return (
    <div className="page">
      <div className="page-header">
        <h2>📡 القنوات</h2>
        {health && (
          <div className={`status-badge ${health.overall === "good" ? "ok" : health.overall === "degraded" ? "warning" : "error"}`}>
            {health.overall === "good" ? "🟢" : health.overall === "degraded" ? "🟡" : "🔴"} {health.overall === "good" ? "كل شيء تمام" : health.overall === "degraded" ? "بعض المشاكل" : "عطل"}
          </div>
        )}
      </div>

      {loading ? (
        <div className="page-loading">
          <div className="spinner" />
          <p>جاري فحص القنوات...</p>
        </div>
      ) : channels.length === 0 ? (
        <div className="empty-state">
          <p className="muted">لا توجد قنوات مربوطة حاليًا</p>
          <p>لإضافة قناة، افتح إعدادات OpenClaw وأضف WhatsApp أو Telegram</p>
          <div className="section" style={{ marginTop: 16 }}>
            <h3>📊 ملخص النظام</h3>
            <div className="stats-grid">
              {health?.active_sessions !== undefined && (
                <div className="stat-item">
                  <span className="stat-value">{health.active_sessions}</span>
                  <span className="stat-label">جلسات نشطة</span>
                </div>
              )}
              {health?.agents && (
                <div className="stat-item">
                  <span className="stat-value">{health.agents.length}</span>
                  <span className="stat-label">وكلاء</span>
                </div>
              )}
            </div>
          </div>
        </div>
      ) : (
        <>
          <div className="channel-list">
            {channels.map((ch) => (
              <div key={ch.name} className="channel-card">
                <div className="channel-icon">
                  {ch.connected ? "🟢" : "🔴"}
                </div>
                <div className="channel-info">
                  <h3>{ch.name === "whatsapp" ? "WhatsApp" : ch.name}</h3>
                  <span className={ch.connected ? "status-ok" : "status-err"}>
                    {ch.connected ? "🟢 متصل" : "🔴 غير متصل"}
                  </span>
                  {ch.status && <p className="muted">{ch.status}</p>}
                </div>
                {!ch.connected && (
                  <button
                    className="btn btn-sm btn-warning"
                    onClick={async () => {
                      try {
                        const r = await invoke("run_playbook", { playbookId: "run-doctor" });
                        alert(r);
                      } catch (e) {
                        alert(`خطأ: ${e}`);
                      }
                    }}
                  >
                    🔄 إعادة ربط
                  </button>
                )}
              </div>
            ))}
          </div>

          {/* Quick stats */}
          <div className="section" style={{ marginTop: 16 }}>
            <h3>📊 إحصائيات</h3>
            <div className="stats-grid">
              {health?.active_sessions !== undefined && (
                <div className="stat-item">
                  <span className="stat-value">{health.active_sessions}</span>
                  <span className="stat-label">جلسات نشطة</span>
                </div>
              )}
              {health?.agents && (
                <div className="stat-item">
                  <span className="stat-value">{health.agents.length}</span>
                  <span className="stat-label">وكلاء</span>
                </div>
              )}
              <div className="stat-item">
                <span className="stat-value">{channels.length}</span>
                <span className="stat-label">قنوات</span>
              </div>
              <div className="stat-item">
                <span className="stat-value">{channels.filter(c => c.connected).length}</span>
                <span className="stat-label">قنوات متصلة</span>
              </div>
            </div>
          </div>
        </>
      )}

      {health?.recommended_action && (
        <div className="alert alert-warning" style={{ marginTop: 16 }}>
          💡 {health.recommended_action}
        </div>
      )}
    </div>
  );
}

export default Channels;
