import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

// ============================================================
// Types (تطابق Rust backend)
// ============================================================

interface AgentMessage {
  role: string; // "system" | "user" | "agent" | "tool"
  content: string;
  timestamp: string;
  tool_calls?: ToolCall[];
}

interface ToolCall {
  tool: string;
  args: any;
  result?: string;
}

interface AgentSession {
  session_id: string;
  messages: AgentMessage[];
  created_at: string;
  updated_at: string;
}

interface Diagnosis {
  severity: string;
  component: string;
  summary: string;
  details?: string | null;
  fix_available: boolean;
  fix_command?: string | null;
  fix_description?: string | null;
}

// ============================================================
// خيارات سريعة للمساعد
// ============================================================

const QUICK_ACTIONS = [
  { label: "🩺 تشخيص", msg: "شخّص النظام" },
  { label: "🗂️ هيكل المشروع", msg: "عرض ملفات المشروع" },
  { label: "🫀 ساعدني", msg: "وش تقدر تسوي؟" },
  { label: "📡 حالة القنوات", msg: "تحقق من القنوات" },
  { label: "🐧 WSL", msg: "تحقق من حالة WSL" },
  { label: "🔥 Gateway", msg: "حالة Gateway" },
];

// ============================================================
// مكون الفقاعة — يعرض رسالة مع Markdown بسيط
// ============================================================

