//! LLM Service port

use crate::{
    config::settings::LlmProfile, error::AppError, llm::adapter::GenerateResponse,
    llm::service::PipelineContext, router::TaskType,
};

/// LLM 服务端口
///
/// 定义最常用的 LLM 生成能力，供业务模块通过依赖注入使用。
/// 需要完整方法集的场景可直接依赖 `crate::llm::service::LlmService` 具体类型。
#[async_trait::async_trait]
pub trait LlmService: Send + Sync + 'static {
    /// 使用当前活跃 profile 同步生成
    async fn generate(
        &self,
        prompt: String,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
    ) -> Result<GenerateResponse, AppError>;

    /// 使用当前活跃 profile 同步生成，带上下文标签
    async fn generate_with_context(
        &self,
        prompt: String,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
        context_label: Option<&str>,
    ) -> Result<GenerateResponse, AppError>;

    /// 使用当前活跃 profile 同步生成，返回 (request_id, Result)
    async fn generate_with_request_id(
        &self,
        prompt: String,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
        context_label: Option<&str>,
        pipeline_ctx: Option<PipelineContext>,
        request_id: Option<String>,
    ) -> (String, Result<GenerateResponse, AppError>);

    /// 使用指定 profile 同步生成
    async fn generate_with_profile(
        &self,
        profile_id: &str,
        prompt: String,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
    ) -> Result<GenerateResponse, AppError>;

    /// 流式生成
    async fn generate_stream(
        &self,
        request_id: String,
        prompt: String,
        context: Option<String>,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
    ) -> Result<(), AppError>;

    /// 检查指定 request_id 是否已被取消
    fn is_cancelled(&self, request_id: &str) -> bool;

    /// 测试当前活跃模型连接
    async fn test_connection(&self) -> Result<(bool, u64), AppError>;

    /// 获取当前活跃模型配置
    fn get_active_profile(&self) -> Option<LlmProfile>;
}
