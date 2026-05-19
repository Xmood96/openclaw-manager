import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ChannelData {
  name: string;
  connected: boolean;
  status: string;
}

function Channels() {
  const [channels, setChannels] = useState<ChannelData[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchChannels = async () => {
      try {
        // نستخدم health summary عشان نجيب القنوات
        const health: any = await invoke("get_health_summary");
        if (health.channels) {
          setChannels(health.channels);
        }
      } catch (e) {
        console.error("فشل جلب القنوات:", e);
      } finally {
        setLoading(false);
      }
    };
    fetchChannels();
  }, []);

  return (
    <div className="page">
      <div className="page-header">
        <h2>📡 القنوات</h2>
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
        </div>
      ) : (
        <div className="channel-list">
          {channels.map((ch) => (
            <div key={ch.name} className="channel-card">
              <div className="channel-icon">
                {ch.connected ? "🟢" : "🔴"}
              </div>
              <div className="channel-info">
                <h3>{ch.name}</h3>
                <span className={ch.connected ? "status-ok" : "status-err"}>
                  {ch.connected ? "متصل" : "غير متصل"}
                </span>
                {ch.status && <p className="muted">{ch.status}</p>}
              </div>
              {!ch.connected && (
                <button className="btn btn-sm btn-warning">إعادة ربط</button>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default Channels;
