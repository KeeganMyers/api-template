pub mod error;
pub mod user;
pub mod user_permission;
pub mod user_readmodel;

use crate::{error::ModelError, user_readmodel::UserReadModel};
use broker::{BrokerLayer, RedisStream, Subscriber};
use serde::{Deserialize, Serialize};
use util::{
    env::Env,
    store::{CacheLayer, Pagination, Redis, RODB, RWDB},
    AppState, JsonNum,
};
use utoipa::ToSchema;

#[derive(Clone)]
pub struct State {
    pub rw_db: RWDB,
    pub ro_db: RODB,
    pub env: Env,
    pub cache: Redis,
    pub broker: Option<RedisStream>,
}

impl AppState for State {
    type StateType = State;
    type ErrorType = ModelError;

    async fn from_env(env: Env) -> Result<Self::StateType, Self::ErrorType> {
        Ok(Self {
            rw_db: RWDB::connect(&env).await?,
            ro_db: RODB::connect(&env).await?,
            cache: Redis::new(&env).await?,
            broker: Some(RedisStream::new(&env).await?),
            env,
        })
    }

    fn get_rw_store(&self) -> &RWDB {
        &self.rw_db
    }
    fn get_ro_store(&self) -> &RODB {
        &self.ro_db
    }
    fn get_env(&self) -> &Env {
        &self.env
    }

    fn cache(&self) -> Option<&impl CacheLayer> {
        Some(&self.cache)
    }
}

pub fn subscribers() -> Vec<impl Subscriber> {
    let subs = vec![UserReadModel::default()];
    subs
}

#[derive(Serialize, PartialEq, Deserialize, ToSchema, Clone, Debug, Default)]
pub struct Paging {
    pub page: Option<JsonNum>,
    pub limit: Option<JsonNum>,
    pub offset: Option<JsonNum>,
}

impl Pagination for Paging {
    fn page(&self) -> i64 {
        match &self.page {
            Some(JsonNum::I(i)) => *i,
            Some(JsonNum::S(i_str)) => i_str.parse::<i64>().unwrap_or(1),
            _ => 1,
        }
    }

    fn limit(&self) -> i64 {
        match &self.limit {
            Some(JsonNum::I(i)) => *i,
            Some(JsonNum::S(i_str)) => i_str.parse::<i64>().unwrap_or(1),
            _ => 100,
        }
    }

    fn offset(&self) -> i64 {
        match &self.offset {
            Some(JsonNum::I(i)) => *i,
            Some(JsonNum::S(i_str)) => i_str.parse::<i64>().unwrap_or(1),
            _ => {
                let page = if self.page() == 0 {
                    self.page()
                } else {
                    (self.page() - 1).abs()
                };

                self.limit() * page
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use derive_model::Model;
    use derive_query::Query;
    use to_params::{FromParams, ToParams};
    use util::{
        error::UtilError,
        macros::make_sort,
        store::{NewModel, PaginatedResult, UpdateModel, RODB, RWDB},
        FromParams, ToParams,
    };

    #[derive(sqlx::FromRow, Model)]
    #[allow(dead_code)]
    #[model(table_name = "test_tbl")]
    pub struct TestModel {
        pub test: String,
        #[model(col_name = "db_col_name")]
        pub test2: String,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, ToSchema, Default, Clone)]
    pub enum SortColumn {
        #[default]
        Test,
    }

    #[derive(
        Debug,
        PartialEq,
        Serialize,
        Deserialize,
        ToSchema,
        Default,
        Clone,
        Query,
        ToParams,
        FromParams,
    )]
    pub struct Query {
        test: Option<String>,
        test2: String,
        #[serde(flatten)]
        sort: Option<TestSort>,
        #[serde(flatten)]
        paging: Option<Paging>,
    }

    make_sort!(TestSort, SortColumn);

    #[test]
    fn builds_valid_sql() {
        assert_eq!(
            TestModel::fields(),
            vec!["test".to_string(), "db_col_name".to_string()]
        );
        assert_eq!(
            TestModel::base_select(),
            "SELECT test,db_col_name AS test2,CAST(COUNT(*) OVER() AS BigInt) AS total FROM test_tbl ".to_string()
        );
    }

    #[test]
    fn builds_valid_sql_with_query() {
        let query = Query {
            test: Some("some string".to_string()),
            test2: "some string".to_string(),
            ..Query::default()
        };

        assert_eq!(TestModel::build_query(&query).sql(), r#"SELECT test,db_col_name AS test2,CAST(COUNT(*) OVER() AS BigInt) AS total FROM test_tbl  WHERE test = $1 AND test2 = $2 ORDER BY "Test" asc FETCH NEXT $3 ROWS ONLY OFFSET $4"#.to_string());
    }

    #[derive(Debug, Default, PartialEq, ToParams, FromParams)]
    pub struct Query2 {
        test: String,
    }
    #[test]
    fn builds_redis_param_vec() {
        let query = Query2 {
            test: "some string".to_string(),
        };
        let query_vec = query.to_params();
        assert_eq!(Query2::from_params(query_vec), query);
    }
}
