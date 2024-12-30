pub use crate::*;

#[macro_export]
macro_rules! make_sort {
    ($name: ident, $r: ty) => {
        #[derive(Debug, PartialEq, Serialize, Deserialize, ToSchema, Default, Clone)]
        #[serde(rename_all = "camelCase")]
        pub struct $name {
            pub direction: Option<SortDirection>,
            pub sort_by: Option<$r>,
        }
    };
}

#[macro_export]
macro_rules! redis_op {
    ($p: ident,$r: expr) => {
        match &$p.pool {
            ConnectionPool::Instance(pool) => {
                let mut conn = pool.get().await?;
                $r.query_async(&mut conn).await.map_err(UtilError::from)
            }
            ConnectionPool::Cluster(pool) => {
                let mut conn = pool.get().await?;
                $r.query_async(&mut conn).await.map_err(UtilError::from)
            }
        }
    };
    ($p: ident,$r: expr,$t: ty) => {
        match &$p.pool {
            ConnectionPool::Instance(pool) => {
                let mut conn = pool.get().await?;
                $r.query_async::<$t>(&mut conn)
                    .await
                    .map_err(UtilError::from)
            }
            ConnectionPool::Cluster(pool) => {
                let mut conn = pool.get().await?;
                $r.query_async::<$t>(&mut conn)
                    .await
                    .map_err(UtilError::from)
            }
        }
    };
}
