import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

function Settings() {
  const [firebaseKey, setFirebaseKey] = useState("");
  const [projectId, setProjectId] = useState("");
  const [saved, setSaved] = useState(false);
  const [diagResult, setDiagResult] = useState<string | null>(null);
  const [doctorResult, setDoctorResult] = useState<string | null>(null);

  useEffect(() => {
    // Load saved settings from backend
    // TODO: actual secure storage load
  }, []);

  const saveSettings = async () => {
    setSaved(false);
    // Simulate save — in production use tauri-plugin-store or keyring
    await new Promise(r => setTimeout(r, 500));
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div className="page">
      <div className="page-header">
        <h2>⚙️ الإعدادات</h2>
      </div>

      <div className="section">
        <h3>🔗 Firebase</h3>
        <div className="form-group">
          <label>API Key</label>
          <input
            type="password"
            value={firebaseKey}
            onChange={(e) => setFirebaseKey(e.target.value)}
            placeholder="أدخل مفتاح Firebase"
          />
        </div>
        <div className="form-group">
          <label>Project ID</label>
          <input
            type="text"
            value={projectId}
            onChange={(e) => setProjectId(e.target.value)}
            placeholder="أدخل Project ID"
          />
        </div>
        <button className="btn btn-primary" onClick={saveSettings}>
          {saved ? "✅ تم الحفظ" : "💾 حفظ"}
        </button>
      </div>

      <div className="section">
        <h3>🔧 Diagnostics</h3>
        <div className="action-buttons">
          <button
            className="btn"
            onClick={async () => {
              setDiagResult("⏳ جاري التصدير...");
              try {
                const report: any = await invoke("export_diagnostics");
                setDiagResult(`
✅ تم تصدير تقرير التشخيص
⏱ ${report.timestamp}
🐧 WSL: ${report.wsl.running ? "🟢 شغال" : "🔴 موقف"}
🔵 Gateway: ${report.gateway.health_ok ? "🟢 سليم" : "🔴 معطل"}
📋 ${report.errors.length > 0 ? `⚠️ مشاكل (${report.errors.length})` : "✅ لا توجد مشاكل"}
💡 توصيات: ${report.recommendations.join("، ") || "لا توجد"}
                `.trim());
              } catch (e) {
                setDiagResult(`❌ ${e}`);
              }
            }}
          >
            📤 تصدير تقرير تشخيص
          </button>

          <button
            className="btn btn-warning"
            onClick={async () => {
              setDoctorResult("⏳ جاري تشغيل doctor...");
              try {
                const r: any = await invoke("run_openclaw_doctor");
                setDoctorResult(r.success ? "✅ تم تشغيل doctor بنجاح" : `❌ فشل: ${r.stderr}`);
              } catch (e) {
                setDoctorResult(`❌ ${e}`);
              }
            }}
          >
            🩺 تشغيل Doctor
          </button>
        </div>

        {diagResult && (
          <div className="alert alert-info" style={{ marginTop: 8, whiteSpace: "pre-line" }}>
            {diagResult}
          </div>
        )}
        {doctorResult && (
          <div className="alert alert-info" style={{ marginTop: 8 }}>
            {doctorResult}
          </div>
        )}
      </div>

      <div className="section">
        <h3>🔐 Secure Storage</h3>
        <p className="muted">
          قريبًا: حفظ المفاتيح في Keychain/Windows Credential Manager
        </p>
      </div>

      <div className="section">
        <h3>ℹ️ About</h3>
        <p>OpenClaw Manager v0.1.0</p>
        <p className="muted">لوحة تحكم لإدارة OpenClaw Gateway على WSL</p>
        <button
          className="btn btn-sm"
          onClick={async () => {
            try {
              const r: any = await invoke("check_wsl_status");
              alert(`WSL: ${r.success ? "🟢 شغال" : "🔴 موقف"}\n${r.stdout || r.stderr}`);
            } catch (e) {
              alert(`خطأ: ${e}`);
            }
          }}
        >
          🐧 اختبار WSL
        </button>
      </div>
    </div>
  );
}

export default Settings;
