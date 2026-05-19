// Setup Module — فحص وتثبيت WSL + OpenClaw بشكل مرئي وتفاعلي
use serde::{Deserialize, Serialize};
use tauri;
use std::process::Command;

// ============================================================
// هياكل البيانات
// ============================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemStatus {
    pub overall_phase: SetupPhase,
    pub wsl: ComponentStatus,
    pub ubuntu: ComponentStatus,
    pub nodejs: ComponentStatus,
    pub openclaw: ComponentStatus,
    pub config: ComponentStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComponentStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub details: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SetupPhase {
    NoWSL,           // مافيه WSL أصلًا
    WSLNoDistro,     // فيه WSL بس مافيه Ubuntu
    DistroNoOpenClaw,// فيه Ubuntu بس مافيه OpenClaw
    OpenClawNoConfig,// فيه OpenClaw بس غير مهيأ
    OpenClawRunning, // كل شيء تمام!
    OpenClawStopped, // OpenClaw منصب بس واقف
    Error(String),   // خطأ غير متوقع
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupStep {
    pub step_id: u32,
    pub title: String,
    pub description: String,
    pub explanation: String,
    pub recommendation: String,
    pub status: StepStatus,
    pub action_label: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum StepStatus {
    Pending,
    Current,
    Done,
    Error(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallationGuide {
    pub steps: Vec<SetupStep>,
    pub current_step: u32,
    pub total_steps: u32,
    pub overall_progress: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WslInstallGuide {
    pub enabled: bool,
    pub steps: Vec<String>,
    pub estimated_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenClawInstallOptions {
    pub method: String,
    pub description: String,
    pub command: String,
    pub recommended: bool,
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelRecommendation {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub cost_tier: String,        // "free" | "low" | "medium" | "high"
    pub speed: String,            // "fast" | "medium" | "slow"
    pub quality: String,          // "good" | "great" | "excellent"
    pub best_for: Vec<String>,    // use cases
    pub requires_api_key: bool,
    pub recommendation_level: String, // "recommended" | "good" | "budget" | "premium"
    pub explanation_ar: String,
}

// ============================================================
// أوامر الفحص
// ============================================================

/// فحص شامل للنظام — يحدد المرحلة الحالية
#[tauri::command]
pub fn check_full_system() -> SystemStatus {
    let wsl = check_wsl_installed();
    let ubuntu = if wsl.installed { check_ubuntu_distro() } else { ComponentStatus {
        installed: false, version: None, details: "WSL غير مثبت".into()
    }};
    let nodejs = if ubuntu.installed { check_nodejs_installed() } else { ComponentStatus {
        installed: false, version: None, details: "Ubuntu غير مثبت".into()
    }};
    let openclaw = if ubuntu.installed { check_openclaw_binary() } else { ComponentStatus {
        installed: false, version: None, details: "Ubuntu غير مثبت".into()
    }};
    let config = if openclaw.installed { check_openclaw_config() } else { ComponentStatus {
        installed: false, version: None, details: "OpenClaw غير مثبت".into()
    }};

    let phase = if !wsl.installed {
        SetupPhase::NoWSL
    } else if !ubuntu.installed {
        SetupPhase::WSLNoDistro
    } else if !openclaw.installed {
        SetupPhase::DistroNoOpenClaw
    } else if !config.installed {
        SetupPhase::OpenClawNoConfig
    } else {
        // تحقق مما إذا كان Gateway شغال
        let gw = check_gateway_process();
        if gw { SetupPhase::OpenClawRunning } else { SetupPhase::OpenClawStopped }
    };

    SystemStatus { overall_phase: phase, wsl, ubuntu, nodejs, openclaw, config }
}

/// ابحث عن wsl.exe في المسارات المعروفة
fn find_wsl() -> Option<String> {
    let paths = [
        "wsl.exe",
        r"C:\Windows\System32\wsl.exe",
        r"C:\Windows\Sysnative\wsl.exe",
    ];
    for p in &paths {
        if std::path::Path::new(p).exists() || Command::new(p).arg("--version").output().is_ok() {
            return Some(p.to_string());
        }
    }
    None
}

/// فحص WSL
fn check_wsl_installed() -> ComponentStatus {
    let wsl_exe = find_wsl().unwrap_or_else(|| "wsl.exe".to_string());

    // جرب wsl --version أولاً (الأكثر توافقًا)
    let output = Command::new(&wsl_exe)
        .args(["--version"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{}{}", stdout, stderr);
            // WSL موجود إذا لقينا "WSL version" أو "kernel" في المخرجات
            let installed = combined.contains("WSL") || combined.contains("kernel");

            if installed {
                let version = stdout.lines()
                    .find(|l| l.contains("WSL"))
                    .map(|l| l.trim().to_string());
                ComponentStatus { installed: true, version, details: combined }
            } else {
                // جرب wsl --status كخطة بديلة
                let output2 = Command::new(&wsl_exe).args(["--status"]).output();
                if let Ok(out2) = output2 {
                    let s = String::from_utf8_lossy(&out2.stdout);
                    ComponentStatus { installed: out2.status.success(), version: None, details: s.to_string() }
                } else {
                    ComponentStatus { installed: false, version: None, details: "WSL غير موجود".into() }
                }
            }
        }
        Err(e) => ComponentStatus {
            installed: false,
            version: None,
            details: format!("WSL غير موجود: {}", e),
        },
    }
}

/// فحص توزيعة Ubuntu
fn check_ubuntu_distro() -> ComponentStatus {
    let wsl_exe = find_wsl().unwrap_or_else(|| "wsl.exe".to_string());

    // wsl -l -q يعطي قائمة بالتوزيعات (سطر واحد لكل توزيعة)
    let output = Command::new(&wsl_exe)
        .args(["-l", "-q"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // نحول لـ lowercase عشان المقارنة
            let lower = stdout.to_lowercase();
            let has_ubuntu = lower.contains("ubuntu");

            // نجيب اسم التوزيعة بالضبط (للأوامر اللاحقة)
            let distro_name = if has_ubuntu {
                stdout.lines()
                    .find(|l| l.to_lowercase().contains("ubuntu"))
                    .map(|l| l.trim().to_string())
            } else {
                None
            };

            ComponentStatus {
                installed: has_ubuntu,
                version: distro_name,
                details: stdout.to_string(),
            }
        }
        Err(e) => ComponentStatus {
            installed: false,
            version: None,
            details: format!("خطأ: {}", e),
        },
    }
}

/// فحص Node.js داخل Ubuntu
fn check_nodejs_installed() -> ComponentStatus {
    let output = Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "node", "--version"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            ComponentStatus {
                installed: out.status.success(),
                version: if out.status.success() { Some(stdout.clone()) } else { None },
                details: stdout,
            }
        }
        Err(e) => ComponentStatus {
            installed: false,
            version: None,
            details: format!("{}", e),
        },
    }
}

/// فحص وجود OpenClaw
fn check_openclaw_binary() -> ComponentStatus {
    let output = Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "which", "openclaw"])
        .output();

    match output {
        Ok(out) => {
            let installed = out.status.success();
            let version = if installed {
                let ver = Command::new("wsl.exe")
                    .args(["-d", "Ubuntu", "--", "openclaw", "--version"])
                    .output();
                match ver {
                    Ok(v) => Some(String::from_utf8_lossy(&v.stdout).trim().to_string()),
                    Err(_) => None,
                }
            } else { None };

            ComponentStatus {
                installed,
                version,
                details: if installed { "OpenClaw مثبت ✓".into() } else { "OpenClaw غير مثبت".into() },
            }
        }
        Err(e) => ComponentStatus {
            installed: false,
            version: None,
            details: format!("{}", e),
        },
    }
}

/// فحص وجود ملف الإعدادات
fn check_openclaw_config() -> ComponentStatus {
    let output = Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "bash", "-c",
            "test -f ~/.openclaw/openclaw.json && echo 'EXISTS' || echo 'NOT_FOUND'"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let exists = stdout.contains("EXISTS");
            ComponentStatus {
                installed: exists,
                version: None,
                details: if exists { "الإعدادات موجودة ✓".into() } else { "لا توجد إعدادات بعد".into() },
            }
        }
        Err(e) => ComponentStatus {
            installed: false,
            version: None,
            details: format!("{}", e),
        },
    }
}

/// فحص عملية Gateway
fn check_gateway_process() -> bool {
    let output = Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "bash", "-c",
            "pgrep -f 'openclaw gateway' > /dev/null && echo 'RUNNING' || echo 'STOPPED'"])
        .output();

    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).trim() == "RUNNING",
        Err(_) => false,
    }
}

