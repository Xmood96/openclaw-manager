import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

// ============================================================
// أنواع البيانات
// ============================================================

interface SystemStatus {
  overall_phase: string;
  wsl: CompStatus;
  ubuntu: CompStatus;
  nodejs: CompStatus;
  openclaw: CompStatus;
  config: CompStatus;
}

interface CompStatus {
  installed: boolean;
  version: string | null;
  details: string;
}

interface SetupStep {
  step_id: number;
  title: string;
  description: string;
  explanation: string;
  recommendation: string;
  status: "Pending" | "Current" | "Done" | "Error";
  action_label: string;
}

interface InstallGuide {
  steps: SetupStep[];
  current_step: number;
  total_steps: number;
  overall_progress: number;
}

interface ModelRec {
  id: string;
  name: string;
  provider: string;
  cost_tier: string;
  speed: string;
  quality: string;
  best_for: string[];
  requires_api_key: boolean;
  recommendation_level: string;
  explanation_ar: string;
}

// ============================================================
// دالة الحصول على عنوان المرحلة
// ============================================================

function getPhaseTitle(phase: string): { icon: string; title: string; subtitle: string } {
  switch (phase) {
    case "NoWSL":
      return { icon: "🪟", title: "مرحبًا بك في مساعدك الشخصي", subtitle: "لنبدأ بتجهيز البيئة" };
    case "WSLNoDistro":
      return { icon: "🐧", title: "جهز Linux", subtitle: "نحتاج توزيعة Ubuntu" };
    case "DistroNoOpenClaw":
      return { icon: "🧠", title: "نصب OpenClaw", subtitle: "القلب النابض للمساعد" };
    case "OpenClawNoConfig":
      return { icon: "⚙️", title: "الإعدادات الأولية", subtitle: "لنضبط المساعد" };
    case "OpenClawStopped":
      return { icon: "▶️", title: "شغّل المساعد", subtitle: "كل شيء جاهز للتشغيل" };
    case "OpenClawRunning":
      return { icon: "✅", title: "كل شيء تمام!", subtitle: "المساعد يعمل" };
    default:
      return { icon: "❓", title: "جاري الفحص...", subtitle: "" };
  }
}

// ============================================================
// المكون الرئيسي
// ============================================================

