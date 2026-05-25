import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

function Settings() {
  const [deepseekKey, setDeepseekKey] = useState("");
  const [saveStatus, setSaveStatus] = useState<"idle" | "saving" | "saved" | "error">("idle");
  const [saveMsg, setSaveMsg] = useState("");
  const [loggedIn, setLoggedIn] = useState(false);
  const [hasKey, setHasKey] = useState(false);
  const [diagResult, setDiagResult] = useState<string | null>(null);
  const [doctorResult, setDoctorResult] = useState<string | null>(null);

  useEffect(() => {
    checkAuth();
    checkKey();
  }, []);

  const checkAuth = async () => {
    try {
      const session: any = await invoke("check_session");
      setLoggedIn(session.logged_in);
    } catch {
      setLoggedIn(false);
    }
  };

  const checkKey = async () => {
    try {
      const exists = await invoke<boolean>("agent_has_deepseek_key");
      setHasKey(exists);
      if (exists) {
        setSaveMsg("✅ مفتاح DeepSeek موجود");
      }
    } catch {
      // ignore
    }
  };

  const saveDeepseekKey = async () => {
    if (!deepseekKey.trim()) return;
    setSaveStatus("saving");
    setSaveMsg("");

    // 1. احفظ محليًا (دائمًا)
    try {
      await invoke("agent_set_deepseek_key", { key: deepseekKey.trim() });
    } catch (e) {
      setSaveStatus("error");
      setSaveMsg(`❌ فشل الحفظ المحلي: ${e}`);
      return;
    }

    // 2. احفظ في Firebase (إذا مسجل)
    if (loggedIn) {
      try {
        await invoke("save_setting", {
          key: "deepseek_api_key",
          value: deepseekKey.trim(),
        });
        setSaveStatus("saved");
        setSaveMsg("✅ حفظ في Firebase ✅ وحفظ محلي ✅");
      } catch {
        setSaveStatus("saved");
        setSaveMsg("✅ حفظ محليًا ✅ (Firebase غير متاح — سجّل دخول للمزامنة)");
      }
    } else {
      setSaveStatus("saved");
      setSaveMsg("✅ حفظ محليًا ✅ (سجّل دخول عشان تحفظ في Firebase)");
    }

    setDeepseekKey("");
    setHasKey(true);
    setTimeout(() => {
      setSaveStatus("idle");
      setSaveMsg("");
    }, 3000);
  };

  const loadFromFirebase = async () => {
    if (!loggedIn) {
      setSaveMsg("⚠️ سجّل دخول أولاً");
      return;
    }
    try {
      const key = await invoke<string>("load_setting", { key: "deepseek_api_key" });
      setDeepseekKey(key);
      setSaveMsg("📥 تم تحميل المفتاح من Firebase");
    } catch (e) {
      setSaveMsg(`⚠️ ما في مفتاح في Firebase: ${e}`);
    }
  };

  return (
    <div className="page">
      <div className="page-header">
        <h2>⚙️ الإعدادات</h2>
      </div>

      {/* 🔐 DeepSeek API Key */}
      <div className="section">
        <h3>🧠 DeepSeek API</h3>
        <p className="muted">
          هذا المفتاح يربط المساعد الذكي بـ DeepSeek API مباشرة.
          {loggedIn
            ? "🟢 حسـبـك متصل — يحفظ تلقائيًا في Firebase."
            : "🔴 غير مسجل — يحفظ محليًا فقط."}
        </p>

        <div className="form-group">
          <label>DeepSeek API Key</label>
          <div className="input-row">
            <input
              type="password"
              value={deepseekKey}
              onChange={(e) => setDeepseekKey(e.target.value)}
              placeholder={hasKey ? "🔑 مفتاح موجود — اكتب مفتاح جديد للتحديث" : "sk-..."}
              style={{ flex: 1 }}
            />
            <button
              className="btn btn-primary"
              onClick={saveDeepseekKey}
              disabled={!deepseekKey.trim() || saveStatus === "saving"}
            >
              {saveStatus === "saving" ? "🔄..." : "💾 حفظ"}
            </button>
          </div>
        </div>

        {saveMsg && (
          <div className={`alert ${saveStatus === "error" ? "alert-error" : "alert-info"}`} style={{ marginTop: 8 }}>
            {saveMsg}
          </div>
        )}

        <div className="action-buttons" style={{ marginTop: 8 }}>
          {loggedIn && (
            <button className="btn btn-sm" onClick={loadFromFirebase}>
              📥 تحميل من Firebase
            </button>
          )}
          {hasKey && (
            <span className="badge badge-success" style={{ marginLeft: 8 }}>
              🟢 المفتاح موجود
            </span>
          )}
        </div>
      </div>

      {/* 🔧 Diagnostics */}
      <div className="section">
        <h3>🩺 Diagnostics</h3>
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

      {/* ℹ️ About */}
      <div className="section">
        <h3>ℹ️ About</h3>
        <p>OpenClaw Manager v0.2.0</p>
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
