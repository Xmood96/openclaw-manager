import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";
import { Key, Cloud, CloudOff, Save, Stethoscope, FileDown, Activity, Info, ShieldCheck } from "lucide-react";

export default function SettingsPage() {
  const [deepseekKey, setDeepseekKey] = useState("");
  const [saveStatus, setSaveStatus] = useState<"idle" | "saving" | "saved" | "error">("idle");
  const [saveMsg, setSaveMsg] = useState("");
  const [loggedIn, setLoggedIn] = useState(false);
  const [hasKey, setHasKey] = useState(false);
  const [diagResult, setDiagResult] = useState<string | null>(null);
  const [doctorResult, setDoctorResult] = useState<string | null>(null);
  const [wslStatus, setWslStatus] = useState<string | null>(null);

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
    } catch {}
  };

  const saveDeepseekKey = async () => {
    if (!deepseekKey.trim()) return;
    setSaveStatus("saving");
    setSaveMsg("");
    try {
      await invoke("agent_set_deepseek_key", { key: deepseekKey.trim() });
    } catch (e) {
      setSaveStatus("error");
      setSaveMsg(`❌ فشل: ${e}`);
      return;
    }
    if (loggedIn) {
      try {
        await invoke("save_setting", { key: "deepseek_api_key", value: deepseekKey.trim() });
        setSaveStatus("saved");
        setSaveMsg("✅ حفظ سحابي 🌐 + محلي 💻");
      } catch {
        setSaveStatus("saved");
        setSaveMsg("✅ حفظ محلي (Firebase غير متاح)");
      }
    } else {
      setSaveStatus("saved");
      setSaveMsg("✅ حفظ محلي (سجل دخول للمزامنة السحابية)");
    }
    setDeepseekKey("");
    setHasKey(true);
    setTimeout(() => { setSaveStatus("idle"); setSaveMsg(""); }, 4000);
  };

  return (
    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.25 }}>
      <div className="flex items-center justify-between mb-5">
        <div>
          <h2 className="text-2xl font-bold text-primary">الإعدادات</h2>
          <p className="text-sm text-muted mt-0.5">إدارة المفاتيح والتشخيص</p>
        </div>
      </div>

      {/* DeepSeek API Key */}
      <motion.div
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        className="bg-surface border border-border rounded-2xl p-5 mb-4"
      >
        <div className="flex items-center gap-2 mb-3">
          <Key size={18} className="text-primary" />
          <h3 className="font-semibold">DeepSeek API Key</h3>
        </div>

        <p className="text-sm text-muted mb-4">
          اربط المساعد بـ DeepSeek API مباشرة.{" "}
          {loggedIn ? (
            <span className="flex items-center gap-1 text-success">
              <Cloud size={12} /> حسابك متصل — يحفظ تلقائياً
            </span>
          ) : (
            <span className="flex items-center gap-1 text-muted">
              <CloudOff size={12} /> حفظ محلي فقط
            </span>
          )}
        </p>

        <div className="flex gap-2">
          <input
            type="password"
            value={deepseekKey}
            onChange={(e) => setDeepseekKey(e.target.value)}
            placeholder={hasKey ? "🔑 مفتاح موجود — اكتب مفتاح جديد" : "sk-..."}
            className="flex-1 px-4 py-2.5 rounded-xl border border-border bg-bg text-sm focus:outline-none focus:border-primary-light focus:ring-2 focus:ring-primary-light/20"
            dir="ltr"
          />
          <button
            onClick={saveDeepseekKey}
            disabled={!deepseekKey.trim() || saveStatus === "saving"}
            className="flex items-center gap-1.5 px-4 py-2.5 rounded-xl bg-primary text-white hover:bg-primary-dark transition-colors disabled:opacity-50 text-sm font-medium"
          >
            {saveStatus === "saving" ? (
              <motion.div animate={{ rotate: 360 }} transition={{ repeat: Infinity, duration: 1 }}>
                <Activity size={14} />
              </motion.div>
            ) : (
              <Save size={14} />
            )}
            حفظ
          </button>
        </div>

        <AnimatedAlert message={saveMsg} type={saveStatus === "error" ? "error" : saveStatus === "saved" ? "success" : null} />
        {hasKey && <p className="mt-2 text-xs text-success flex items-center gap-1"><ShieldCheck size={12} /> المفتاح محفوظ وجاهز</p>}
      </motion.div>

      {/* Diagnostics */}
      <motion.div
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="bg-surface border border-border rounded-2xl p-5 mb-4"
      >
        <div className="flex items-center gap-2 mb-3">
          <Stethoscope size={18} className="text-primary" />
          <h3 className="font-semibold">التشخيص</h3>
        </div>
        <div className="flex flex-wrap gap-2">
          <button
            onClick={async () => {
              setDiagResult("⏳ جاري التصدير...");
              try {
                const report: any = await invoke("export_diagnostics");
                setDiagResult(`✅ تصدير: ${report.timestamp}\nWSL: ${report.wsl?.running ? "🟢" : "🔴"} | Gateway: ${report.gateway?.health_ok ? "🟢" : "🔴"} | مشاكل: ${report.errors?.length ?? 0}`);
              } catch (e) {
                setDiagResult(`❌ ${e}`);
              }
            }}
            className="flex items-center gap-1.5 px-3 py-2 rounded-xl text-sm border border-border hover:bg-bg transition-colors"
          >
            <FileDown size={14} />
            تصدير تقرير
          </button>
          <button
            onClick={async () => {
              setDoctorResult("⏳ جاري doctor...");
              try {
                const r: any = await invoke("run_openclaw_doctor");
                setDoctorResult(r.success ? "✅ تم doctor بنجاح" : `❌ ${r.stderr}`);
              } catch (e) {
                setDoctorResult(`❌ ${e}`);
              }
            }}
            className="flex items-center gap-1.5 px-3 py-2 rounded-xl text-sm bg-warning text-white hover:bg-amber-600 transition-colors"
          >
            <Stethoscope size={14} />
            Doctor --fix
          </button>
        </div>
        <AnimatedAlert message={diagResult} />
        <AnimatedAlert message={doctorResult} />
      </motion.div>

      {/* About */}
      <motion.div
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.15 }}
        className="bg-surface border border-border rounded-2xl p-5"
      >
        <div className="flex items-center gap-2 mb-3">
          <Info size={18} className="text-muted" />
          <h3 className="font-semibold">حول</h3>
        </div>
        <p className="text-sm">OpenClaw Manager v0.4.0</p>
        <p className="text-xs text-muted mt-1 mb-3">لوحة تحكم لإدارة OpenClaw على WSL</p>
        <button
          onClick={async () => {
            try {
              const r: any = await invoke("check_wsl_status");
              setWslStatus(`WSL: ${r.success ? "🟢 شغال" : "🔴 موقف"}\n${r.stdout || r.stderr}`);
            } catch (e) {
              setWslStatus(`❌ ${e}`);
            }
          }}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs border border-border hover:bg-bg transition-colors"
        >
          <Activity size={12} />
          اختبار WSL
        </button>
        <AnimatedAlert message={wslStatus} />
      </motion.div>
    </motion.div>
  );
}

function AnimatedAlert({ message, type }: { message: string | null; type?: "success" | "error" | null }) {
  if (!message) return null;
  return (
    <motion.div
      initial={{ opacity: 0, height: 0 }}
      animate={{ opacity: 1, height: "auto" }}
      className={`mt-3 p-3 rounded-xl text-sm ${
        type === "error"
          ? "bg-error/10 border border-error/20 text-error"
          : type === "success"
          ? "bg-success/10 border border-success/20 text-success"
          : "bg-bg border border-border text-muted"
      }`}
    >
      {message}
    </motion.div>
  );
}