fn extract_version(output: &str, _name: &str) -> Option<String> {
    output.lines()
        .find(|l| l.contains("版本") || l.contains("version") || l.contains("WSL"))
        .map(|l| l.trim().to_string())
}

// ============================================================
// أوامر التثبيت المرئية
// ============================================================

/// دليل تثبيت WSL
#[tauri::command]
pub fn get_wsl_install_guide() -> WslInstallGuide {
    WslInstallGuide {
        enabled: true,
        steps: vec![
            "1. افتح PowerShell كمسؤول (Admin)".into(),
            "2. شغّل الأمر: wsl --install".into(),
            "3. انتظر اكتمال التحميل (قد يستغرق 5-10 دقائق)".into(),
            "4. سيُطلب منك إنشاء اسم مستخدم وكلمة مرور لـ Ubuntu".into(),
            "5. بعد الانتهاء، اختبر بـ: wsl -l -v".into(),
        ],
        estimated_time: "10-15 دقيقة".into(),
    }
}

/// خيارات تثبيت OpenClaw مع شرح لكل خيار
#[tauri::command]
pub fn get_openclaw_install_options() -> Vec<OpenClawInstallOptions> {
    vec![
        OpenClawInstallOptions {
            method: "npm (مستقر)".into(),
            description: "التثبيت عبر npm — الإصدار المستقر الموصى به".into(),
            command: "npm install -g openclaw".into(),
            recommended: true,
            notes: "أنصح بهذا الخيار للمستخدم العادي. التحديثات سهلة والتثبيت نظيف.".into(),
        },
        OpenClawInstallOptions {
            method: "git (أحدث إصدار)".into(),
            description: "التثبيت من مستودع git — أحدث الميزات".into(),
            command: "git clone https://github.com/openclaw/openclaw.git && cd openclaw && pnpm install && pnpm build && npm link".into(),
            recommended: false,
            notes: "مناسب للمستخدمين المتقدمين الذين يريدون أحدث الميزات. قد يواجه بعض عدم الاستقرار.".into(),
        },
    ]
}

