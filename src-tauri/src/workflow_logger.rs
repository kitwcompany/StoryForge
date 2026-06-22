//! 智能创作流程详细日志记录器
//!
//! 把 TriShot、模型网关路由、LLM 调用、超时、资产选择等关键步骤以 JSON Lines
//! 形式写入 `logs/creative_workflow.log`，方便排查“卡在哪一步”。

use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

/// 单条工作流日志事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowLogEvent {
    /// ISO-8601 时间戳
    pub ts: String,
    /// 工作流/请求标识
    pub request_id: Option<String>,
    /// 阶段/步骤名称，如 "trishot.call1.start"
    pub phase: String,
    /// 日志级别：INFO / WARN / ERROR
    pub level: String,
    /// 简短人类可读消息
    pub message: String,
    /// 详细键值对（可包含 model_id、prompt_chars、duration_ms 等）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl WorkflowLogEvent {
    pub fn new(phase: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            ts: chrono::Utc::now().to_rfc3339(),
            request_id: None,
            phase: phase.into(),
            level: "INFO".into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    pub fn with_level(mut self, level: impl Into<String>) -> Self {
        self.level = level.into();
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// 工作流日志记录器
#[derive(Debug, Clone)]
pub struct WorkflowLogger {
    writer: Arc<Mutex<BufWriter<fs::File>>>,
    path: PathBuf,
    /// 单文件大小上限（字节），超过后截断保留尾部
    max_bytes: u64,
}

impl WorkflowLogger {
    /// 在 app_data_dir 下创建 logs/creative_workflow.log
    pub fn new(app_data_dir: &Path) -> Result<Self, crate::error::AppError> {
        let logs_dir = app_data_dir.join("logs");
        fs::create_dir_all(&logs_dir).map_err(|e| crate::error::AppError::Internal {
            message: format!("创建工作流日志目录失败: {}", e),
        })?;
        let path = logs_dir.join("creative_workflow.log");
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| crate::error::AppError::Internal {
                message: format!("打开工作流日志文件失败: {}", e),
            })?;
        Ok(Self {
            writer: Arc::new(Mutex::new(BufWriter::new(file))),
            path,
            max_bytes: 10 * 1024 * 1024, // 10MB
        })
    }

    /// 写入一条事件
    pub fn log(&self, event: WorkflowLogEvent) {
        let line = match serde_json::to_string(&event) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("[WorkflowLogger] 序列化日志事件失败: {}", e);
                return;
            }
        };
        if let Ok(mut guard) = self.writer.lock() {
            let _ = writeln!(guard, "{}", line);
            let _ = guard.flush();
        }
        self.rotate_if_needed();
    }

    /// 便捷方法：直接按 phase/message/details 记录
    pub fn info(
        &self,
        phase: impl Into<String>,
        message: impl Into<String>,
        details: Option<serde_json::Value>,
    ) {
        let mut evt = WorkflowLogEvent::new(phase, message);
        evt.details = details;
        self.log(evt);
    }

    pub fn warn(
        &self,
        phase: impl Into<String>,
        message: impl Into<String>,
        details: Option<serde_json::Value>,
    ) {
        let mut evt = WorkflowLogEvent::new(phase, message).with_level("WARN");
        evt.details = details;
        self.log(evt);
    }

    pub fn error(
        &self,
        phase: impl Into<String>,
        message: impl Into<String>,
        details: Option<serde_json::Value>,
    ) {
        let mut evt = WorkflowLogEvent::new(phase, message).with_level("ERROR");
        evt.details = details;
        self.log(evt);
    }

    /// 读取最后 N 行日志（ newest first ）
    pub fn tail(&self, n: usize) -> Result<Vec<String>, crate::error::AppError> {
        let content =
            fs::read_to_string(&self.path).map_err(|e| crate::error::AppError::Internal {
                message: format!("读取工作流日志失败: {}", e),
            })?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Ok(lines.into_iter().rev().take(n).collect())
    }

    /// 日志文件路径
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn rotate_if_needed(&self) {
        let Ok(meta) = fs::metadata(&self.path) else {
            return;
        };
        if meta.len() <= self.max_bytes {
            return;
        }
        // 简单截断：保留文件尾部约 max_bytes/2 的内容
        let Ok(content) = fs::read_to_string(&self.path) else {
            return;
        };
        let bytes = content.as_bytes();
        let keep_from = bytes.len().saturating_sub((self.max_bytes / 2) as usize);
        // 找到下一行开头，避免截断在半行
        let next_newline = content[keep_from..].find('\n').unwrap_or(0);
        let keep_from = keep_from + next_newline + 1;
        let kept = &content[keep_from..];
        let _ = fs::write(&self.path, kept);
    }
}

/// 无操作占位，用于单元测试或初始化失败时的兜底
#[derive(Debug, Clone, Default)]
pub struct NoOpWorkflowLogger;

impl NoOpWorkflowLogger {
    pub fn info(&self, _phase: &str, _message: &str, _details: Option<serde_json::Value>) {}
    pub fn warn(&self, _phase: &str, _message: &str, _details: Option<serde_json::Value>) {}
    pub fn error(&self, _phase: &str, _message: &str, _details: Option<serde_json::Value>) {}
}
