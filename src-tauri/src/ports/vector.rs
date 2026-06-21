//! Vector Store port

use crate::vector::{SearchResult, VectorRecord};

/// 向量存储端口
///
/// 抽象 LanceDB 等向量数据库的增删查能力，供业务模块依赖注入使用。
#[async_trait::async_trait]
pub trait VectorStore: Send + Sync + 'static {
    /// 初始化向量存储（幂等）
    async fn init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 插入或更新单条记录
    async fn upsert(
        &self,
        record: VectorRecord,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 批量插入或更新
    async fn upsert_batch(
        &self,
        records: &[VectorRecord],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 使用 embedding 向量语义搜索
    async fn search(
        &self,
        story_id: &str,
        query_embedding: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error + Send + Sync>>;

    /// 全文搜索
    async fn text_search(
        &self,
        story_id: &str,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error + Send + Sync>>;

    /// 混合搜索
    async fn hybrid_search(
        &self,
        story_id: &str,
        query: &str,
        query_embedding: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error + Send + Sync>>;

    /// 按 id 删除记录
    async fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 按 chapter_id 删除记录
    async fn delete_chapter(
        &self,
        chapter_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 返回向量表记录数
    async fn count(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;
}
