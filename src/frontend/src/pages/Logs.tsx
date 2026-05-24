import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function Logs() {
  const [logs, setLogs] = useState<string>("");
  const [loading, setLoading] = useState(false);

  const fetchLogs = async () => {
    setLoading(true);
    try {
      const result = await invoke<string>("read_gateway_logs", { lines: 100 });
      setLogs(result);
    } catch (e) {
      setLogs(`خطأ في جلب السجلات: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="page">
      <div className="page-header">
        <h2>📋 سجلات النظام</h2>
        <button className="btn btn-sm" onClick={fetchLogs} disabled={loading}>
          {loading ? "جاري التحميل..." : "🔄 تحديث"}
        </button>
      </div>
      <pre className="log-viewer">{logs || "اضغط 'تحديث' لعرض السجلات"}</pre>
    </div>
  );
}

export default Logs;
