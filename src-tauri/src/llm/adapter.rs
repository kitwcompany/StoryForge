#![allow(dead_code)]
use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

/// 适配器内部用于标识连接阶段超时的错误标记。
pub const CONNECTION_TIMEOUT_MARKER: &str = "LLM_CONNECTION_TIMEOUT";
/// 适配器内部用于标识生成/读取阶段超时的错误标记。
pub const GENERATION_TIMEOUT_MARKER: &str = "LLM_GENERATION_TIMEOUT";

/// 发送 HTTP 请求并在连接阶段应用超时。
/// 超时或 reqwest 连接错误均映射为 CONNECTION_TIMEOUT_MARKER。
pub async fn send_with_connection_timeout(
    request: reqwest::RequestBuilder,
    connect_timeout: Duration,
) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
    match timeout(connect_timeout, request.send()).await {
        Ok(Ok(resp)) => Ok(resp),
        Ok(Err(e)) => {
            if e.is_timeout() {
                Err(CONNECTION_TIMEOUT_MARKER.into())
            } else {
                Err(e.into())
            }
        }
        Err(_) => Err(CONNECTION_TIMEOUT_MARKER.into()),
    }
}

/// 以流式方式读取响应体，每收到一个 chunk 刷新一次生成超时计时器，
/// 避免本地模型生成慢但仍在输出时被整体超时误杀。
pub async fn read_body_with_generation_timeout(
    response: reqwest::Response,
    generation_timeout: Duration,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut stream = response.bytes_stream();
    let mut chunks: Vec<Vec<u8>> = Vec::new();
    loop {
        match timeout(generation_timeout, stream.next()).await {
            Ok(Some(Ok(bytes))) => chunks.push(bytes.to_vec()),
            Ok(Some(Err(e))) => return Err(e.into()),
            Ok(None) => break,
            Err(_) => return Err(GENERATION_TIMEOUT_MARKER.into()),
        }
    }
    Ok(chunks.into_iter().flatten().collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    pub max_tokens: Option<i32>,
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub content: String,
    pub model: String,
    pub tokens_used: i32,
    pub cost: f64,
}

#[async_trait::async_trait]
pub trait LlmAdapter: Send + Sync {
    async fn generate(
        &self,
        request: GenerateRequest,
    ) -> Result<GenerateResponse, Box<dyn std::error::Error>>;

    async fn generate_stream(
        &self,
        request: GenerateRequest,
    ) -> Result<
        tokio::sync::mpsc::Receiver<Result<String, Box<dyn std::error::Error + Send + Sync>>>,
        Box<dyn std::error::Error + Send + Sync>,
    >;

    fn model_name(&self) -> String;

    /// 克隆自身为新的 Box<dyn LlmAdapter>，用于缓存复用
    fn box_clone(&self) -> Box<dyn LlmAdapter>;
}