function MessageBubble({ msg, isUser }: { msg: AgentMessage; isUser: boolean }) {
  const content = msg.content;

  // تقسيم النص إلى أجزاء: عادي + blocks
  const parts: { type: "text" | "code" | "codeblock"; content: string }[] = [];
  const lines = content.split("\n");
  let inCodeBlock = false;
  let codeBuffer: string[] = [];
  for (const line of lines) {
    if (line.startsWith("```")) {
      if (inCodeBlock) {
        parts.push({ type: "codeblock", content: codeBuffer.join("\n") });
        codeBuffer = [];
        inCodeBlock = false;
      } else {
        inCodeBlock = true;
      }
      continue;
    }

    if (inCodeBlock) {
      codeBuffer.push(line);
      continue;
    }

    // Inline code
    if (line.includes("`")) {
      const segments = line.split(/(`[^`]+`)/);
      for (const seg of segments) {
        if (seg.startsWith("`") && seg.endsWith("`")) {
          parts.push({ type: "code", content: seg.slice(1, -1) });
        } else {
          parts.push({ type: "text", content: seg });
        }
      }
      parts.push({ type: "text", content: "\n" });
    } else {
      parts.push({ type: "text", content: line + "\n" });
    }
  }

  if (inCodeBlock && codeBuffer.length > 0) {
    parts.push({ type: "codeblock", content: codeBuffer.join("\n") });
  }

  return (
    <div className={`chat-msg ${isUser ? "user" : msg.role === "system" ? "system" : "agent"}`}>
      <div className="chat-msg-avatar">
        {isUser ? "👤" : "🫀"}
      </div>
      <div className="chat-msg-content">
        {msg.role === "system" ? (
          <div className="chat-system-banner">🛠️ معرفة النظام محملة</div>
        ) : (
          parts.map((part, i) => {
            if (part.type === "codeblock") {
              return (
                <pre key={i} className="chat-codeblock">
                  <code>{part.content}</code>
                </pre>
              );
            }
            if (part.type === "code") {
              return (
                <code key={i} className="chat-inline-code">{part.content}</code>
              );
            }
            // Text — معالجة bold و headers
            const text = part.content
              .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
              .replace(/^### (.+)$/gm, "<h4>$1</h4>")
              .replace(/^## (.+)$/gm, "<h3>$1</h3>")
              .replace(/^- (.+)$/gm, "• $1");
            return (
              <span key={i} dangerouslySetInnerHTML={{ __html: text }} />
            );
          })
        )}
        {msg.timestamp && (
          <div className="chat-msg-time">
            {msg.timestamp.slice(11, 19)}
          </div>
        )}
      </div>
    </div>
  );
}

// ============================================================
// المكون الرئيسي — واجهة المحادثة
// ============================================================

function AIAssistant() {
  const [session, setSession] = useState<AgentSession | null>(null);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [initLoading, setInitLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [gatewayOk, setGatewayOk] = useState(false);
  const [showActions, setShowActions] = useState(true);

  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  // ==========================================================
  // بدء جلسة جديدة
  // ==========================================================
  const startNewSession = useCallback(async () => {
    try {
      setInitLoading(true);
      const newSession = await invoke<AgentSession>("agent_new_chat");
      setSession(newSession);
      setShowActions(true);
      setError(null);
    } catch (e) {
      setError(`فشل بدء الجلسة: ${e}`);
    } finally {
      setInitLoading(false);
    }
  }, []);

  // ==========================================================
  // تحميل الجلسة السابقة عند البداية
  // ==========================================================
  useEffect(() => {
    const init = async () => {
      try {
        // حاول تحميل آخر جلسة
        const savedJson = await invoke<string>("agent_load_session");
        const saved: AgentSession = JSON.parse(savedJson);
        // تحقق إنها حديثة (أقل من 24 ساعة)
        setSession(saved);
        setShowActions(false);
      } catch {
        // ما في جلسة سابقة — ابدأ جديدة
        await startNewSession();
      } finally {
        setInitLoading(false);
      }
    };
    init();
  }, [startNewSession]);

  // ==========================================================
  // تحقق سريع من حالة النظام
  // ==========================================================
  useEffect(() => {
    const checkGateway = async () => {
      try {
        const snap: any = await invoke("take_snapshot_cmd");
        setGatewayOk(snap?.gateway_ok ?? false);
      } catch {
        setGatewayOk(false);
      }
    };
    checkGateway();
    const interval = setInterval(checkGateway, 30000);
    return () => clearInterval(interval);
  }, []);

  // ==========================================================
  // Auto-scroll عند رسالة جديدة
  // ==========================================================
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [session?.messages]);

  // ==========================================================
  // إرسال رسالة
  // ==========================================================
  const sendMessage = useCallback(async (text: string) => {
    if (!text.trim() || !session || loading) return;

    const msg = text.trim();
    setInput("");
    setLoading(true);
    setShowActions(false);
    setError(null);

    try {
      const sessionJson = JSON.stringify(session);
      await invoke<string>("agent_send_message", {
        sessionJson,
        message: msg,
      });

      // أعد تحميل الجلسة (بعد إضافة الرسائل في الـ backend)
      const updatedJson = await invoke<string>("agent_load_session");
      const updated: AgentSession = JSON.parse(updatedJson);
      setSession(updated);
    } catch (e) {
      setError(`❌ ${e}`);
      // أضف رسالة المستخدم محليًا لو فشل
      setSession(prev => {
        if (!prev) return prev;
        return {
          ...prev,
          messages: [
            ...prev.messages,
            {
              role: "user",
              content: msg,
              timestamp: new Date().toISOString(),
            },
            {
              role: "agent",
              content: `❌ عذرًا، صار خطأ: ${e}\n\nجرّب مرة ثانية أو اكتب \`شخّص\` عشان أشوف وضع النظام.`,
              timestamp: new Date().toISOString(),
            },
          ],
          updated_at: new Date().toISOString(),
        };
      });
    } finally {
      setLoading(false);
    }
  }, [session, loading]);

  // ==========================================================
  // Keyboard shortcuts
  // ==========================================================
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage(input);
    }
  };

  // ==========================================================
  // تصفية الرسائل المعروضة (نخفي system message)
  // ==========================================================
  const visibleMessages = session?.messages.filter(
    (m) => m.role !== "system"
  ) ?? [];

  // ==========================================================
  // شاشة التحميل الأولي
  // ==========================================================
  if (initLoading) {
    return (
      <div className="page-loading">
        <div className="spinner" />
        <p>🫀 جاري تحضير المساعد...</p>
      </div>
    );
  }

  // ==========================================================
  // الواجهة الرئيسية
  // ==========================================================
  return (
    <div className="chat-page">
      {/* Header */}
      <div className="chat-header">
        <div className="chat-header-info">
          <span className="chat-avatar-large">🫀</span>
          <div>
            <h2>المساعد الذكي</h2>
            <span className="chat-subtitle">
              {gatewayOk ? "🟢 متصل" : "🔴 غير متصل"} · 
              {session?.session_id 
                ? `جلسة ${session.session_id.slice(0, 8)}` 
                : "غير مهيأ"}
            </span>
          </div>
        </div>
        <div className="chat-header-actions">
          <button
            className="btn btn-sm"
            onClick={startNewSession}
            title="جلسة جديدة"
          >
            🆕 جديد
          </button>
          <button
            className="btn btn-sm"
            onClick={async () => {
              try {
                const diagnoses = await invoke<Diagnosis[]>("agent_health_check");
                const issues = diagnoses.filter(d => d.severity !== "ok");
                const ok = diagnoses.filter(d => d.severity === "ok");
                alert(
                  `✅ سليم: ${ok.length}\n` +
                  `⚠️ مشاكل: ${issues.length}\n` +
                  issues.map(d => `  • ${d.component}: ${d.summary}`).join("\n")
                );
              } catch (e) {
                alert(`❌ ${e}`);
              }
            }}
            title="فحص سريع"
          >
            🩺 فحص
          </button>
          <button
            className="btn btn-sm"
            onClick={() => setShowActions(!showActions)}
            title="الإجراءات السريعة"
          >
            💡 إجراءات
          </button>
        </div>
      </div>

      {/* Gateway status bar */}
      {!gatewayOk && (
        <div className="chat-warning-bar">
          ⚠️ Gateway غير شغال. بعض الأوامر (مثل فحص القنوات) ما راح تشتغل.
          اكتب "شغّل Gateway" عشان تشغّله.
        </div>
      )}

      {/* Messages area */}
      <div className="chat-messages" ref={scrollRef}>
        {/* Quick Actions */}
        {showActions && !loading && (
          <div className="chat-quick-actions">
            <p className="chat-actions-title">💡 اختر إجراء سريع أو اكتب سؤالك:</p>
            <div className="chat-actions-grid">
              {QUICK_ACTIONS.map((action) => (
                <button
                  key={action.label}
                  className="chat-action-btn"
                  onClick={() => sendMessage(action.msg)}
                >
                  {action.label}
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Initial greeting */}
        {visibleMessages.length === 0 && !showActions && (
          <div className="chat-empty">
            <div className="chat-empty-icon">🫀</div>
            <h3>مرحباً! أنا قلب النظام</h3>
            <p className="muted">
              أقدر أشخّص المشاكل، أقرا الملفات، أنفّذ أوامر، وأساعدك في صيانة OpenClaw Manager.
              <br />
              اكتب <code>ساعدني</code> عشان تشوف كلشي أقدر أسويه.
            </p>
          </div>
        )}

        {/* Messages */}
        {visibleMessages.length > 0 && (
          <div className="chat-msg-list">
            {visibleMessages.map((msg, i) => (
              <MessageBubble
                key={i}
                msg={msg}
                isUser={msg.role === "user"}
              />
            ))}
          </div>
        )}

        {/* Loading indicator */}
        {loading && (
          <div className="chat-msg agent">
            <div className="chat-msg-avatar">🫀</div>
            <div className="chat-msg-content">
              <div className="chat-typing">
                <span className="typing-dot" />
                <span className="typing-dot" />
                <span className="typing-dot" />
              </div>
            </div>
          </div>
        )}

        {/* Error message */}
        {error && !loading && (
          <div className="chat-error">
            <span>❌ {error}</span>
            <button className="btn btn-sm" onClick={() => { setError(null); }}>
              تجاهل
            </button>
          </div>
        )}

        {/* End of messages spacer */}
        <div style={{ height: 8 }} />
      </div>

      {/* Input area */}
      <div className="chat-input-area">
        <textarea
          ref={inputRef}
          className="chat-input"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="اسألني عن النظام، أو اكتب 'شخّص'، 'ملف <مسار>'، 'نفّذ <أمر>'..."
          rows={2}
          disabled={loading}
        />
        <button
          className="chat-send-btn"
          onClick={() => sendMessage(input)}
          disabled={!input.trim() || loading}
          title="إرسال (Enter)"
        >
          {loading ? (
            <span className="spinner-sm" />
          ) : (
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="22" y1="2" x2="11" y2="13" />
              <polygon points="22 2 15 22 11 13 2 9 22 2" />
            </svg>
          )}
        </button>
      </div>

      {/* Keyboard hint */}
      <div className="chat-hint">
        <span>Enter ↵ للإرسال · Shift+Enter ↵ لسطر جديد · 🆕 لجلسة جديدة</span>
      </div>
    </div>
  );
}

export default AIAssistant;
