import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface HealthSummary {
  overall: string;
  wsl: { running: boolean; distro: string };
  gateway: { reachable: boolean; version: string | null; uptime: string | null };
  channels: { name: string; connected: boolean; status: string }[];
  active_sessions: number;
  diagnosis: string | null;
  recommended_action: string | null;
}

function Dashboard() {
  const [health, setHealth] = useState<HealthSummary | null>(null);
  const [loading, setLoading] = useState(true);

  const fetchHealth = async () => {
    try {
      const result = await invoke<HealthSummary>("get_health_summary");
      setHealth(result);
    } catch (e) {
      console.error("فشل جلب الحالة:", e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchHealth();
    const interval = setInterval(fetchHealth, 30000);
    return () => clearInterval(interval);
  }, []);

  const overallColor =
    health?.overall === "good"
      ? "green"
      : health?.overall === "degraded"
      ? "orange"
      : "red";

  if (loading) {
    return (
      <div className="page-loading">
        <div className="spinner" />
        <p>جاري فحص النظام...</p>
      </div>
    );
  }

  return (
    <div className="page">
      <div className="page-header">
        <h2>لوحة التحكم</h2>
        <button className="btn btn-sm" onClick={fetchHealth}>
          🔄 تحديث
        </button>
      </div>

      <div className="status-bar" style={{ background: overallColor }}>
        {health?.overall === "good"
          ? "✅ النظام يعمل بكفاءة"
          : health?.overall === "degraded"
          ? "⚠️ النظام يعمل مع بعض المشاكل"
          : "❌ النظام يحتاج تدخلاً"}
      </div>

      <div className="cards-grid">
        <div className="card">
          <div className="card-header">
            <span className="card-icon">🐧</span>
            <h3>WSL</h3>
          </div>
          <div className={`card-status ${health?.wsl.running ? "ok" : "error"}`}>
            {health?.wsl.running ? "🟢 شغال" : "🔴 موقف"}
          </div>
          <div className="card-detail">{health?.wsl.distro}</div>
        </div>

        <div className="card">
          <div className="card-header">
            <span className="card-icon">🔵</span>
            <h3>Gateway</h3>
          </div>
          <div
            className={`card-status ${health?.gateway.reachable ? "ok" : "error"}`}
          >
            {health?.gateway.reachable ? "🟢 متصل" : "🔴 غير متصل"}
          </div>
          {health?.gateway.version && (
            <div className="card-detail">الإصدار: {health.gateway.version}</div>
          )}
          {health?.gateway.uptime && (
            <div className="card-detail">مدة التشغيل: {health.gateway.uptime}</div>
          )}
        </div>

        <div className="card">
          <div className="card-header">
            <span className="card-icon">📊</span>
            <h3>الجلسات</h3>
          </div>
          <div className="card-status ok">
            {health?.active_sessions ?? 0} نشطة
          </div>
        </div>
      </div>

      {health?.channels && health.channels.length > 0 && (
        <div className="section">
          <h3>📡 القنوات</h3>
          <div className="channel-list">
            {health.channels.map((ch) => (
              <div key={ch.name} className="channel-item">
                <span className={ch.connected ? "status-ok" : "status-err"}>
                  {ch.connected ? "✅" : "❌"}
                </span>
                <span className="channel-name">{ch.name}</span>
                <span className="channel-status">{ch.status}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {health?.recommended_action && (
        <div className="alert alert-warning">
          💡 {health.recommended_action}
        </div>
      )}
    </div>
  );
}

export default Dashboard;
