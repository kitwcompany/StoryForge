//! Model capability audit & feedback loop — v0.11.0
//!
//! 提供：
//! 1. 按任务类型对模型进行轻量 benchmark（探测 + 评分）
//! 2. 从 llm_calls 聚合生成模型健康报告
//! 3. 用户/系统反馈回填，修正路由偏好

use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Manager};

use super::TaskType;
use crate::{
    config::settings::AppConfig,
    db::{DbPool, LlmCallRepository},
    error::AppError,
    llm::service::LlmService,
};

/// 单个任务类型的 benchmark 结果
#[derive(Debug, Clone, Serialize)]
pub struct TaskBenchmarkResult {
    pub task: TaskType,
    pub model_id: String,
    pub model_name: String,
    pub success: bool,
    pub latency_ms: u64,
    pub score: f64,
    pub reason: String,
}

/// 模型健康报告
#[derive(Debug, Clone, Serialize)]
pub struct ModelHealthReport {
    pub model_id: String,
    pub model_name: String,
    /// 近 N 次调用成功率（0-1）
    pub success_rate: f64,
    /// 平均延迟（ms）
    pub avg_latency_ms: f64,
    /// 平均质量分（如有）
    pub avg_quality_score: Option<f64>,
    /// 最近错误信息
    pub last_error: Option<String>,
    /// 综合健康评级：healthy / degraded / unhealthy
    pub status: String,
    /// v0.17.1: 最近聚合的调用次数（让用户判断数据是否陈旧）
    #[serde(default)]
    pub total_calls: i64,
    /// v0.17.1: 最近一次调用时间（ISO8601）
    #[serde(default)]
    pub last_called_at: Option<String>,
    /// v0.17.1: 报告生成时间（ISO8601），刷新后端会更新
    #[serde(default)]
    pub generated_at: String,
}

/// 路由反馈条目：用户或系统对某次路由结果的评价
#[derive(Debug, Clone, Deserialize)]
pub struct RouteFeedback {
    pub call_id: String,
    /// 1-5 分，5 为非常满意
    pub score: i32,
    pub comment: Option<String>,
}

/// 对指定模型执行轻量任务 benchmark
#[command]
pub async fn benchmark_model_for_task(
    model_id: String,
    task: TaskType,
    app_handle: AppHandle,
) -> Result<TaskBenchmarkResult, AppError> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app dir: {}", e))?;
    let config = AppConfig::load(&app_dir).map_err(AppError::from)?;

    let profile =
        config
            .llm_profiles
            .get(&model_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound {
                resource: "llm_profile".to_string(),
                id: model_id.clone(),
            })?;

    let prompt = benchmark_prompt_for_task(task);
    let service = LlmService::new(app_handle.clone());
    let start = std::time::Instant::now();

    match service
        .generate_with_profile_and_request_id(
            &profile.id,
            prompt,
            Some(256),
            Some(0.3),
            Some("benchmark"),
            None,
            None,
            None,
        )
        .await
    {
        (_request_id, Ok(response)) => {
            let latency_ms = start.elapsed().as_millis();
            let latency_ms_u64 = latency_ms as u64;
            let score = heuristic_score(&response.content, task);
            Ok(TaskBenchmarkResult {
                task,
                model_id: profile.id.clone(),
                model_name: profile.name.clone(),
                success: true,
                latency_ms: latency_ms_u64,
                score,
                reason: format!("{}ms, 启发式评分 {:.1}", latency_ms_u64, score),
            })
        }
        (request_id, Err(e)) => Ok(TaskBenchmarkResult {
            task,
            model_id: profile.id.clone(),
            model_name: profile.name.clone(),
            success: false,
            latency_ms: start.elapsed().as_millis() as u64,
            score: 0.0,
            reason: format!("生成失败: {}, request_id={}", e, request_id),
        }),
    }
}

