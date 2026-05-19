import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface GatewayStatus {
  connected: boolean;
  version: string | null;
  error: string | null;
}

function ModelMonitor() {
  const [gwStatus, setGwStatus] = useState<GatewayStatus | null>(null);

  useEffect(() => {
    const check = async () => {
      try {
        const result = await invoke<GatewayStatus>("get_gateway_status");
        setGwStatus(result);
      } catch (e) {
        console.error("فشل:", e);
      }
    };
    check();
  }, []);

  return (
    <div className="page">
      <div className="page-header">
        <h2>🧠 الموديلات والاستهلاك</h2>
      </div>

      <div className="card">
        <h3>حالة Gateway</h3>
        <div className={`card-status ${gwStatus?.connected ? "ok" : "error"}`}>
          {gwStatus?.connected ? "🟢 متصل" : "🔴 غير متصل"}
        </div>
        {gwStatus?.version && (
          <p className="muted">الإصدار: {gwStatus.version}</p>
        )}
      </div>

      <div className="section">
        <h3>📊 الموديلات النشطة</h3>
        <p className="muted">
          تتبع الاستهلاك والتكاليف قادم في الإصدار القادم.
          حاليًا يمكنك رؤية الحالة العامة لـ Gateway والموديلات المرتبطة.
        </p>
      </div>

      <div className="section">
        <h3>🔗 WebSocket Status</h3>
        <button
          className="btn"
          onClick={async () => {
            try {
              const result: any = await invoke("connect_to_gateway");
              alert(
                result.connected
                  ? "✅ متصل بـ Gateway"
                  : `❌ فشل الاتصال: ${result.error}`
              );
            } catch (e) {
              alert(`❌ خطأ: ${e}`);
            }
          }}
        >
          اختبار الاتصال بـ Gateway
        </button>
      </div>
    </div>
  );
}

export default ModelMonitor;
