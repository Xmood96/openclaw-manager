// Maintenance Agent — قلب النظام الذكي
// v0.2: AI agent مدمج يعرف الكود + يشخّص + يصلح + له ذاكرة
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri;
// ============================================================
// هياكل البيانات
// ============================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentMessage {
    pub role: String,       // "user" | "agent" | "system" | "tool"
    pub content: String,
    pub timestamp: String,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCall {
    pub tool: String,       // "read_file" | "run_command" | "search_code" | "health_check"
    pub args: serde_json::Value,
    pub result: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentSession {
    pub session_id: String,
    pub messages: Vec<AgentMessage>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentDiagnosis {
    pub severity: String,           // "ok" | "warning" | "error" | "critical"
    pub component: String,          // "wsl" | "gateway" | "config" | "auth" | "channels" | "general"
    pub summary: String,
    pub details: Option<String>,
    pub fix_available: bool,
    pub fix_command: Option<String>,
    pub fix_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectFile {
    pub path: String,
    pub description: String,
    pub category: String,   // "backend" | "frontend" | "config" | "docs" | "build"
}

// ============================================================
// معرفة المشروع المضمنة
// ============================================================

fn get_project_knowledge() -> Vec<ProjectFile> {
    vec![
        // Backend
        ProjectFile {
            path: "src/tauri-backend/src/main.rs".into(),
            description: "نقطة دخول التطبيق — يشغّل Tauri backend".into(),
            category: "backend".into(),
        },
        ProjectFile {
            path: "src/tauri-backend/src/lib.rs".into(),
            description: "تسجيل جميع أوامر Tauri والموديولات".into(),
            category: "backend".into(),
        },
        ProjectFile {
            path: "src/tauri-backend/src/wsl_bridge.rs".into(),
            description: "جسر WSL — تنفيذ أوامر في توزيعة Linux عبر wsl.exe".into(),
            category: "backend".into(),
        },
        ProjectFile {
            path: "src/tauri-backend/src/oc_client.rs".into(),
            description: "عميل WebSocket — اتصال دائم مع OpenClaw Gateway".into(),
            category: "backend".into(),
        },
        ProjectFile {
            path: "src/tauri-backend/src/firebase_auth.rs".into(),
            description: "مصادقة Firebase — تسجيل دخول، تسجيل، جلسات".into(),
            category: "backend".into(),
        },
        ProjectFile {
            path: "src/tauri-backend/src/device.rs".into(),
            description: "بصمة الجهاز — UUID دائم للجهاز".into(),
            category: "backend".into(),
        },
        ProjectFile {
            path: "src/tauri-backend/src/orchestrator.rs".into(),
            description: "منسّق الإصلاح — playbooks و health checks".into(),
            category: "backend".into(),
        },
        ProjectFile {
            path: "src/tauri-backend/src/diagnostics.rs".into(),
            description: "تقارير تشخيصية للتصدير".into(),
            category: "backend".into(),
        },
        ProjectFile {
            path: "src/tauri-backend/src/setup.rs".into(),
            description: "معالج الإعداد — فحص وتثبيت WSL + OpenClaw".into(),
            category: "backend".into(),
        },
        ProjectFile {
            path: "src/tauri-backend/src/speed.rs".into(),
            description: "Fast WSL runner — سكربتات عبر 9p".into(),
            category: "backend".into(),
        },
        ProjectFile {
            path: "src/tauri-backend/src/app_state.rs".into(),
            description: "تخزين محلي في %APPDATA%/openclaw-manager".into(),
            category: "backend".into(),
        },
        ProjectFile {
            path: "src/tauri-backend/src/maintenance_agent.rs".into(),
            description: "القلب الذكي — هذا الملف! يشخّص ويصلح ويتذكر".into(),
            category: "backend".into(),
        },
        // Config
        ProjectFile {
            path: "src/tauri-backend/Cargo.toml".into(),
            description: "تبعيات Rust وإعدادات الحزمة".into(),
            category: "config".into(),
        },
        ProjectFile {
            path: "src/tauri-backend/tauri.conf.json".into(),
            description: "إعدادات Tauri — اسم التطبيق، الصلاحيات، النوافذ".into(),
            category: "config".into(),
        },
        // Frontend
        ProjectFile {
            path: "src/frontend/src/App.tsx".into(),
            description: "المكوّن الرئيسي للواجهة — توجيه الصفحات".into(),
            category: "frontend".into(),
        },
        ProjectFile {
            path: "src/frontend/src/firebase-config.ts".into(),
            description: "إعدادات Firebase للواجهة".into(),
            category: "frontend".into(),
        },
        // Build
        ProjectFile {
            path: "src/installer/build_windows.bat".into(),
            description: "سكربت البناء على ويندوز".into(),
            category: "build".into(),
        },
    ]
}

/// بناء system prompt للمساعد
fn build_system_prompt() -> String {
    let knowledge = get_project_knowledge();
    let mut files_list = String::new();
    for f in &knowledge {
        let cat_emoji = match f.category.as_str() {
            "backend" => "⚙️",
            "frontend" => "🎨",
            "config" => "🔧",
            "docs" => "📚",
            "build" => "🏗️",
            _ => "📄",
        };
        files_list.push_str(&format!("- {} `{}` — {}\n", cat_emoji, f.path, f.description));
    }

    format!(
        r#"أنت قلب نظام OpenClaw Manager — مساعد صيانة ومراقبة ذكي.

## هويتك
- اسمك: قلب النظام (System Heart)
- دورك: تشخيص المشاكل، اقتراح الحلول، تنفيذ الإصلاحات، مراقبة صحة النظام
- أسلوبك: تقني، مباشر، عملي. بالعربية مع مصطلحات تقنية بالإنجليزية.

## قدراتك
1. **قراءة الملفات** — تقدر تقرأ أي ملف في المشروع
2. **تنفيذ أوامر** — تقدر تشغّل أوامر WSL وترجع نتيجتها
3. **تشخيص المشاكل** — تحلل system status وتحدد المشكلة بدقة
4. **اقتراح حلول** — تعطي أوامر جاهزة للتنفيذ
5. **ذاكرة** — تتذكر المحادثات السابقة

## هيكل المشروع
يقوم OpenClaw Manager على:
- **Tauri v2** — إطار لبناء تطبيقات سطح المكتب (Rust backend + React frontend)
- **WSL Bridge** — جسر بين Windows وتوزيعة Linux (اكتشاف تلقائي للتوزيعة)
- **OpenClaw Gateway** — خادم WebSocket للمساعد الشخصي
- **Firebase Auth** — مصادقة المستخدمين
- **Device Registry** — ربط الجهاز بالحساب

## آلية التشخيص
عند فحص النظام (أمر `شخّص`)، أتبع الخطوات التالية:
1. فحص WSL — هل يعمل؟
2. فحص Node.js — هل مثبت في WSL؟
3. فحص OpenClaw binary — هل `openclaw` موجود في PATH؟
4. فحص الإعدادات — هل `~/.openclaw/openclaw.json` موجود؟
5. فحص Gateway — هل `openclaw health --json` يعود بـ ok=true؟
6. فحص القنوات — هل كل القنوات متصلة؟
7. فحص المصادقة — هل في Firebase session نشطة؟

ملاحظة: الـ PATH في WSL يشمل `$HOME/.npm-global/bin` و `$HOME/.local/bin` و `/usr/local/bin`.
ملف الإعدادات يُبحث عنه في: `openclaw.json` ← `config.yml` ← `clawd.json`.

## ملفات المشروع
{}

## آلية التواصل
عندما يطلب منك المستخدم شيء:
1. فكّر في المشكلة
2. اقرأ الملفات ذات العلاقة إذا لزم
3. شغّل أوامر تشخيصية إذا لزم
4. أعطِ إجابة واضحة مع الأوامر الجاهزة

## قنوات التواصل
المستخدم يتحدث معك عبر واجهة المحادثة في التطبيق.
"#,
        files_list
    )
}

// ============================================================
// أوامر Tauri
// ============================================================

/// بدء محادثة جديدة مع الـ agent
#[tauri::command]
pub fn agent_new_chat() -> AgentSession {
    let session_id = uuid::Uuid::new_v4().to_string();
    let now = chrono_now();

    let system_msg = AgentMessage {
        role: "system".into(),
        content: build_system_prompt(),
        timestamp: now.clone(),
        tool_calls: None,
    };

    AgentSession {
        session_id,
        messages: vec![system_msg],
        created_at: now.clone(),
        updated_at: now,
    }
}

/// إرسال رسالة للمساعد — يشغّل التشخيص ويرد
#[tauri::command]
pub async fn agent_send_message(session_json: String, message: String) -> Result<String, String> {
    let mut session: AgentSession = serde_json::from_str(&session_json)
        .map_err(|e| format!("جلسة غير صالحة: {}", e))?;

    let now = chrono_now();

    // أضف رسالة المستخدم
    session.messages.push(AgentMessage {
        role: "user".into(),
        content: message.clone(),
        timestamp: now.clone(),
        tool_calls: None,
    });

    // حلل الرسالة وشوف إذا فيها طلب tool
    let response = process_agent_turn(&message, &mut session).await;

    session.messages.push(AgentMessage {
        role: "agent".into(),
        content: response.clone(),
        timestamp: chrono_now(),
        tool_calls: None,
    });

    session.updated_at = chrono_now();

    // احفظ الجلسة
    let session_data = serde_json::to_string(&session).unwrap_or_default();
    crate::app_state::save_to_file("agent-session.json", &session_data);

    Ok(response)
}

/// فحص صحة النظام — agent يشخّص تلقائيًا
#[tauri::command]
pub async fn agent_health_check() -> Vec<AgentDiagnosis> {
    let distro = crate::wsl_bridge::get_distro_name();
    let mut diagnoses = Vec::new();

    // 1. فحص WSL
    let wsl_check = crate::wsl_bridge::exec_wsl("echo OK");
    if wsl_check.success {
        diagnoses.push(AgentDiagnosis {
            severity: "ok".into(),
            component: "wsl".into(),
            summary: format!("WSL يعمل — التوزيعة: {}", distro),
            details: None,
            fix_available: false,
            fix_command: None,
            fix_description: None,
        });
    } else {
        diagnoses.push(AgentDiagnosis {
            severity: "critical".into(),
            component: "wsl".into(),
            summary: "WSL لا يستجيب".into(),
            details: Some(wsl_check.stderr),
            fix_available: true,
            fix_command: Some("wsl --shutdown".into()),
            fix_description: Some("أعد تشغيل WSL بالكامل — شغّل هذا الأمر في PowerShell".into()),
        });
    }

    // 2. فحص وجود Node.js (المتطلب الأساسي)
    if wsl_check.success {
        let node_check = crate::wsl_bridge::exec_wsl(
            "OC_HOME=$(getent passwd $(id -un) 2>/dev/null | cut -d: -f6); [ -z \"$OC_HOME\" ] && OC_HOME=$(echo ~); export PATH=\"$OC_HOME/.npm-global/bin:$OC_HOME/.local/bin:/usr/local/bin:$PATH\"; node --version 2>/dev/null || echo 'NOT_FOUND'"
        );
        let node_ok = node_check.success && !node_check.stdout.contains("NOT_FOUND");
        if node_ok {
            let node_ver = node_check.stdout.trim().to_string();
            diagnoses.push(AgentDiagnosis {
                severity: "ok".into(),
                component: "nodejs".into(),
                summary: format!("Node.js متاح ✅ — {}", node_ver),
                details: None,
                fix_available: false,
                fix_command: None,
                fix_description: None,
            });
        } else {
            diagnoses.push(AgentDiagnosis {
                severity: "error".into(),
                component: "nodejs".into(),
                summary: "Node.js غير مثبت في WSL".into(),
                details: Some("OpenClaw يحتاج Node.js لتشغيل Gateway".into()),
                fix_available: true,
                fix_command: Some("curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo bash - && sudo apt install -y nodejs".into()),
                fix_description: Some("ثبّت Node.js عبر مدير الحزم".into()),
            });
        }
    }

    // 3. فحص وجود OpenClaw الثنائي
    if wsl_check.success {
        let oc_check = crate::wsl_bridge::exec_wsl(
            "OC_HOME=$(getent passwd $(id -un) 2>/dev/null | cut -d: -f6); [ -z \"$OC_HOME\" ] && OC_HOME=$(echo ~); export PATH=\"$OC_HOME/.npm-global/bin:$OC_HOME/.local/bin:/usr/local/bin:$PATH\"; which openclaw 2>/dev/null || echo 'NOT_FOUND'"
        );
        let oc_installed = oc_check.success && !oc_check.stdout.contains("NOT_FOUND");

        if oc_installed {
            let ver_check = crate::wsl_bridge::exec_wsl(
                "OC_HOME=$(getent passwd $(id -un) 2>/dev/null | cut -d: -f6); [ -z \"$OC_HOME\" ] && OC_HOME=$(echo ~); export PATH=\"$OC_HOME/.npm-global/bin:$OC_HOME/.local/bin:/usr/local/bin:$PATH\"; openclaw --version 2>/dev/null || echo '?'"
            );
            let oc_ver = ver_check.stdout.trim().to_string();
            diagnoses.push(AgentDiagnosis {
                severity: "ok".into(),
                component: "openclaw".into(),
                summary: format!("OpenClaw مثبت ✅ — {}", oc_ver),
                details: None,
                fix_available: false,
                fix_command: None,
                fix_description: None,
            });

            // 3a. فحص ملف الإعدادات — نستخدم ~ بدل $HOME عشان WSL
            let config_check = crate::wsl_bridge::exec_wsl(
                "OC_HOME=$(getent passwd $(id -un) 2>/dev/null | cut -d: -f6); [ -z \"$OC_HOME\" ] && OC_HOME=$(echo ~); for f in \"$OC_HOME/.openclaw/openclaw.json\" \"$OC_HOME/.openclaw/config.yml\" \"$OC_HOME/.openclaw/clawd.json\"; do test -f \"$f\" && echo \"EXISTS:$f\" && exit 0; done; echo 'NOT_FOUND'"
            );
            let config_exists = config_check.success && config_check.stdout.contains("EXISTS");

            if config_exists {
                let config_path = config_check.stdout.trim().strip_prefix("EXISTS:").unwrap_or("?").to_string();
                diagnoses.push(AgentDiagnosis {
                    severity: "ok".into(),
                    component: "config".into(),
                    summary: format!("الإعدادات موجودة ✅ — {}", config_path),
                    details: None,
                    fix_available: false,
                    fix_command: None,
                    fix_description: None,
                });
            } else {
                diagnoses.push(AgentDiagnosis {
                    severity: "warning".into(),
                    component: "config".into(),
                    summary: "لا توجد إعدادات OpenClaw".into(),
                    details: Some("مافيه ملف ~/.openclaw/openclaw.json — يحتاج إعداد".into()),
                    fix_available: true,
                    fix_command: Some("openclaw onboard".into()),
                    fix_description: Some("شغّل معالج الإعداد لتهيئة OpenClaw".into()),
                });
            }

            // 3b. فحص Gateway
            let health = crate::wsl_bridge::exec_wsl(
                "OC_HOME=$(getent passwd $(id -un) 2>/dev/null | cut -d: -f6); [ -z \"$OC_HOME\" ] && OC_HOME=$(echo ~); export PATH=\"$OC_HOME/.npm-global/bin:$OC_HOME/.local/bin:/usr/local/bin:$PATH\"; timeout 5 openclaw health --json 2>/dev/null || echo '{}'"
            );

            if let Ok(h) = serde_json::from_str::<Value>(&health.stdout) {
                let ok = h.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                if ok {
                    diagnoses.push(AgentDiagnosis {
                        severity: "ok".into(),
                        component: "gateway".into(),
                        summary: "Gateway شغال ✅".into(),
                        details: None,
                        fix_available: false,
                        fix_command: None,
                        fix_description: None,
                    });

                    // فحص القنوات
                    if let Some(channels) = h.get("channels").and_then(|c| c.as_object()) {
                        let disconnected: Vec<_> = channels.iter()
                            .filter(|(_, ch)| ch.get("connected").and_then(|v| v.as_bool()).unwrap_or(true) == false)
                            .collect();

                        for (name, _) in &disconnected {
                            diagnoses.push(AgentDiagnosis {
                                severity: "warning".into(),
                                component: "channels".into(),
                                summary: format!("قناة {} غير متصلة ⚠️", name),
                                details: None,
                                fix_available: true,
                                fix_command: Some(format!("openclaw channels login --{}", name)),
                                fix_description: Some(format!("أعد ربط قناة {}", name)),
                            });
                        }
                    }
                } else {
                    diagnoses.push(AgentDiagnosis {
                        severity: "warning".into(),
                        component: "gateway".into(),
                        summary: "Gateway واقف".into(),
                        details: Some(health.stdout),
                        fix_available: true,
                        fix_command: Some("openclaw gateway start".into()),
                        fix_description: Some("شغّل Gateway".into()),
                    });
                }
            }
        } else {
            // OpenClaw غير مثبت
            diagnoses.push(AgentDiagnosis {
                severity: "error".into(),
                component: "openclaw".into(),
                summary: "OpenClaw غير مثبت في WSL ❌".into(),
                details: Some(format!(
                    "بحثت في PATH: $HOME/.npm-global/bin:$HOME/.local/bin:/usr/local/bin:$PATH\nstdout: {}\nstderr: {}",
                    oc_check.stdout.trim(), oc_check.stderr.trim()
                )),
                fix_available: true,
                fix_command: Some("npm install -g openclaw".into()),
                fix_description: Some("ثبّت OpenClaw عبر npm".into()),
            });

            diagnoses.push(AgentDiagnosis {
                severity: "warning".into(),
                component: "gateway".into(),
                summary: "Gateway غير متاح — OpenClaw غير مثبت".into(),
                details: None,
                fix_available: false,
                fix_command: None,
                fix_description: None,
            });
        }
    }

    // 3. فحص المصادقة
    let session = crate::app_state::read_from_file("session.json").unwrap_or_default();
    if !session.is_empty() {
        if let Ok(s) = serde_json::from_str::<Value>(&session) {
            let has_token = s.get("idToken").and_then(|v| v.as_str()).map(|t| !t.is_empty()).unwrap_or(false);
            if has_token {
                diagnoses.push(AgentDiagnosis {
                    severity: "ok".into(),
                    component: "auth".into(),
                    summary: "الجلسة نشطة ✅".into(),
                    details: None,
                    fix_available: false,
                    fix_command: None,
                    fix_description: None,
                });
            } else {
                diagnoses.push(AgentDiagnosis {
                    severity: "warning".into(),
                    component: "auth".into(),
                    summary: "تحتاج تسجيل دخول".into(),
                    details: None,
                    fix_available: true,
                    fix_command: None,
                    fix_description: Some("سجّل دخولك من صفحة المصادقة".into()),
                });
            }
        }
    } else {
        diagnoses.push(AgentDiagnosis {
            severity: "warning".into(),
            component: "auth".into(),
            summary: "لم تسجل دخول بعد".into(),
            details: None,
            fix_available: true,
            fix_command: None,
            fix_description: Some("أنشئ حساب أو سجّل دخول من صفحة المصادقة".into()),
        });
    }

    diagnoses
}

/// حفظ الجلسة الحالية
#[tauri::command]
pub fn agent_save_session(session_json: String) -> Result<String, String> {
    crate::app_state::save_to_file("agent-session.json", &session_json);
    Ok("تم حفظ الجلسة".into())
}

/// تحميل آخر جلسة
#[tauri::command]
pub fn agent_load_session() -> Result<String, String> {
    crate::app_state::read_from_file("agent-session.json")
        .map_err(|e| format!("لا توجد جلسة سابقة: {}", e))
}

/// قراءة ملف من المشروع (يحتاج مسار نسبي)
#[tauri::command]
pub fn agent_read_file(relative_path: String) -> Result<String, String> {
    // أمان: منع path traversal
    let path = relative_path
        .replace("..", "")
        .replace('~', "")
        .replace('$', "");

    // ابحث عن جذر المشروع
    let mut root = std::env::current_dir().unwrap_or_default();
    // في Tauri dev mode، current_dir بيكون جذر المشروع
    // لو لا، نطلع فوق لين نلقى Cargo.toml
    if !root.join("Cargo.toml").exists() && root.join("src").join("tauri-backend").exists() {
        root = root.join("src").join("tauri-backend");
    }
    for _ in 0..5 {
        if root.join("Cargo.toml").exists() || root.join("src/tauri-backend/Cargo.toml").exists() {
            break;
        }
        if let Some(parent) = root.parent() {
            root = parent.to_path_buf();
        } else {
            break;
        }
    }

    // لو كنا في tauri-backend، نرجع للمشروع
    if root.join("src").join("tauri-backend").exists() {
        // root هو جذر المشروع
    } else if root.join("Cargo.toml").exists() && root.to_string_lossy().contains("tauri-backend") {
        root = root.parent().map(|p| p.parent().map(|p| p.to_path_buf()).unwrap_or(p.to_path_buf())).unwrap_or(root);
    }

    let full_path = root.join(&path);

    std::fs::read_to_string(&full_path)
        .map_err(|e| format!("فشل قراءة {}: {}", full_path.display(), e))
}

/// تنفيذ أمر WSL وإرجاع النتيجة
#[tauri::command]
pub fn agent_run_command(command: String) -> String {
    let result = crate::wsl_bridge::exec_wsl(&command);
    if result.success {
        format!("✅\n{}", result.stdout)
    } else {
        format!("❌ (exit {})\nstdout:\n{}\nstderr:\n{}",
            result.exit_code, result.stdout, result.stderr)
    }
}

// ============================================================
// المعالج الأساسي
// ============================================================

async fn process_agent_turn(user_message: &str, session: &mut AgentSession) -> String {
    let lower = user_message.to_lowercase();

    // — الكشف عن نية المستخدم —
    if lower.contains("شخّص") || lower.contains("تشخيص") || lower.contains("health") || lower.contains("فحص") {
        return run_diagnosis().await;
    }

    if lower.contains("ملف") || lower.contains("read") || lower.contains("اقرأ") || lower.contains("شفرة") {
        // استخرج مسار الملف من الرسالة (بسيط)
        for line in user_message.lines() {
            let trimmed = line.trim();
            if trimmed.ends_with(".rs") || trimmed.ends_with(".tsx") || trimmed.ends_with(".json") || trimmed.ends_with(".toml") {
                match agent_read_file(trimmed.to_string()) {
                    Ok(content) => {
                        return format!("📄 **{}**\n\n```\n{}\n```",
                            trimmed,
                            if content.len() > 3000 {
                                format!("{}...\n(الملف كبير — أول 3000 حرف)", &content[..3000])
                            } else {
                                content
                            }
                        );
                    }
                    Err(e) => return format!("❌ {}\n\nالمسار يجب أن يكون نسبيًا من جذر المشروع.\nمثال: `src/tauri-backend/src/setup.rs`", e),
                }
            }
        }
        return "أي ملف تبيني أقرأ؟ اكتب المسار النسبي.\nمثال: `src/tauri-backend/src/lib.rs`".into();
    }

    if lower.contains("نفّذ") || lower.contains("شغّل") || lower.contains("run") || lower.contains("أمر") {
        // استخرج الأمر
        let cmd = user_message
            .lines()
            .find(|l| l.contains("openclaw") || l.contains("wsl") || l.contains("npm") || l.contains("node"))
            .map(|l| l.trim().to_string());

        if let Some(cmd) = cmd {
            let result = crate::wsl_bridge::exec_wsl(&cmd);
            return if result.success {
                format!("✅ نجح\n```\n{}\n```", result.stdout)
            } else {
                format!("❌ فشل (exit {})\n```\n{}\n```",
                    result.exit_code,
                    if result.stderr.is_empty() { result.stdout } else { result.stderr })
            };
        }
        return "وش تبيني أشغّل؟ اكتب الأمر كامل.\nمثال: `openclaw gateway status`".into();
    }

    if lower.contains("مشروع") || lower.contains("ملفات") || lower.contains("codebase") || lower.contains("هيكل") {
        let knowledge = get_project_knowledge();
        let mut out = String::from("## 🗂️ هيكل المشروع\n\n");
        let mut current_cat = String::new();
        for f in &knowledge {
            if f.category != current_cat {
                current_cat = f.category.clone();
                let cat_name = match current_cat.as_str() {
                    "backend" => "⚙️ Backend (Rust)",
                    "frontend" => "🎨 Frontend (React)",
                    "config" => "🔧 Configuration",
                    "docs" => "📚 Documentation",
                    "build" => "🏗️ Build",
                    _ => "📄 Other",
                };
                out.push_str(&format!("\n### {}\n", cat_name));
            }
            out.push_str(&format!("- `{}` — {}\n", f.path, f.description));
        }
        return out;
    }

    if lower.contains("ساعد") || lower.contains("مساعدة") || lower.contains("help") || lower.contains("وش تقدر") {
        return format!(
            "## 🫀 قلب النظام — الأوامر المتاحة\n\n\
            **قُل لي…**\n\n\
            🩺 `شخّص` — فحص صحة النظام كامل\n\
            📄 `ملف <مسار>` — اقرأ ملف من المشروع\n\
            ⚡ `نفّذ <أمر>` — شغّل أمر WSL\n\
            🗂️ `مشروع` أو `ملفات` — اعرض هيكل المشروع\n\
            🧠 `اعدادات` — اضبط النموذج أو القنوات\n\
            🔌 `قنوات` — تحقق من حالة القنوات\n\
            📊 `حالة` — حالة النظام الحالية\n\n\
            **أو اسأل أي سؤال عن النظام!**\n\n\
            ---\n\
            أنا أعرف كود المشروع كامل وأقدر:\n\
            - أقرأ أي ملف (`src/tauri-backend/src/...`)\n\
            - أشغّل أوامر WSL\n\
            - أشخّص المشاكل وأقترح حلول\n\
            - أتذكّر المحادثات السابقة"
        );
    }

    // — سؤال عام — نرد بإجابة مفيدة
    format!(
        "أهلاً! 🫀\n\n\
        فهمت سؤالك: \"{}\"\n\n\
        عشان أساعدك بدقة، جرّب:\n\
        - `شخّص` — أشوف حالة النظام\n\
        - `ملف <مسار>` — أقرا ملف معين\n\
        - `نفّذ <أمر>` — أشغّل أمر\n\n\
        وش تبغى بالضبط؟",
        user_message.chars().take(80).collect::<String>()
    )
}

async fn run_diagnosis() -> String {
    let diagnoses = agent_health_check().await;
    if diagnoses.is_empty() {
        return "لم أجد أي مشاكل — كل شيء تمام ✅".into();
    }

    let mut out = String::from("## 🩺 تشخيص النظام\n\n");
    let mut has_issues = false;

    for d in &diagnoses {
        let icon = match d.severity.as_str() {
            "ok" => "✅",
            "warning" => "⚠️",
            "error" => "❌",
            "critical" => "🔥",
            _ => "ℹ️",
        };

        out.push_str(&format!("### {} {} — {}\n", icon, d.component, d.summary));

        if let Some(details) = &d.details {
            if !details.is_empty() && details != "null" {
                out.push_str(&format!("التفاصيل: {}\n", details));
            }
        }

        if d.fix_available {
            has_issues = true;
            if let Some(desc) = &d.fix_description {
                out.push_str(&format!("الحل: {}\n", desc));
            }
            if let Some(cmd) = &d.fix_command {
                out.push_str(&format!("الأمر: `{}`\n", cmd));
            }
        }

        out.push('\n');
    }

    if !has_issues {
        out.push_str("---\n✅ **النظام سليم! لا توجد مشاكل.**");
    } else {
        out.push_str("---\n⚠️ فيه مشاكل تحتاج انتباه. اكتب `نفّذ <الأمر>` عشان تطبّق الحل.");
    }

    out
}

fn chrono_now() -> String {
    use std::time::SystemTime;
    let total_secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // YYYY-MM-DDTHH:MM:SS بسيط UTC
    let secs_in_day: u64 = 86400;
    let days = total_secs / secs_in_day;
    let remaining = total_secs % secs_in_day;
    let hrs = remaining / 3600;
    let mins = (remaining % 3600) / 60;
    let secs = remaining % 60;
    // Epoch-based: 1970-01-01 + days
    // بسيط: نستخدم صيغة ISO يدوية
    format!("2026-05-23T{:02}:{:02}:{:02}Z", hrs, mins, secs)
}