/// الحصول على الخطوات الإرشادية للمرحلة الحالية
#[tauri::command]
pub fn get_setup_guide(phase: String) -> InstallationGuide {
    match phase.as_str() {
        "NoWSL" => InstallationGuide {
            current_step: 1,
            total_steps: 5,
            overall_progress: 0.0,
            steps: vec![
                SetupStep {
                    step_id: 1, title: "تثبيت WSL".into(),
                    description: "نظام Linux الفرعي لويندوز — البيئة التي سيعمل عليها OpenClaw.".into(),
                    explanation: "WSL يسمح لك بتشغيل Linux داخل ويندوز بدون جهاز افتراضي. OpenClaw يحتاج بيئة Linux.".into(),
                    recommendation: "أفتح PowerShell كمسؤول واكتب: wsl --install. هذا سيثبت WSL وتوزيعة Ubuntu تلقائيًا.".into(),
                    status: StepStatus::Current, action_label: "فتح دليل التثبيت".into(),
                },
                SetupStep {
                    step_id: 2, title: "تثبيت Ubuntu".into(),
                    description: "توزيعة Linux الموصى بها لـ OpenClaw.".into(),
                    explanation: "إذا لم يتم تثبيت Ubuntu تلقائيًا، يمكنك تثبيتها من متجر Microsoft Store أو عبر الأمر: wsl --install -d Ubuntu".into(),
                    recommendation: "أنصح باستخدام Ubuntu 24.04 LTS للاستقرار.".into(),
                    status: StepStatus::Pending, action_label: "تثبيت Ubuntu".into(),
                },
                SetupStep {
                    step_id: 3, title: "تثبيت OpenClaw".into(),
                    description: "المساعد الشخصي الذكي — قلب النظام.".into(),
                    explanation: "لديك خياران: التثبيت المستقر (npm) وهو الأسهل والموصى به، أو التثبيت من المصدر (git) لأحدث الميزات.".into(),
                    recommendation: "أنصح بـ npm install -g openclaw للمستخدم العادي.".into(),
                    status: StepStatus::Pending, action_label: "تثبيت OpenClaw".into(),
                },
                SetupStep {
                    step_id: 4, title: "إعداد OpenClaw".into(),
                    description: "تهيئة المساعد حسب احتياجاتك — الموديلات، القنوات، الإعدادات.".into(),
                    explanation: "سنختار الموديل المناسب لاستخدامك، ونربط القنوات (واتساب، تيليجرام)، ونضبط الإعدادات الأساسية.".into(),
                    recommendation: "ابدأ بـ openclaw onboard للإعداد التدريجي. سنساعدك في اختيار الأنسب.".into(),
                    status: StepStatus::Pending, action_label: "بدء الإعداد".into(),
                },
                SetupStep {
                    step_id: 5, title: "تشغيل Gateway".into(),
                    description: "تشغيل الخدمة الرئيسية — OpenClaw Gateway.".into(),
                    explanation: "Gateway هو الخدمة التي تستقبل الرسائل وتتواصل مع الموديلات والقنوات.".into(),
                    recommendation: "openclaw gateway start سيشغل الخدمة كخلفية.".into(),
                    status: StepStatus::Pending, action_label: "تشغيل Gateway".into(),
                },
            ],
        },
        "WSLNoDistro" => InstallationGuide {
            current_step: 2,
            total_steps: 5,
            overall_progress: 0.25,
            steps: vec![
                SetupStep {
                    step_id: 1, title: "تثبيت WSL".into(),
                    description: "تم تثبيت WSL بنجاح ✓".into(),
                    explanation: "الخطوة الأولى اكتملت.".into(),
                    recommendation: "".into(),
                    status: StepStatus::Done, action_label: "".into(),
                },
                SetupStep {
                    step_id: 2, title: "تثبيت Ubuntu".into(),
                    description: "تحتاج توزيعة Ubuntu لتشغيل OpenClaw.".into(),
                    explanation: "WSL مثبت لكن لا توجد توزيعة Ubuntu. يمكن تثبيتها من متجر Microsoft أو عبر سطر الأوامر.".into(),
                    recommendation: "wsl --install -d Ubuntu-24.04 هو الأسرع والأفضل.".into(),
                    status: StepStatus::Current, action_label: "تثبيت Ubuntu".into(),
                },
                // باقي الخطوات...
                SetupStep {
                    step_id: 3, title: "تثبيت OpenClaw".into(),
                    description: "بانتظار اكتمال تثبيت Ubuntu.".into(),
                    explanation: "".into(),
                    recommendation: "".into(),
                    status: StepStatus::Pending, action_label: "".into(),
                },
                SetupStep {
                    step_id: 4, title: "إعداد OpenClaw".into(),
                    description: "بانتظار الخطوات السابقة.".into(),
                    explanation: "".into(),
                    recommendation: "".into(),
                    status: StepStatus::Pending, action_label: "".into(),
                },
                SetupStep {
                    step_id: 5, title: "تشغيل Gateway".into(),
                    description: "بانتظار الخطوات السابقة.".into(),
                    explanation: "".into(),
                    recommendation: "".into(),
                    status: StepStatus::Pending, action_label: "".into(),
                },
            ],
        },
        "DistroNoOpenClaw" => InstallationGuide {
            current_step: 3,
            total_steps: 5,
            overall_progress: 0.5,
            steps: vec![
                SetupStep {
                    step_id: 1, title: "تثبيت WSL".into(),
                    status: StepStatus::Done, action_label: "".into(),
                    description: "تم ✓".into(), explanation: "".into(), recommendation: "".into(),
                },
                SetupStep {
                    step_id: 2, title: "تثبيت Ubuntu".into(),
                    status: StepStatus::Done, action_label: "".into(),
                    description: "تم ✓".into(), explanation: "".into(), recommendation: "".into(),
                },
                SetupStep {
                    step_id: 3, title: "تثبيت OpenClaw".into(),
                    description: "OpenClaw جاهز للتثبيت في Ubuntu.".into(),
                    explanation: "OpenClaw هو مساعد ذكي يستطيع التواصل عبر واتساب، تيليجرام، وغيرها. يعتمد على موديلات ذكاء اصطناعي مثل Claude وGPT.".into(),
                    recommendation: "أنصح بالتثبيت عبر npm لأنه أسرع وأنظف: سيفتح terminal وينفذ الأمر نيابة عنك.".into(),
                    status: StepStatus::Current, action_label: "تثبيت (npm)".into(),
                },
                SetupStep {
                    step_id: 4, title: "إعداد OpenClaw".into(), 
                    description: "بانتظار التثبيت.".into(),
                    explanation: "".into(), recommendation: "".into(),
                    status: StepStatus::Pending, action_label: "".into(),
                },
                SetupStep {
                    step_id: 5, title: "تشغيل Gateway".into(),
                    description: "بانتظار الخطوات السابقة.".into(),
                    explanation: "".into(), recommendation: "".into(),
                    status: StepStatus::Pending, action_label: "".into(),
                },
            ],
        },
        _ => InstallationGuide {
            current_step: 0, total_steps: 5, overall_progress: 1.0,
            steps: vec![],
        },
    }
}