/// 生成所有启用模型在常见任务上的健康报告
/// v0.22.3: 改为 async 命令，避免阻塞 Tauri IPC 主线程。
/// 配合 settings.rs 的钥匙串内存缓存，查询毫秒级返回。
#[command]
pub async fn get_model_health_reports(
    window_limit: Option<i64>,
    app_handle: AppHandle,
) -> Result<Vec<ModelHealthReport>, AppError> {
    let pool = app_handle
        .try_state::<DbPool>()
        .ok_or_else(|| AppError::internal("DbPool not available".to_string()))?;
    let repo = LlmCallRepository::new(pool.inner().clone());

    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app dir: {}", e))?;
    let config = AppConfig::load(&app_dir).map_err(AppError::from)?;

    let limit = window_limit.unwrap_or(50);
    let calls = repo.get_recent(limit * 10).map_err(AppError::from)?;

    let mut by_model: std::collections::HashMap<String, Vec<crate::db::models::LlmCall>> =
        std::collections::HashMap::new();
    for call in calls {
        by_model
            .entry(call.model_id.clone())
            .or_default()
            .push(call);
    }

    let now_iso = chrono::Local::now().to_rfc3339();
    let mut reports = Vec::new();
    for (model_id, calls) in by_model {
        let model_name = calls
            .first()
            .and_then(|c| c.model_name.clone())
            .or_else(|| config.llm_profiles.get(&model_id).map(|p| p.name.clone()))
            .unwrap_or_else(|| model_id.clone());

        let total = calls.len() as f64;
        let successes = calls.iter().filter(|c| c.success).count() as f64;
        let success_rate = if total > 0.0 { successes / total } else { 0.0 };

        let avg_latency = calls.iter().map(|c| c.duration_ms as f64).sum::<f64>() / total.max(1.0);
        let quality_scores: Vec<f64> = calls.iter().filter_map(|c| c.quality_score).collect();
        let avg_quality = if !quality_scores.is_empty() {
            Some(quality_scores.iter().sum::<f64>() / quality_scores.len() as f64)
        } else {
            None
        };

        let last_error = calls
            .iter()
            .find(|c| !c.success)
            .and_then(|c| c.error_message.clone());

        // v0.17.1: 最近一次调用时间（用 calls 已按 created_at DESC 排序）
        let last_called_at = calls.first().map(|c| c.created_at.to_rfc3339());

        let status = if success_rate >= 0.95 && avg_latency < 10000.0 {
            "healthy"
        } else if success_rate >= 0.7 {
            "degraded"
        } else {
            "unhealthy"
        }
        .to_string();

        reports.push(ModelHealthReport {
            model_id,
            model_name,
            success_rate,
            avg_latency_ms: avg_latency,
            avg_quality_score: avg_quality,
            last_error,
            status,
            total_calls: calls.len() as i64,
            last_called_at,
            generated_at: now_iso.clone(),
        });
    }

    // 补充配置中有但无调用记录的模型
    for profile in config.llm_profiles.values() {
        if !reports.iter().any(|r| r.model_id == profile.id) {
            reports.push(ModelHealthReport {
                model_id: profile.id.clone(),
                model_name: profile.name.clone(),
                success_rate: 0.0,
                avg_latency_ms: 0.0,
                avg_quality_score: None,
                last_error: None,
                status: "unknown".to_string(),
                total_calls: 0,
                last_called_at: None,
                generated_at: now_iso.clone(),
            });
        }
    }

    reports.sort_by(|a, b| b.success_rate.partial_cmp(&a.success_rate).unwrap());
    Ok(reports)
}

/// 提交路由/生成反馈，回填到 llm_calls 记录
#[command]
pub fn submit_route_feedback(
    feedback: RouteFeedback,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let pool = app_handle
        .try_state::<DbPool>()
        .ok_or_else(|| AppError::internal("DbPool not available".to_string()))?;

    let conn = pool
        .inner()
        .get()
        .map_err(|e| AppError::internal(format!("Failed to get db connection: {}", e)))?;

    let audit = serde_json::json!({
        "feedback_score": feedback.score.clamp(1, 5),
        "comment": feedback.comment,
        "submitted_at": chrono::Utc::now().to_rfc3339(),
    })
    .to_string();

    conn.execute(
        "UPDATE llm_calls SET audit_feedback = ?1 WHERE id = ?2",
        [&audit, &feedback.call_id],
    )
    .map_err(|e| AppError::internal(format!("Failed to update feedback: {}", e)))?;

    Ok(())
}

fn benchmark_prompt_for_task(task: TaskType) -> String {
    match task {
        TaskType::CreativeWriting => "用一句话写一个带有悬念的开头。".to_string(),
        TaskType::Editing => "将以下句子改写得更加流畅：'他走进了房间，看到了她。'".to_string(),
        TaskType::Analysis => "分析'他握紧了拳头'这句话暗示的人物情绪。".to_string(),
        TaskType::Dialogue => "为一位古代剑客写一句符合身份的台词。".to_string(),
        TaskType::Summarization => {
            "用一句话总结：'春天来了，花儿开了，鸟儿在枝头歌唱。'".to_string()
        }
        TaskType::Brainstorming => "给出三个关于'时间旅行'的故事创意。".to_string(),
        TaskType::Proofreading => "找出这句话中的错别字：'他兴高彩烈地离开了。'".to_string(),
        TaskType::WorldBuilding => "描述一个终年下雪的魔法城市。".to_string(),
        TaskType::Vision => "（vision benchmark 需由多模态模块单独实现）".to_string(),
        TaskType::ImageGeneration => {
            "（image generation benchmark 需由图像模块单独实现）".to_string()
        }
    }
}

fn heuristic_score(text: &str, task: TaskType) -> f64 {
    let len = text.chars().count() as f64;
    if len < 5.0 {
        return 10.0;
    }
    let mut score = 50.0;
    score += len.min(200.0) * 0.1;

    match task {
        TaskType::CreativeWriting | TaskType::WorldBuilding => {
            if text.contains("？") || text.contains("!") || text.contains("？") {
                score += 10.0;
            }
        }
        TaskType::Proofreading => {
            if text.contains("采") {
                score += 20.0;
            }
        }
        TaskType::Brainstorming => {
            if text.contains("1.") || text.contains("2.") || text.contains("3.") {
                score += 15.0;
            }
        }
        _ => {}
    }

    score.min(100.0)
}
