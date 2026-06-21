//! Database connection pool port

use crate::db::DbPool;
use crate::error::AppError;

/// 数据库连接池端口
///
/// 对 r2d2 SQLite 连接池的 thin 抽象，使业务模块不再依赖全局单例。
pub trait ConnectionPool: Send + Sync + 'static {
    /// 获取一个数据库连接
    fn get(&self)
        -> Result<r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>, AppError>;
}

impl ConnectionPool for DbPool {
    fn get(
        &self,
    ) -> Result<r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>, AppError> {
        self.get().map_err(AppError::from)
    }
}
