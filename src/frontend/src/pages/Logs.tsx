import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";
import { ScrollText, RefreshCw, Loader2 } from "lucide-react";

export default function Logs() {
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
    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.25 }}>
      <div className="flex items-center justify-between mb-5">
        <div>
          <h2 className="text-2xl font-bold text-primary">سجلات النظام</h2>
          <p className="text-sm text-muted mt-0.5">مراقبة السجلات والأحداث</p>
        </div>
        <button
          onClick={fetchLogs}
          disabled={loading}
          className="flex items-center gap-1.5 px-3 py-2 rounded-xl text-sm bg-surface border border-border hover:bg-bg transition-colors disabled:opacity-50"
        >
          {loading ? (
            <Loader2 size={14} className="animate-spin" />
          ) : (
            <RefreshCw size={14} />
          )}
          تحديث
        </button>
      </div>

      <motion.div
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        className="bg-sidebar text-sidebar-text rounded-2xl overflow-hidden"
      >
        {loading && (
          <div className="flex items-center justify-center gap-2 py-8 text-sidebar-text/50">
            <Loader2 size={20} className="animate-spin" />
            جاري التحميل...
          </div>
        )}
        {!logs && !loading && (
          <div className="flex flex-col items-center justify-center py-16 text-sidebar-text/40 gap-2">
            <ScrollText size={40} />
            <span>اضغط "تحديث" لعرض السجلات</span>
          </div>
        )}
        {logs && (
          <pre
            className="log-viewer p-5 text-[12px] leading-relaxed max-h-[70vh] overflow-y-auto"
          >
            {logs}
          </pre>
        )}
      </motion.div>
    </motion.div>
  );
}
