use crate::{
    env::{Env, PostgresConfig},
    error::UtilError,
    macros::redis_op,
    AppConfig,
};
use deadpool_redis::{
    cluster::{Config as ClusterConfig, Pool as RedisClusterPool, Runtime},
    redis::aio::ConnectionLike,
    redis::cmd,
    Config as InstanceConfig, Pool as RedisInstancePool,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool, Postgres, QueryBuilder};
use utoipa::ToSchema;

#[derive(Debug, PartialEq, Serialize, Deserialize, ToSchema, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

pub trait RWStore {}
pub trait ROStore {}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Default)]
pub struct PaginatedResult<T> {
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    pub data: Vec<T>,
}

pub trait NewModel {
    fn add_column_names(&self, qb: &mut QueryBuilder<Postgres>);
    fn add_column_values(&self, qb: &mut QueryBuilder<Postgres>);
}

pub trait UpdateModel {
    fn add_columns(&self, qb: &mut QueryBuilder<Postgres>);
}

#[allow(async_fn_in_trait)]
pub trait Model: Sized + Send {
    fn table_name() -> String;
    fn fields() -> Vec<String>;
    fn fields_str() -> String {
        Self::fields().join(",")
    }
    fn select_fields() -> Vec<String>;
    fn select_fields_str() -> String {
        Self::select_fields().join(",")
    }
    fn base_select() -> String {
        format!(
            "SELECT {},CAST(COUNT(*) OVER() AS BigInt) AS total FROM {} ",
            Self::select_fields_str(),
            Self::table_name()
        )
    }

    async fn update<Q>(
        query: &Q,
        updated_model: impl UpdateModel,
        db: &RWDB,
    ) -> Result<Self, UtilError>
    where
        Q: ToSqlQuery + Pagination + ToSqlSort;
    async fn insert(new_model: impl NewModel, db: &RWDB) -> Result<Self, UtilError>;
    async fn upsert(new_model: impl NewModel + UpdateModel, db: &RWDB) -> Result<Self, UtilError>;
    fn build_query<Q>(query: &Q) -> QueryBuilder<'static, Postgres>
    where
        Q: ToSqlQuery + Pagination + ToSqlSort,
    {
        let mut qb = QueryBuilder::new(Self::base_select());
        query.add_where(&mut qb);
        query.add_sort(&mut qb);
        query.add_paging(&mut qb);
        qb
    }

    fn build_query_from_base<Q>(query: &Q, base_query: &str) -> QueryBuilder<'static, Postgres>
    where
        Q: ToSqlQuery + Pagination + ToSqlSort,
    {
        let mut qb = QueryBuilder::new(base_query);
        query.add_where(&mut qb);
        query.add_sort(&mut qb);
        query.add_paging(&mut qb);
        qb
    }

    async fn query<Q>(
        query: Q,
        query_str: Option<String>,
        db: &RODB,
    ) -> Result<PaginatedResult<Self>, UtilError>
    where
        Q: ToSqlQuery + Pagination + ToSqlSort;

    async fn execute<Q>(query: Q, query_str: &str, db: &RWDB) -> Result<(), UtilError>
    where
        Q: ToSqlQuery + Pagination + ToSqlSort;

    fn paginated_result<Q>(
        data: Vec<Self>,
        total: Option<i64>,
        query: Q,
    ) -> Result<PaginatedResult<Self>, UtilError>
    where
        Q: ToSqlQuery + Pagination + ToSqlSort,
    {
        Ok(PaginatedResult {
            page: query.page(),
            limit: query.limit(),
            total: total.unwrap_or_default(),
            data,
        })
    }
}

pub trait Pagination {
    fn limit(&self) -> i64;
    fn page(&self) -> i64;
    fn offset(&self) -> i64;
    fn add_paging(&self, qb: &mut QueryBuilder<Postgres>) {
        qb.push(" FETCH NEXT ");
        qb.push_bind(self.limit());
        qb.push(" ROWS ONLY OFFSET ");
        qb.push_bind(self.offset());
    }
}

pub trait ToSqlQuery {
    fn add_where(&self, qb: &mut QueryBuilder<Postgres>);
}

pub trait ToSqlSort {
    fn direction(&self) -> String;
    fn column(&self) -> String;
    fn add_sort(&self, qb: &mut QueryBuilder<Postgres>) {
        qb.push(" ORDER BY ");
        qb.push(self.column());
        qb.push(" ");
        qb.push(self.direction());
    }
}

#[derive(Clone)]
pub struct RWDB(PgPool);
#[derive(Clone)]
pub struct RODB(PgPool);

impl RODB {
    pub fn get_conn(&self) -> &PgPool {
        &self.0
    }