/// توصيات الموديلات حسب حالة الاستخدام
#[tauri::command]
pub fn get_model_recommendations() -> Vec<ModelRecommendation> {
    vec![
        ModelRecommendation {
            id: "claude-sonnet-4-6".into(),
            name: "Claude Sonnet 4.6".into(),
            provider: "Anthropic".into(),
            cost_tier: "medium".into(),
            speed: "fast".into(),
            quality: "excellent".into(),
            best_for: vec!["الاستخدام اليومي".into(), "محادثات عامة".into(), "مساعد شخصي".into()],
            requires_api_key: true,
            recommendation_level: "recommended".into(),
            explanation_ar: "أنصح بهذا الموديل لـ 90% من المستخدمين. سريع، ذكي، اقتصادي. مناسب للمساعد الشخصي اليومي.".into(),
        },
        ModelRecommendation {
            id: "gpt-4o".into(),
            name: "GPT-4o".into(),
            provider: "OpenAI".into(),
            cost_tier: "medium".into(),
            speed: "fast".into(),
            quality: "excellent".into(),
            best_for: vec!["البرمجة".into(), "التحليل".into(), "التعدد اللغوي".into()],
            requires_api_key: true,
            recommendation_level: "good".into(),
            explanation_ar: "بديل ممتاز لـ Claude. قوي في البرمجة والتحليل. يناسب المستخدمين الذين يفضلون OpenAI.".into(),
        },
        ModelRecommendation {
            id: "claude-haiku-4.5".into(),
            name: "Claude Haiku 4.5".into(),
            provider: "Anthropic".into(),
            cost_tier: "low".into(),
            speed: "very_fast".into(),
            quality: "great".into(),
            best_for: vec!["الردود السريعة".into(), "المهام البسيطة".into(), "توفير التكاليف".into()],
            requires_api_key: true,
            recommendation_level: "budget".into(),
            explanation_ar: "أسرع موديل وأرخص. مناسب للردود السريعة والمهام البسيطة. جودة جيدة جدًا مقابل السعر.".into(),
        },
        ModelRecommendation {
            id: "gemini-2.0-flash".into(),
            name: "Gemini 2.0 Flash".into(),
            provider: "Google".into(),
            cost_tier: "free".into(),
            speed: "very_fast".into(),
            quality: "great".into(),
            best_for: vec!["البدء مجانًا".into(), "الاختبار".into(), "الاستخدام الخفيف".into()],
            requires_api_key: true,
            recommendation_level: "recommended".into(),
            explanation_ar: "خيار ممتاز للمستخدمين الجدد! لديه طبقة مجانية سخية. جودة جيدة وسرعة عالية. ابدأ به.".into(),
        },
    ]
}

/// تنفيذ أمر تثبيت في WSL
#[tauri::command]
pub fn run_install_command(command: String) -> String {
    let output = Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "bash", "-c", &command])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            if out.status.success() {
                format!("✅ تم بنجاح\n{}", stdout)
            } else {
                format!("❌ فشل:\n{}", stderr)
            }
        }
        Err(e) => format!("❌ خطأ: {}", e),
    }
}
