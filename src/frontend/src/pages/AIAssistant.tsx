import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion, AnimatePresence } from "framer-motion";
import {
  Bot,
  User,
  Send,
  Plus,
  Stethoscope,
  Lightbulb,
  Loader2,
  AlertCircle,
  Wifi,
  WifiOff,
  Zap,
} from "lucide-react";

interface AgentMessage {
  role: string;
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

const QUICK_ACTIONS = [
  { label: "تشخيص", icon: Stethoscope, msg: "شخّص النظام" },
  { label: "ساعدني", icon: Lightbulb, msg: "وش تقدر تسوي؟" },
  { label: "القنوات", icon: Wifi, msg: "تحقق من القنوات" },
  { label: "WSL", icon: Zap, msg: "تحقق من حالة WSL" },
  { label: "Gateway", icon: Bot, msg: "حالة Gateway" },
];

function MessageBubble({ msg, isUser }: { msg: AgentMessage; isUser: boolean }) {
  const content = msg.content;
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
      } else inCodeBlock = true;
      continue;
    }
    if (inCodeBlock) { codeBuffer.push(line); continue; }
    if (line.includes("`")) {
      const segments = line.split(/(`[^`]+`)/);
      for (const seg of segments) {
        if (seg.startsWith("`") && seg.endsWith("`"))
          parts.push({ type: "code", content: seg.slice(1, -1) });
        else parts.push({ type: "text", content: seg });
      }
      parts.push({ type: "text", content: "\n" });
    } else parts.push({ type: "text", content: line + "\n" });
  }
  if (inCodeBlock && codeBuffer.length > 0)
    parts.push({ type: "codeblock", content: codeBuffer.join("\n") });

  const render = (part: { type: string; content: string }, i: number) => {
    if (part.type === "codeblock")
      return <pre key={i} className="bg-sidebar text-sidebar-text p-3 rounded-xl text-xs font-mono overflow-x-auto my-2">{part.content}</pre>;
    if (part.type === "code")
      return <code key={i} className="bg-primary/5 text-primary px-1.5 py-0.5 rounded-md text-xs font-mono">{part.content}</code>;
    const text = part.content
      .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
      .replace(/^### (.+)$/gm, "<h4 class='font-bold text-sm mt-2 mb-1'>$1</h4>")
      .replace(/^## (.+)$/gm, "<h3 class='font-bold text-primary mt-3 mb-1'>$1</h3>")
      .replace(/^- (.+)$/gm, "• $1");
    return <span key={i} dangerouslySetInnerHTML={{ __html: text }} />;
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      className={`flex gap-2.5 max-w-[85%] ${isUser ? "self-end flex-row-reverse" : "self-start"}`}
    >
      <div className="flex-shrink-0 mt-1">
        {isUser ? (
          <div className="w-7 h-7 rounded-full bg-primary flex items-center justify-center"><User size={14} className="text-white" /></div>
        ) : (
          <div className="w-7 h-7 rounded-full bg-primary/10 flex items-center justify-center"><Bot size={14} className="text-primary" /></div>
        )}
      </div>
      <div className={`rounded-2xl px-3.5 py-2.5 text-sm leading-relaxed ${
        isUser ? "bg-primary text-white rounded-bl-md" : "bg-surface border border-border rounded-br-md"
      }`}>
        {msg.role === "system" ? (
          <p className="text-xs text-muted italic">🛠️ معرفة النظام محملة</p>
        ) : (
          parts.map(render)
        )}
        {msg.timestamp && (
          <div className={`text-[10px] mt-1.5 ${isUser ? "text-white/50" : "text-muted"}`}>
            {msg.timestamp.slice(11, 19)}
          </div>
        )}
      </div>
    </motion.div>
  );
}

export default function AIAssistant() {
  const [session, setSession] = useState<AgentSession | null>(null);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [initLoading, setInitLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [gatewayOk, setGatewayOk] = useState(false);
  const [showActions, setShowActions] = useState(true);
  const [hasAI, setHasAI] = useState(false);
  const [aiModel, setAiModel] = useState("محلي");

  const scrollRef = useRef<HTMLDivElement>(null);

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

  useEffect(() => {
    (async () => {
      try {
        try { setHasAI(await invoke<boolean>("agent_has_deepseek_key")); } catch {}
        const savedJson = await invoke<string>("agent_load_session");
        const saved: AgentSession = JSON.parse(savedJson);
        setSession(saved);
        setShowActions(false);
      } catch { await startNewSession(); }
    })();
  }, [startNewSession]);

  useEffect(() => {
    const check = async () => {
      try {
        const snap: any = await invoke("take_snapshot_cmd");
        setGatewayOk(snap?.gateway_ok ?? false);
      } catch { setGatewayOk(false); }
    };
    check();
    const interval = setInterval(check, 30000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    if (scrollRef.current) scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
  }, [session?.messages]);

  const sendMessage = useCallback(async (text: string) => {
    if (!text.trim() || !session || loading) return;
    const msg = text.trim();
    setInput("");
    setLoading(true);
    setShowActions(false);
    setError(null);
    try {
      await invoke("agent_send_message", { sessionJson: JSON.stringify(session), message: msg });
      const updatedJson = await invoke<string>("agent_load_session");
      setSession(JSON.parse(updatedJson));
    } catch (e) {
      setError(`❌ ${e}`);
      setSession((prev) => prev ? {
        ...prev,
        messages: [...prev.messages,
          { role: "user", content: msg, timestamp: new Date().toISOString() },
          { role: "agent", content: `❌ عذرًا: ${e}\n\nجرّب مرة ثانية أو اكتب "شخّص".`, timestamp: new Date().toISOString() },
        ],
        updated_at: new Date().toISOString(),
      } : prev);
    } finally { setLoading(false); }
  }, [session, loading]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); sendMessage(input); }
  };

  const visibleMessages = session?.messages.filter(m => m.role !== "system") ?? [];

  if (initLoading) {
    return (
      <div className="flex flex-col items-center justify-center h-60 gap-4">
        <Loader2 size={32} className="animate-spin text-primary" />
        <p className="text-muted">جاري تحضير المساعد...</p>
      </div>
    );
  }

  return (
    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="flex flex-col h-[calc(100vh-48px)]">
      {/* Header */}
      <div className="flex items-center justify-between p-3 bg-surface border border-border rounded-2xl mb-2 flex-shrink-0">
        <div className="flex items-center gap-3">
          <div className="text-3xl">🫀</div>
          <div>
            <h2 className="font-bold text-lg text-primary">المساعد الذكي</h2>
            <div className="flex items-center gap-2 text-xs text-muted mt-0.5">
              <span className={hasAI ? "text-success" : "text-warning"}>● {aiModel}</span>
              <span>{gatewayOk ? <Wifi size={12} className="inline text-success" /> : <WifiOff size={12} className="inline text-error" />} Gateway</span>
              <span>جلسة {session?.session_id?.slice(0, 8) ?? "—"}</span>
            </div>
          </div>
        </div>
        <div className="flex items-center gap-1.5">
          <button onClick={startNewSession} className="p-1.5 rounded-lg hover:bg-bg transition-colors" title="جلسة جديدة"><Plus size={16} /></button>
          <button onClick={() => setShowActions(!showActions)} className="p-1.5 rounded-lg hover:bg-bg transition-colors" title="الإجراءات"><Lightbulb size={16} /></button>
        </div>
      </div>

      {!hasAI && (
        <div className="text-xs text-warning px-3 py-1.5 bg-warning/5 rounded-xl mb-2 flex-shrink-0">
          💡 اذهب إلى الإعدادات وأضف مفتاح DeepSeek API لتفعيل الذكاء
        </div>
      )}

      {!gatewayOk && (
        <div className="flex items-center gap-2 text-xs bg-warning/10 border border-warning/20 text-warning rounded-xl px-3 py-2 mb-2 flex-shrink-0">
          <AlertCircle size={14} /> Gateway غير شغال — اكتب "شغّل Gateway"
        </div>
      )}

      {/* Messages */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto px-1 py-3 flex flex-col gap-3">
        {showActions && !loading && (
          <motion.div
            initial={{ opacity: 0, y: -8 }}
            animate={{ opacity: 1, y: 0 }}
            className="bg-surface border border-border rounded-2xl p-4 mb-2"
          >
            <p className="text-sm font-semibold mb-3">اختر إجراء أو اكتب سؤالك:</p>
            <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
              {QUICK_ACTIONS.map((action) => (
                <button
                  key={action.label}
                  onClick={() => sendMessage(action.msg)}
                  className="flex items-center gap-2 p-2.5 rounded-xl border border-border bg-bg hover:bg-primary/5 hover:border-primary-light transition-all text-sm"
                >
                  <action.icon size={16} className="text-primary-light" />
                  {action.label}
                </button>
              ))}
            </div>
          </motion.div>
        )}

        {visibleMessages.length === 0 && !showActions && (
          <div className="text-center py-10">
            <Bot size={48} className="mx-auto text-muted opacity-30 mb-3" />
            <h3 className="text-lg font-bold mb-2">مرحباً! أنا قلب النظام</h3>
            <p className="text-sm text-muted">
              أقدر أشخّص المشاكل، أقرا الملفات، وأنفّذ الأوامر.
              <br />
              اكتب <code className="bg-primary/5 text-primary px-1.5 py-0.5 rounded text-xs font-mono">ساعدني</code> عشان تشوف وش أقدر أسوي.
            </p>
          </div>
        )}

        <AnimatePresence>
          {visibleMessages.map((msg, i) => (
            <MessageBubble key={i} msg={msg} isUser={msg.role === "user"} />
          ))}
        </AnimatePresence>

        {loading && (
          <div className="flex gap-2.5 items-center self-start">
            <div className="w-7 h-7 rounded-full bg-primary/10 flex items-center justify-center"><Bot size={14} className="text-primary" /></div>
            <div className="flex gap-1 px-3 py-2">
              {[0, 1, 2].map((i) => (
                <span key={i} className="w-1.5 h-1.5 rounded-full bg-primary-light animate-typing" style={{ animationDelay: `${i * 0.2}s` }} />
              ))}
            </div>
          </div>
        )}

        {error && !loading && (
          <div className="flex items-center gap-2 p-3 rounded-xl bg-error/10 border border-error/20 text-error text-sm">
            <AlertCircle size={14} />
            <span className="flex-1">{error}</span>
            <button onClick={() => setError(null)} className="text-xs underline">تجاهل</button>
          </div>
        )}
      </div>

      {/* Input */}
      <div className="flex gap-2 pt-2 flex-shrink-0">
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="اسألني عن النظام..."
          rows={2}
          disabled={loading}
          className="flex-1 px-4 py-3 rounded-2xl border-2 border-border bg-surface text-sm resize-none focus:outline-none focus:border-primary-light focus:ring-2 focus:ring-primary-light/10 transition-all"
          dir="rtl"
        />
        <button
          onClick={() => sendMessage(input)}
          disabled={!input.trim() || loading}
          className="w-12 h-12 rounded-full bg-primary text-white flex items-center justify-center hover:bg-primary-dark transition-all hover:scale-105 disabled:opacity-40 disabled:scale-100 flex-shrink-0 self-end"
        >
          <Send size={18} />
        </button>
      </div>
      <p className="text-center text-[10px] text-muted py-1 flex-shrink-0">
        Enter ↵ للإرسال · Shift+Enter ↵ لسطر جديد
      </p>
    </motion.div>
  );
}