function SetupWizard({ onComplete }: { onComplete: () => void }) {
  const [status, setStatus] = useState<SystemStatus | null>(null);
  const [guide, setGuide] = useState<InstallGuide | null>(null);
  const [models, setModels] = useState<ModelRec[]>([]);
  const [loading, setLoading] = useState(true);
  const [actionOutput, setActionOutput] = useState("");
  const [runningAction, setRunningAction] = useState(false);
  const [expandedStep, setExpandedStep] = useState<number | null>(null);

  useEffect(() => {
    checkSystem();
  }, []);

  const checkSystem = async () => {
    setLoading(true);
    try {
      const s = await invoke<SystemStatus>("check_full_system");
      setStatus(s);
      const g = await invoke<InstallGuide>("get_setup_guide", {
        phase: s.overall_phase,
      });
      setGuide(g);
      const m = await invoke<ModelRec[]>("get_model_recommendations");
      setModels(m);
    } catch (e) {
      console.error("فحص فاشل:", e);
    } finally {
      setLoading(false);
    }
  };

  const runAction = async (label: string, command: string) => {
    setRunningAction(true);
    setActionOutput(`🔄 ${label}...`);
    try {
      const result = await invoke<string>("run_install_command", {
        command,
      });
      setActionOutput(result);
    } catch (e) {
      setActionOutput(`❌ ${e}`);
    }
    setRunningAction(false);
    // أعد الفحص
    setTimeout(checkSystem, 2000);
  };

  // ==========================================================
  // شاشة التحميل
  // ==========================================================
  if (loading) {
    return (
      <div className="page-loading">
        <div className="spinner" />
        <p>🔍 جاري فحص النظام...</p>
      </div>
    );
  }

  const phase = getPhaseTitle(status?.overall_phase || "");
  const isComplete = status?.overall_phase === "OpenClawRunning";

  // ==========================================================
  // شاشة "كل شيء تمام"
  // ==========================================================
  if (isComplete) {
    return (
      <div className="page">
        <div className="success-screen">
          <div className="success-icon">🎉</div>
          <h2>مساعدك الشخصي جاهز!</h2>
          <p className="success-sub">
            OpenClaw يعمل وكل القنوات جاهزة. تفضل إلى لوحة التحكم.
          </p>
          <div className="status-summary">
            <div className="summary-item ok">✅ WSL {status?.wsl.version ? `(${status.wsl.version})` : ""}</div>
            <div className="summary-item ok">✅ Ubuntu مثبت</div>
            <div className="summary-item ok">✅ OpenClaw {status?.openclaw.version ? `(${status.openclaw.version})` : ""}</div>
            <div className="summary-item ok">✅ الإعدادات موجودة</div>
          </div>
          <button className="btn btn-primary btn-lg" onClick={onComplete}>
            🚀 فتح لوحة التحكم
          </button>
        </div>
      </div>
    );
  }

  // ==========================================================
  // شاشة الإعداد
  // ==========================================================
  return (
    <div className="page">
      {/* الهيدر */}
      <div className="setup-header">
        <div className="phase-badge">{phase.icon}</div>
        <h2>{phase.title}</h2>
        <p className="phase-subtitle">{phase.subtitle}</p>
      </div>

      {/* شريط التقدم */}
      {guide && (
        <div className="progress-section">
          <div className="progress-bar">
            <div
              className="progress-fill"
              style={{ width: `${guide.overall_progress * 100}%` }}
            />
          </div>
          <span className="progress-label">
            {Math.round(guide.overall_progress * 100)}% — الخطوة {guide.current_step} من {guide.total_steps}
          </span>
        </div>
      )}

      {/* قائمة الخطوات */}
      {guide?.steps.map((step) => {
        const isActive = step.status === "Current";
        const isDone = step.status === "Done";
        const hasError = step.status === "Error";
        const isExpanded = expandedStep === step.step_id;

        return (
          <div
            key={step.step_id}
            className={`setup-step ${isActive ? "active" : ""} ${isDone ? "done" : ""} ${hasError ? "error" : ""}`}
            onClick={() => step.description && setExpandedStep(isExpanded ? null : step.step_id)}
          >
            {/* أيقونة الحالة */}
            <div className="step-indicator">
              {isDone ? "✅" : isActive ? "▶️" : hasError ? "❌" : "⏳"}
            </div>

            {/* المحتوى */}
            <div className="step-content">
              <div className="step-header">
                <h3>{step.title}</h3>
                {step.description && (
                  <p className="step-desc">{step.description}</p>
                )}
              </div>

              {/* التفاصيل الموسعة */}
              {isExpanded && (step.explanation || step.recommendation) && (
                <div className="step-details">
                  {step.explanation && (
                    <div className="explanation-box">
                      <span className="box-label">💡 شرح:</span>
                      <p>{step.explanation}</p>
                    </div>
                  )}
                  {step.recommendation && (
                    <div className="recommendation-box">
                      <span className="box-label">⭐ أنصح بـ:</span>
                      <p>{step.recommendation}</p>
                    </div>
                  )}
                </div>
              )}
            </div>

            {/* زر الإجراء */}
            {isActive && step.action_label && (
              <div className="step-actions">
                <button
                  className="btn btn-primary"
                  disabled={runningAction}
                  onClick={async (e) => {
                    e.stopPropagation();
                    const phase = status?.overall_phase;
                    let cmd = "";
                    switch (phase) {
                      case "NoWSL":
                        cmd = "wsl --install";
                        break;
                      case "WSLNoDistro":
                        cmd = "wsl --install -d Ubuntu-24.04";
                        break;
                      case "DistroNoOpenClaw":
                        // اختر npm تلقائيًا
                        cmd = "npm install -g openclaw";
                        break;
                      case "OpenClawNoConfig":
                        cmd = "openclaw onboard --non-interactive";
                        break;
                      case "OpenClawStopped":
                        cmd = "openclaw gateway start";
                        break;
                      default:
                        cmd = "openclaw doctor --fix --non-interactive";
                    }
                    await runAction(step.action_label, cmd);
                  }}
                >
                  {runningAction ? "🔄 جاري..." : step.action_label}
                </button>
              </div>
            )}
          </div>
        );
      })}

      {/* مخرجات الإجراء */}
      {actionOutput && (
        <div className="action-output">
          <pre>{actionOutput}</pre>
        </div>
      )}

      {/* توصيات الموديلات (في مرحلة OpenClawNoConfig) */}
      {status?.overall_phase === "OpenClawNoConfig" && models.length > 0 && (
        <div className="section model-recommendations">
          <h3>🧠 اختر موديل المساعد</h3>
          <p className="muted">
            اختر الموديل المناسب لاستخدامك. كل خيار موضح مع توصيتنا:
          </p>
          <div className="model-grid">
            {models.map((model) => {
              const levelColors: Record<string, string> = {
                recommended: "var(--primary)",
                good: "var(--secondary)",
                budget: "var(--warning)",
                premium: "#7c3aed",
              };
              return (
                <div
                  key={model.id}
                  className="model-card"
                  style={{ borderColor: levelColors[model.recommendation_level] || "var(--border)" }}
                >
                  <div className="model-badge" style={{ background: levelColors[model.recommendation_level] }}>
                    {model.recommendation_level === "recommended" ? "⭐ موصى به" :
                     model.recommendation_level === "budget" ? "💰 اقتصادي" :
                     model.recommendation_level === "premium" ? "💎 احترافي" : "👍 جيد"}
                  </div>
                  <h4>{model.name}</h4>
                  <div className="model-provider">{model.provider}</div>
                  <div className="model-tags">
                    <span className={`tag tag-${model.speed}`}>
                      {model.speed === "very_fast" ? "⚡ سريع جدًا" :
                       model.speed === "fast" ? "🚀 سريع" : "🐢 عادي"}
                    </span>
                    <span className={`tag tag-${model.cost_tier}`}>
                      {model.cost_tier === "free" ? "💰 مجاني" :
                       model.cost_tier === "low" ? "💰💰 رخيص" : "💰💰💰 متوسط"}
                    </span>
                    <span className={`tag tag-${model.quality}`}>
                      {model.quality === "excellent" ? "🏆 ممتاز" :
                       model.quality === "great" ? "👍 جيد جدًا" : "👌 جيد"}
                    </span>
                  </div>
                  <p className="model-explanation">{model.explanation_ar}</p>
                  <div className="model-uses">
                    {model.best_for.map((use) => (
                      <span key={use} className="use-tag">{use}</span>
                    ))}
                  </div>
                  {model.requires_api_key && (
                    <div className="model-api-key-note">🔑 يتطلب API key</div>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* أزرار إضافية */}
      <div className="setup-footer">
        <button className="btn" onClick={checkSystem}>
          🔄 إعادة فحص
        </button>
        {actionOutput && (
          <button className="btn" onClick={() => setActionOutput("")}>
            🧹 مسح المخرجات
          </button>
        )}
      </div>
    </div>
  );
}

export default SetupWizard;
