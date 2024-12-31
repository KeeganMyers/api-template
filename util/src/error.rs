use deadpool_redis::ConfigError;
use redis::RedisError;
use sqlx::error::{Error as SqlxError, ErrorKind as SqlxErrorKind};
use thiserror::Error as ThisError;

impl From<SqlxError> for UtilError {
    fn from(database_error: SqlxError) -> Self {
        match &database_error {
            SqlxError::Database(db_error) => match db_error.kind() {
                SqlxErrorKind::UniqueViolation => {
                    UtilError::SqlDuplicateRecord(db_error.message().to_owned())
                }
                SqlxErrorKind::ForeignKeyViolation => {
                    UtilError::SqlRelationMissing(db_error.message().to_owned())
                }
                SqlxErrorKind::NotNullViolation => {
                    UtilError::SqlNotNullViolation(db_error.message().to_owned())
                }
                SqlxErrorKind::CheckViolation => {
                    UtilError::SqlCheckFailed(db_error.message().to_owned())
                }
                _ => UtilError::SqlError(format!("{:?}", database_error)),
            },
            SqlxError::RowNotFound => Self::SqlFailedToFindRecord,
            SqlxError::TypeNotFound { type_name } => {
                Self::SqlFailedToFindType(type_name.to_owned())
            }
            SqlxError::ColumnNotFound(s) => Self::SqlFailedToFindColumn(s.to_owned()),
            _ => UtilError::SqlError(format!("{:?}", database_error)),
        }
    }
}

#[derive(Debug, ThisError)]
pub enum UtilError {
    #[error("SQL transaction failed {0}")]
    SqlError(String),
    #[error(transparent)]
    SqlMigrationError(#[from] sqlx::migrate::MigrateError),
    #[error("Duplicate record found {0}")]
    SqlDuplicateRecord(String),
    #[error("Required relationship between records violated {0}")]
    SqlRelationMissing(String),
    #[error("Required field missing or null {0}")]
    SqlNotNullViolation(String),
    #[error("Database logic check failed {0}")]
    SqlCheckFailed(String),
    #[error("Record not found")]
    SqlFailedToFindRecord,
    #[error(
        "Env var for Redis not set, host (for single instance) or hosts(for cluster) must be set"
    )]
    RedisNotConfigured,
    #[error("Column specified in query not found {0}")]
    SqlFailedToFindColumn(String),
    #[error("Type specified in query not found {0}")]
    SqlFailedToFindType(String),
    #[error(transparent)]
    EnvyError(#[from] envy::Error),
    #[error(transparent)]
    DeadpoolRedis(#[from] deadpool::managed::CreatePoolError<ConfigError>),
    #[error(transparent)]
    DeadpoolCluserRedis(#[from] deadpool::managed::PoolError<RedisError>),
    #[error(transparent)]
    RedisError(#[from] RedisError),
    #[error("Redis stream params could not be converted into Vec<String>")]
    RedisStreamParams,
}
