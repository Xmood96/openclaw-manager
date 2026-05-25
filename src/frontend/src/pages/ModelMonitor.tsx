import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";
import {
  Brain,
  Cpu,
  Zap,
  Plug,
  Loader2,
  CheckCircle2,
  XCircle,
  Activity,
} from "lucide-react";

interface GatewayStatus {
  connected: boolean;
  version: string | null;
  error: string | null;
}

export default function ModelMonitor() {
  const [gwStatus, setGwStatus] = useState<GatewayStatus | null>(null);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const result = await invoke<GatewayStatus>("get_gateway_status");
        setGwStatus(result);
      } catch (e) {
        console.error("فشل:", e);
      }
    })();
  }, []);

  const testConnection = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const result: any = await invoke("connect_to_gateway");
      setTestResult(
        result.connected
          ? "✅ متصل بـ Gateway بنجاح"
          : `❌ فشل الاتصال: ${result.error}`
      );
    } catch (e) {
      setTestResult(`❌ خطأ: ${e}`);
    }
    setTesting(false);
  };

  return (
    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.25 }}>
      <div className="flex items-center justify-between mb-5">
        <div>
          <h2 className="text-2xl font-bold text-primary">الموديلات</h2>
          <p className="text-sm text-muted mt-0.5">مراقبة الموديلات والاستهلاك</p>
        </div>
      </div>

      {/* Gateway Status Card */}
      <motion.div
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        className="bg-surface border border-border rounded-2xl p-5 mb-4"
      >
        <div className="flex items-center gap-3 mb-4">
          <div
            className={`flex items-center justify-center w-12 h-12 rounded-2xl ${
              gwStatus?.connected ? "bg-success/10" : "bg-error/10"
            }`}
          >
            {gwStatus?.connected ? (
              <CheckCircle2 size={24} className="text-success" />
            ) : (
              <XCircle size={24} className="text-error" />
            )}
          </div>
          <div>
            <h3 className="font-bold text-lg">
              {gwStatus?.connected ? "Gateway متصل" : "Gateway غير متصل"}
            </h3>
            {gwStatus?.version && (
              <p className="text-sm text-muted">الإصدار: {gwStatus.version}</p>
            )}
          </div>
        </div>

        <div className="flex gap-2">
          <button
            onClick={testConnection}
            disabled={testing}
            className="flex items-center gap-1.5 px-4 py-2 rounded-xl text-sm bg-primary text-white hover:bg-primary-dark transition-colors disabled:opacity-50"
          >
            {testing ? (
              <Loader2 size={14} className="animate-spin" />
            ) : (
              <Plug size={14} />
            )}
            اختبار الاتصال
          </button>
        </div>

        {testResult && (
          <p className="mt-3 text-sm text-muted">{testResult}</p>
        )}
      </motion.div>

      {/* Models Info */}
      <motion.div
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="bg-surface border border-border rounded-2xl p-5 mb-4"
      >
        <div className="flex items-center gap-2 mb-4">
          <Brain size={18} className="text-primary" />
          <h3 className="font-semibold text-sm">الموديلات النشطة</h3>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <ModelBadge
            icon={Zap}
            name="DeepSeek V4 Flash"
            provider="DeepSeek"
            description="الموديل الافتراضي — سريع وذكي"
            default
          />
          <ModelBadge
            icon={Brain}
            name="DeepSeek V4 Pro"
            provider="DeepSeek"
            description="للتفكير العميق — جودة أعلى"
          />
          <ModelBadge
            icon={Cpu}
            name="Claude Haiku 4.5"
            provider="Anthropic"
            description="سريع واقتصادي"
          />
          <ModelBadge
            icon={Activity}
            name="Gemini 2.0 Flash"
            provider="Google"
            description="طبقة مجانية سخية"
          />
        </div>

        <p className="mt-4 text-xs text-muted text-center">
          تتبع الاستهلاك التفصيلي والتكاليف قادم في الإصدار القادم
        </p>
      </motion.div>
    </motion.div>
  );
}

function ModelBadge({
  icon: Icon,
  name,
  provider,
  description,
  default: isDefault,
}: {
  icon: React.ComponentType<{ size?: number }>;
  name: string;
  provider: string;
  description: string;
  default?: boolean;
}) {
  return (
    <div className="flex items-start gap-3 p-3 rounded-xl bg-bg border border-border">
      <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-primary/5 flex-shrink-0">
        <Icon size={18} className="text-primary-light" />
      </div>
      <div className="min-w-0">
        <div className="flex items-center gap-2">
          <h4 className="font-semibold text-sm">{name}</h4>
          {isDefault && (
            <span className="text-[10px] font-bold px-1.5 py-0.5 rounded-md bg-primary/10 text-primary">
              افتراضي
            </span>
          )}
        </div>
        <p className="text-xs text-muted">{provider}</p>
        <p className="text-xs text-muted mt-0.5">{description}</p>
      </div>
    </div>
  );
}
