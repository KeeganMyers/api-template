pub use crate::*;

#[macro_export]
macro_rules! make_sort {
    ($name: ident, $r: ty) => {
        #[derive(Debug, Serialize, Deserialize, ToSchema, Default, Clone)]
        #[serde(rename_all = "camelCase")]
        pub struct $name {
            pub direction: Option<SortDirection>,
            pub sort_by: Option<$r>,
        }
    };
}
