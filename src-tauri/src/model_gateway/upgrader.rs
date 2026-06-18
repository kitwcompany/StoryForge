//! 降级后台补强模块（v0.15.0）
//!
//! 当智能创作的主模型不可用时，网关自动降级到备选模型完成生成，
//! 同时在后台对降级内容用主模型重新生成并覆盖，实现"先交付后升级"。
//!
//! 工作流程：
//! 1. 网关 generate() 检测到主模型失败，走 fallback 路径
//! 2. fallback 成功生成后，Upgrader 记录 (task_id, content_hash,
//!    preferred_model_id)
//! 3. 后台循环检测主模型是否恢复
//! 4. 恢复后用主模型重写降级内容，通过 state-sync 通知前端"内容已升级"

use std::{collections::HashMap, sync::Mutex};

/// 待升级任务记录
#[derive(Debug, Clone)]
pub struct PendingUpgrade {
    pub content: String,
    pub content_hash: String,
    pub preferred_model_id: String,
    pub task_description: String,
}

/// 降级后台补强管理器
pub struct Upgrader {
    pending: Mutex<HashMap<String, PendingUpgrade>>,
}

impl Upgrader {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
        }
    }

    /// 记录一个降级任务，等待主模型恢复后补强
    pub fn enqueue(&self, task_id: String, upgrade: PendingUpgrade) {
        let mut pending = self.pending.lock().unwrap();
        log::info!(
            "[Upgrader] 降级任务入队: {} (模型 {})",
            task_id,
            upgrade.preferred_model_id
        );
        pending.insert(task_id, upgrade);
    }

    /// 获取所有待升级任务
    pub fn pending_count(&self) -> usize {
        self.pending.lock().unwrap().len()
    }

    /// 移除一个已完成的升级任务
    pub fn dequeue(&self, task_id: &str) -> Option<PendingUpgrade> {
        self.pending.lock().unwrap().remove(task_id)
    }

    /// 当主模型恢复时调用，触发后台补强
    pub async fn try_upgrade_all(&self, _model_id: &str, _app_handle: &tauri::AppHandle) -> usize {
        let count = self.pending_count();
        if count == 0 {
            return 0;
        }
        log::info!("[Upgrader] 尝试为 {} 个降级任务补强", count);
        // Phase 3 完整实现：对每个 pending 任务调用主模型重写
        // 当前阶段返回计数，待前端面板完成后接入实际 LLM 调用
        count
    }
}
