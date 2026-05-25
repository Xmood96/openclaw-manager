import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { motion, AnimatePresence } from "framer-motion";
import {
  Bot, User, Send, Plus, Stethoscope, Lightbulb, Loader2, AlertCircle, Wifi, WifiOff, Zap, Settings, Terminal,
} from "lucide-react";

interface AgentMessage {
  role: string; content: string; timestamp: string;
}
interface AgentSession {
  session_id: string; messages: AgentMessage[]; created_at: string; updated_at: string;
}
interface ProgressEvent {
  event_type: string; content: string; tool?: string; tool_args?: string; tool_result?: string;
}

const QUICK_ACTIONS = [
  { label: "تشخيص", icon: Stethoscope, msg: "شخّص النظام بالكامل" },
  { label: "ساعدني", icon: Lightbulb, msg: "وش تقدر تسوي؟" },
  { label: "القنوات", icon: Wifi, msg: "تحقق من القنوات واعرض حالتها" },
  { label: "Gateway", icon: Bot, msg: "تحقق من حالة Gateway وأعد تشغيله إذا كان واقف" },
  { label: "WSL", icon: Zap, msg: "تحقق من حالة WSL ونفذ فحص كامل" },
];

function MessageBubble({ msg, isUser }: { msg: AgentMessage; isUser: boolean }) {
  const renderContent = (content: string) => {
    return content.split("\n").map((line, i) => {
      const formatted = line
        .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
        .replace(/^### (.+)$/gm, "<strong>$1</strong>")
        .replace(/`([^`]+)`/g, "<code class='bg-primary/10 text-primary px-1 rounded text-xs font-mono'>$1</code>");
      return <span key={i} dangerouslySetInnerHTML={{ __html: formatted + (i < content.split("\n").length - 1 ? "<br/>" : "") }} />;
    });
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      className={`flex gap-2.5 max-w-[88%] ${isUser ? "self-end flex-row-reverse" : "self-start"}`}
    >
      <div className="flex-shrink-0 mt-0.5">
        {isUser ? (
          <div className="w-6 h-6 rounded-full bg-primary flex items-center justify-center"><User size={12} className="text-white" /></div>
        ) : (
          <div className="w-6 h-6 rounded-full bg-primary/10 flex items-center justify-center"><Bot size={12} className="text-primary" /></div>
        )}
      </div>
      <div className={`rounded-2xl px-3 py-2 text-sm leading-relaxed ${isUser ? "bg-primary text-white rounded-bl-md" : "bg-surface border border-border rounded-br-md"}`}>
        {msg.role === "system" ? <p className="text-xs text-muted italic">🛠️ معرفة النظام</p> : renderContent(msg.content)}
        {msg.timestamp && <div className={`text-[10px] mt-1 ${isUser ? "text-white/50" : "text-muted"}`}>{msg.timestamp.slice(11, 19)}</div>}
      </div>
    </motion.div>
  );
}

export default function AIAssistant() {
  const [session, setSession] = useState<AgentSession | null>(null);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [initLoading, setInitLoading] = useState(true);
  const [gatewayOk, setGatewayOk] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasAI, setHasAI] = useState(false);
  const [streamingContent, setStreamingContent] = useState("");
  const [toolStatus, setToolStatus] = useState<string | null>(null);
  const [toolResult, setToolResult] = useState<string | null>(null);

  const scrollRef = useRef<HTMLDivElement>(null);

  const scrollDown = () => { if (scrollRef.current) scrollRef.current.scrollTop = scrollRef.current.scrollHeight; };

  // Event listener for real-time progress
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    (async () => {
      unlisten = await listen<ProgressEvent>("agent-progress", (event) => {
        const p = event.payload;
        switch (p.event_type) {
          case "thinking":
            setStreamingContent("🧠 جاري التفكير...");
            setToolStatus(null);
            setToolResult(null);
            break;
          case "token":
            setStreamingContent(prev => prev === "🧠 جاري التفكير..." ? p.content : prev + p.content);
            break;
          case "tool_start":
            setToolStatus(`${p.content}`);
            setToolResult(null);
            break;
          case "tool_end":
            setToolResult(p.tool_result || p.content);
            break;
          case "done":
            // Streaming finished → reload session
            reloadSession();
            setStreamingContent("");
            setToolStatus(null);
            setToolResult(null);
            setLoading(false);
            break;
          case "error":
            setError(p.content);
            setLoading(false);
            setStreamingContent("");
            break;
        }
        scrollDown();
      });
    })();
    return () => { if (unlisten) unlisten(); };
  }, []);

  const reloadSession = useCallback(async () => {
    try {
      const saved = await invoke<string>("agent_load_session");
      setSession(JSON.parse(saved));
    } catch {}
  }, []);

  const startNewSession = useCallback(async () => {
    try {
      setInitLoading(true);
      const newSession = await invoke<string>("agent_new_chat");
      setSession(JSON.parse(newSession));
      setError(null);
    } catch (e) { setError(`فشل: ${e}`); }
    finally { setInitLoading(false); }
  }, []);

  useEffect(() => {
    (async () => {
      try {
        setHasAI(await invoke<boolean>("agent_has_deepseek_key"));
      } catch {}
      try {
        const saved = await invoke<string>("agent_load_session");
        if (saved && saved !== "{}") setSession(JSON.parse(saved));
        else await startNewSession();
      } catch { await startNewSession(); }
    })();
  }, [startNewSession]);

  useEffect(() => {
    const check = async () => {
      try { const s: any = await invoke("take_snapshot_cmd"); setGatewayOk(s?.gateway_ok ?? false); }
      catch { setGatewayOk(false); }
    };
    check(); const i = setInterval(check, 30000); return () => clearInterval(i);
  }, []);

  useEffect(scrollDown, [streamingContent, session?.messages]);

  const sendMessage = useCallback(async (text: string) => {
    if (!text.trim() || !session || loading) return;
    setInput("");
    setLoading(true);
    setError(null);
    setStreamingContent("🧠 جاري التفكير...");
    try {
      await invoke("agent_send_message", { sessionJson: JSON.stringify(session), message: text.trim() });
    } catch (e) {
      setError(`❌ ${e}`);
      setLoading(false);
      setStreamingContent("");
    }
  }, [session, loading]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); sendMessage(input); }
  };

  const visibleMessages = session?.messages.filter(m => m.role !== "system") ?? [];

  if (initLoading) {
    return <div className="flex flex-col items-center justify-center h-60 gap-4"><Loader2 size={32} className="animate-spin text-primary" /><p>جاري التحضير...</p></div>;
  }

  return (
    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="flex flex-col h-[calc(100vh-48px)]">
      {/* Header */}
      <div className="flex items-center justify-between p-3 bg-surface border border-border rounded-2xl mb-2 flex-shrink-0">
        <div className="flex items-center gap-3">
          <Bot size={24} className="text-primary" />
          <div>
            <h2 className="font-bold text-primary">مساعد الصيانة</h2>
            <div className="flex items-center gap-2 text-xs text-muted mt-0.5">
              <span className={hasAI ? "text-success flex items-center gap-1" : "text-warning flex items-center gap-1"}>
                {hasAI ? <><span className="w-1.5 h-1.5 rounded-full bg-success inline-block" /> DeepSeek</> : <><span className="w-1.5 h-1.5 rounded-full bg-warning inline-block" /> محلي</>}
              </span>
              <span>{gatewayOk ? <Wifi size={11} className="text-success" /> : <WifiOff size={11} className="text-error" />}</span>
              <span className="text-[10px]">جلسة {session?.session_id?.slice(0, 8)}</span>
            </div>
          </div>
        </div>
        <div className="flex items-center gap-1">
          {!hasAI && (
            <button onClick={() => { /* navigate to settings */ }} className="p-1.5 rounded-lg bg-warning/10 text-warning hover:bg-warning/20 text-xs flex items-center gap-1" title="أضف مفتاح DeepSeek">
              <Settings size={13} /> مفتاح
            </button>
          )}
          <button onClick={startNewSession} className="p-1.5 rounded-lg hover:bg-bg"><Plus size={16} /></button>
        </div>
      </div>

      {!hasAI && (
        <div className="flex items-center gap-2 text-xs bg-warning/5 border border-warning/10 text-warning rounded-xl px-3 py-2 mb-2" onClick={() => window.dispatchEvent(new CustomEvent("navigate", { detail: "settings" }))}>
          <Settings size={13} /> أضف مفتاح DeepSeek API من الإعدادات لتفعيل الذكاء الاصطناعي الكامل
        </div>
      )}

      {!gatewayOk && (
        <div className="flex items-center gap-2 text-xs bg-warning/10 border border-warning/20 text-warning rounded-xl px-3 py-2 mb-2">
          <AlertCircle size={13} /> Gateway واقف — الأوامر المباشرة لن تعمل
      {error && <div className="flex items-center gap-2 text-xs bg-error/10 border border-error/20 text-error rounded-xl px-3 py-2 mb-2"><AlertCircle size={13} /> {error}</div>}
        </div>
      )}

      {/* Messages area */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto px-1 py-2 flex flex-col gap-2">
        {visibleMessages.length === 0 && !loading && (
          <div className="text-center py-8">
            <Bot size={40} className="mx-auto text-muted opacity-30 mb-2" />
            <h3 className="font-bold mb-1">مساعد الصيانة الذكي</h3>
            <p className="text-xs text-muted mb-3">يشخّص، ينفذ أوامر، يقرأ ملفات — ويتابع معك خطوة بخطوة</p>
            <div className="grid grid-cols-2 sm:grid-cols-3 gap-2 max-w-md mx-auto">
              {QUICK_ACTIONS.map(a => (
                <button key={a.label} onClick={() => sendMessage(a.msg)}
                  className="flex items-center gap-1.5 p-2.5 rounded-xl border border-border bg-bg hover:bg-primary/5 hover:border-primary-light text-xs">
                  <a.icon size={14} className="text-primary-light" />{a.label}
                </button>
              ))}
            </div>
          </div>
        )}

        <AnimatePresence>
          {visibleMessages.map((msg, i) => <MessageBubble key={i} msg={msg} isUser={msg.role === "user"} />)}
        </AnimatePresence>

        {/* Tool execution display */}
        <AnimatePresence>
          {toolStatus && (
            <motion.div initial={{ opacity: 0, y: 4 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }}
              className="self-start flex items-start gap-2 bg-primary/5 border border-primary/10 rounded-2xl p-3 max-w-[88%]">
              <Terminal size={14} className="text-primary mt-0.5" />
              <div className="flex-1 min-w-0">
                <p className="text-xs font-medium text-primary">{toolStatus}</p>
                {toolResult && <pre className="mt-1.5 text-[11px] text-muted bg-bg rounded-xl p-2 max-h-[150px] overflow-y-auto font-mono">{toolResult}</pre>}
              </div>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Streaming content */}
        {streamingContent && !toolStatus && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }}
            className="self-start flex gap-2.5 max-w-[88%]">
            <div className="w-6 h-6 rounded-full bg-primary/10 flex items-center justify-center flex-shrink-0 mt-0.5"><Bot size={12} className="text-primary" /></div>
            <div className="rounded-2xl rounded-br-md bg-surface border border-border px-3 py-2 text-sm leading-relaxed">
              {streamingContent === "🧠 جاري التفكير..." ? (
                <div className="flex items-center gap-1"><Loader2 size={13} className="animate-spin text-primary" /><span className="text-muted text-xs">جاري التفكير...</span></div>
              ) : (
                streamingContent.split("\n").map((line, i) => (
                  <span key={i}>{line.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")}{i < streamingContent.split("\n").length - 1 ? <br/> : null}</span>
                ))
              )}
            </div>
          </motion.div>
        )}
      </div>

      {/* Input */}
      <div className="flex gap-2 pt-2 flex-shrink-0">
        <textarea value={input} onChange={e => setInput(e.target.value)} onKeyDown={handleKeyDown}
          placeholder={hasAI ? "اسألني أي شيء عن النظام..." : "اكتب 'شخّص' أو 'ساعدني'..."}
          rows={2} disabled={loading}
          className="flex-1 px-4 py-3 rounded-2xl border-2 border-border bg-surface text-sm resize-none focus:outline-none focus:border-primary-light focus:ring-2 focus:ring-primary-light/10"
          dir="rtl" />
        <button onClick={() => sendMessage(input)} disabled={!input.trim() || loading}
          className="w-12 h-12 rounded-full bg-primary text-white flex items-center justify-center hover:bg-primary-dark hover:scale-105 disabled:opacity-40 disabled:scale-100 flex-shrink-0 self-end transition-all">
          <Send size={18} />
        </button>
      </div>
      <p className="text-center text-[10px] text-muted py-1 flex-shrink-0">Enter ↵ إرسال · Shift+Enter ↵ سطر جديد</p>
    </motion.div>
  );
}