    pub fn connect_str(env: &PostgresConfig) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            env.ro_username, env.password, env.host, env.port, env.db_name
        )
    }

    pub async fn connect(state: &impl AppConfig) -> Result<Self, UtilError> {
        let connection = Self::connect_str(state.get_rw_store_settings());
        let pool = PgPoolOptions::new()
            .min_connections(5)
            .connect(&connection)
            .await?;
        Ok(Self(pool))
    }
}

impl RWDB {
    pub fn get_conn(&self) -> &PgPool {
        &self.0
    }

    pub fn connect_str(env: &PostgresConfig) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            env.username, env.password, env.host, env.port, env.db_name
        )
    }

    pub async fn connect(state: &impl AppConfig) -> Result<Self, UtilError> {
        let connection = Self::connect_str(state.get_rw_store_settings());
        let pool = PgPoolOptions::new()
            .min_connections(5)
            .connect(&connection)
            .await?;
        Ok(Self(pool))
    }

    pub async fn migrate(state: &impl AppConfig) -> Result<(), UtilError> {
        let pool = Self::connect(state).await?;
        sqlx::migrate!("../migrations/sql").run(&pool.0).await?;
        Ok(())
    }
}

#[allow(async_fn_in_trait)]
pub trait CacheLayer: Sized {
    async fn get_conn_pool(&self) -> &ConnectionPool;
    async fn new(env: &Env) -> Result<Self, UtilError>;
    async fn set_value(
        &self,
        key: &str,
        value: &str,
        expires: Option<u64>,
    ) -> Result<(), UtilError>;
    async fn delete_value(&self, key: &str) -> Result<(), UtilError>;
    async fn get_value(&self, key: &str) -> Result<String, UtilError>;
    async fn value_exists(&self, key: &str) -> Result<bool, UtilError>;
}

#[derive(Clone)]
pub struct Redis {
    pub pool: ConnectionPool,
}

#[derive(Clone)]
pub enum ConnectionPool {
    Cluster(RedisClusterPool),
    Instance(RedisInstancePool),
}

impl ConnectionPool {
    pub async fn get_instance(&self) -> Result<Option<impl ConnectionLike>, UtilError> {
        if let Self::Instance(pool) = self {
            return Ok(Some(pool.get().await?));
        }
        Ok(None)
    }

    pub async fn get_cluster(&self) -> Result<Option<impl ConnectionLike>, UtilError> {
        if let Self::Cluster(pool) = self {
            return Ok(Some(pool.get().await?));
        }
        Ok(None)
    }
}

impl CacheLayer for Redis {
    async fn get_conn_pool(&self) -> &ConnectionPool {
        &self.pool
    }

    async fn new(env: &Env) -> Result<Self, UtilError> {
        let redis_env = env.redis.as_ref().expect("Env var for Redis not set, host (for single instance) or hosts(for cluster) must be set");
        if let Some(url) = &redis_env.host {
            let url_prefix = if redis_env.insecure == Some("true".to_string()) {
                "redis"
            } else {
                "rediss"
            };
            let con_str = format!(
                "{}://{}:{}",
                url_prefix,
                url,
                redis_env.port.clone().unwrap_or("6379".to_owned())
            );
            let cfg = InstanceConfig::from_url(con_str);
            let pool = cfg
                .create_pool(Some(Runtime::Tokio1))
                .map_err(UtilError::from)?;
            return Ok(Self {
                pool: ConnectionPool::Instance(pool),
            });
        }

        if let Some(url) = &redis_env.hosts {
            let urls = url
                .split(',')
                .map(|s| s.to_owned())
                .collect::<Vec<String>>();
            let cfg = ClusterConfig::from_urls(urls);
            let pool = cfg
                .create_pool(Some(Runtime::Tokio1))
                .map_err(UtilError::from)?;
            return Ok(Self {
                pool: ConnectionPool::Cluster(pool),
            });
        }

        Err(UtilError::RedisNotConfigured)
    }

    async fn set_value(
        &self,
        key: &str,
        value: &str,
        expires: Option<u64>,
    ) -> Result<(), UtilError> {
        let mut args_vec: Vec<String> = vec![key.into(), value.into()];
        if let Some(expires) = expires {
            args_vec.append(&mut vec!["EX".into(), expires.to_string()])
        };
        redis_op!(self, cmd("SET").arg(&args_vec))
    }

    async fn delete_value(&self, key: &str) -> Result<(), UtilError> {
        redis_op!(self, cmd("DEL").arg(&[key]))
    }

    async fn get_value(&self, key: &str) -> Result<String, UtilError> {
        redis_op!(self, cmd("get").arg(&[key]))
    }

    async fn value_exists(&self, key: &str) -> Result<bool, UtilError> {
        let result: usize = redis_op!(self, cmd("EXISTS").arg(&[key]))?;
        Ok(result == 1)
    }
}
