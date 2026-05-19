import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface SystemSnapshot {
  gateway_ok: boolean;
  gateway_version: string | null;
  active_sessions: number;
  wsl_ok: boolean;
  error: string | null;
}

function AIAssistant() {
  const [snap, setSnap] = useState<SystemSnapshot | null>(null);
  const [status, setStatus] = useState<string>("");
  const [logs, setLogs] = useState<string[]>([]);
  const [running, setRunning] = useState(false);

  const refresh = async () => {
    try { setSnap(await invoke<SystemSnapshot>("take_snapshot_cmd")); } catch {}
  };
  useEffect(() => { refresh(); }, []);

  const addLog = (msg: string) => setLogs((prev) => [
    `🕐 ${new Date().toLocaleTimeString("ar-SA")} ${msg}`, ...prev,
  ]);

  const runDiagnosis = async () => {
    setRunning(true); setStatus("🔍 جاري تشخيص النظام...");
    try {
      const result: any = await invoke("run_diagnosis");
      const msg = `تشخيص: ${result.overall_status} — ${result.issues_found.length || 0} مشاكل`;
      setStatus(msg); addLog(msg);
      await refresh();
    } catch (e: any) { setStatus(`❌ ${e}`); }
    setRunning(false);
  };

  const startGateway = async () => {
    setRunning(true); setStatus("▶️ جاري تشغيل Gateway...");
    try {
      const r = await invoke<string>("start_gateway_cmd");
      setStatus(r); addLog(r);
      setTimeout(refresh, 3000);
    } catch (e: any) { setStatus(`❌ ${e}`); }
    setRunning(false);
  };

  const stopGateway = async () => {
    setRunning(true); setStatus("⏹️ جاري إيقاف Gateway...");
    try {
      const r = await invoke<string>("stop_gateway_cmd");
      setStatus(r); addLog(r);
      setTimeout(refresh, 2000);
    } catch (e: any) { setStatus(`❌ ${e}`); }
    setRunning(false);
  };

  const restartGateway = async () => {
    setRunning(true); setStatus("🔄 جاري إعادة تشغيل Gateway...");
    try {
      await invoke<string>("stop_gateway_cmd");
      const r = await invoke<string>("start_gateway_cmd");
      setStatus(r); addLog(r);
      setTimeout(refresh, 3000);
    } catch (e: any) { setStatus(`❌ ${e}`); }
    setRunning(false);
  };

  const exportDiagnostics = async () => {
    setRunning(true); setStatus("📤 جاري تصدير التقرير...");
    try {
      const report: any = await invoke("export_diagnostics");
      const blob = new Blob([JSON.stringify(report, null, 2)], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a"); a.href = url;
      a.download = `diagnostics-${new Date().toISOString().slice(0,10)}.json`;
      a.click();
      setStatus("✅ تم تصدير التقرير"); addLog("تم تصدير تقرير التشخيص");
    } catch (e: any) { setStatus(`❌ ${e}`); }
    setRunning(false);
  };

  return (
    <div className="page">
      <div className="page-header">
        <h2>🤖 مساعد الصيانة</h2>
        <button className="btn btn-sm" onClick={refresh}>🔄 تحديث</button>
      </div>

      {/* حالة سريعة */}
      <div className="status-bar" style={{ background: snap?.gateway_ok ? "green" : snap?.wsl_ok ? "orange" : "red" }}>
        {snap?.gateway_ok ? "✅ Gateway شغال" : "⚠️ Gateway واقف"} · {snap?.active_sessions ?? 0} جلسات
      </div>

      <div className="action-buttons">
        <button className="btn btn-primary" onClick={runDiagnosis} disabled={running}>🔍 تشخيص</button>
        {!snap?.gateway_ok && (
          <button className="btn btn-primary" onClick={startGateway} disabled={running}>▶️ تشغيل Gateway</button>
        )}
        {snap?.gateway_ok && (
          <button className="btn btn-warning" onClick={stopGateway} disabled={running}>⏹️ إيقاف</button>
        )}
        <button className="btn" onClick={restartGateway} disabled={running}>🔄 إعادة تشغيل</button>
        <button className="btn" onClick={exportDiagnostics} disabled={running}>📤 تصدير</button>
      </div>

      {status && (
        <div className="alert alert-info">
          {running && <span className="spinner-sm" />} {status}
        </div>
      )}

      <div className="section">
        <h3>📋 سجل العمليات</h3>
        <div className="log-list">
          {logs.length === 0 ? <p className="muted">لا توجد عمليات</p> : logs.map((l, i) => <div key={i} className="log-entry">{l}</div>)}
        </div>
      </div>
    </div>
  );
}

export default AIAssistant;
