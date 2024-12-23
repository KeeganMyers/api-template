use crate::{env::PostgresConfig, error::UtilError, AppConfig};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool, Postgres, QueryBuilder};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Default)]
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
            "SELECT {} FROM {} ",
            Self::select_fields_str(),
            Self::table_name()
        )
    }
    fn base_update() -> String {
        format!("UPDATE {} SET ", Self::table_name())
    }

    fn base_insert() -> String {
        format!(
            "INSERT INTO {} ({}) VALUES ",
            Self::table_name(),
            Self::fields_str()
        )
    }
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

    async fn query<Q>(query: Q, db: RODB) -> Result<PaginatedResult<Self>, UtilError>
    where
        Q: ToSqlQuery + Pagination + ToSqlSort;

    fn paginated_result<Q>(data: Vec<Self>, query: Q) -> Result<PaginatedResult<Self>, UtilError>
    where
        Q: ToSqlQuery + Pagination + ToSqlSort,
    {
        Ok(PaginatedResult {
            page: query.page(),
            limit: query.limit(),
            total: 0,
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
    /*
    fn direction(&self) -> String {
        self.direction
            .as_ref()
            .and_then(|s| serde_json::to_string(&s).ok())
            .unwrap_or("asc".to_owned())
    }

    fn column(&self) -> String {
        serde_json::to_string(&self.sort_by.clone().unwrap_or_default()).unwrap_or_default()
    }
    */

    fn direction(&self) -> String;
    fn column(&self) -> String;
    fn add_sort(&self, qb: &mut QueryBuilder<Postgres>) {
        qb.push(" ORDER BY ");
        qb.push_bind(self.column());
        qb.push(" ");
        qb.push_bind(self.direction());
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
