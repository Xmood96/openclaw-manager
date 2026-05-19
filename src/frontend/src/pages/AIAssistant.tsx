import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function AIAssistant() {
  const [status, setStatus] = useState<string>("");
  const [logs, setLogs] = useState<string[]>([]);
  const [running, setRunning] = useState(false);

  const runDiagnosis = async () => {
    setRunning(true);
    setStatus("جاري تشخيص النظام...");
    try {
      const result: any = await invoke("run_diagnosis");
      setStatus(
        `تم العثور على ${result.issues_found.length || 0} مشكلة، وتم إصلاح ${result.fixes_applied.length || 0}`
      );
      setLogs((prev) => [
        `🕐 ${new Date().toLocaleTimeString("ar-SA")} تشخيص: ${result.overall_status}`,
        ...prev,
      ]);
    } catch (e) {
      setStatus(`❌ خطأ: ${e}`);
    } finally {
      setRunning(false);
    }
  };

  const restartGateway = async () => {
    setRunning(true);
    setStatus("جاري إعادة تشغيل Gateway...");
    try {
      const result: any = await invoke("run_playbook", {
        playbookId: "gateway-restart",
      });
      setStatus(result);
      setLogs((prev) => [
        `🕐 ${new Date().toLocaleTimeString("ar-SA")} ${result}`,
        ...prev,
      ]);
    } catch (e) {
      setStatus(`❌ خطأ: ${e}`);
    } finally {
      setRunning(false);
    }
  };

  return (
    <div className="page">
      <div className="page-header">
        <h2>🤖 مساعد الصيانة</h2>
      </div>

      <div className="action-buttons">
        <button
          className="btn btn-primary"
          onClick={runDiagnosis}
          disabled={running}
        >
          🔍 تشخيص النظام
        </button>
        <button
          className="btn btn-warning"
          onClick={restartGateway}
          disabled={running}
        >
          🔄 إعادة تشغيل Gateway
        </button>
      </div>

      {status && (
        <div className="alert alert-info">
          {running && <div className="spinner-sm" />}
          {status}
        </div>
      )}

      <div className="section">
        <h3>📋 سجل العمليات</h3>
        <div className="log-list">
          {logs.length === 0 ? (
            <p className="muted">لا توجد عمليات سابقة</p>
          ) : (
            logs.map((log, i) => (
              <div key={i} className="log-entry">
                {log}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

export default AIAssistant;
