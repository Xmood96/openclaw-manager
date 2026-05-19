import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function Settings() {
  const [firebaseKey, setFirebaseKey] = useState("");
  const [projectId, setProjectId] = useState("");
  const [saved, setSaved] = useState(false);

  const saveSettings = async () => {
    // TODO: حفظ في secure storage
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
        <button
          className="btn"
          onClick={async () => {
            try {
              const report: any = await invoke("export_diagnostics");
              alert(`تم تصدير التقرير: ${JSON.stringify(report, null, 2)}`);
            } catch (e) {
              alert(`خطأ: ${e}`);
            }
          }}
        >
          📤 تصدير تقرير تشخيص
        </button>
      </div>
    </div>
  );
}

export default Settings;
