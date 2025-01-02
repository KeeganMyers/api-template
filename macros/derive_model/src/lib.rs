use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, LitStr};

#[proc_macro_derive(Model, attributes(model))]
pub fn derive_model(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let ident = input.ident.clone();

    match &input.data {
        Data::Union(_) => panic!("cannot derive Model for unions"),
        Data::Struct(data) => derive_model_struct(ident, &input, data).into(),
        Data::Enum(_) => panic!("cannot derive Model for enums"),
    }
}

fn derive_model_struct(ident: Ident, input: &DeriveInput, struct_data: &DataStruct) -> TokenStream {
    let mut table_name: String = String::from("");
    let mut fields: Vec<String> = vec![];
    let mut select_fields: Vec<String> = vec![];

    for attr in &input.attrs {
        if attr.path().is_ident("model") {
            let _ = attr.parse_nested_meta(|meta| {
                //#[model(table_name = T)]
                if meta.path.is_ident("table_name") {
                    let value = meta.value()?;
                    let lit: LitStr = value.parse()?;
                    table_name = lit.value();
                    return Ok(());
                }
                Err(meta.error("unrecognized attribute"))
            });
        }
    }
    if table_name.is_empty() {
        panic!("Model `table_name` must be set");
    }

    for field in &struct_data.fields {
        if let Some(attr) = field.attrs.iter().find(|f| f.path().is_ident("model")) {
            let _ = attr.parse_nested_meta(|meta| {
                //#[model(db_col_name = T)]
                if meta.path.is_ident("col_name") {
                    let value = meta.value()?;
                    let lit: LitStr = value.parse()?;

                    if let Some(field_ident) = &field.ident {
                        select_fields.push(format!("{} AS {}", lit.value(), field_ident));
                    }
                    fields.push(lit.value());
                    return Ok(());
                }
                Err(meta.error("unrecognized attribute"))
            });
            continue;
        }

        if let Some(field_ident) = &field.ident {
            fields.push(field_ident.to_string());
            select_fields.push(field_ident.to_string())
        }
    }

    quote! {
        use sqlx::{Postgres, FromRow,Row,QueryBuilder};
        use util::store::{Model, Pagination, SortDirection, ToSqlQuery, ToSqlSort};

        impl Model for #ident {
            fn fields() -> Vec<String> {
                vec![#(#fields.to_owned()),*]
            }

            fn select_fields() -> Vec<String> {
                vec![#(#select_fields.to_owned()),*]
            }

            fn table_name() -> String {
                #table_name.to_owned()
            }

            async fn execute<Q>(
                query: Q,
                query_str: &str,
                db: &RWDB,
            ) -> Result<(), UtilError>
            where
                Q: ToSqlQuery + Pagination + ToSqlSort {
                    let mut qb = QueryBuilder::new(query_str);
                    query.add_where(&mut qb);
                    let built_query = qb.build();
                    let _ = built_query
                        .execute(db.get_conn())
                        .await
                        .map_err(UtilError::from)?;
                    Ok(())
                }

            async fn query<Q>(query: Q,query_str: Option<String>,db: &RODB) -> Result<PaginatedResult<Self>,UtilError>
                where Q: ToSqlQuery + Pagination + ToSqlSort
            {
                let mut qb =  if let Some(qs) = query_str {
                    Self::build_query_from_base(&query,&qs)
                } else {
                    Self::build_query(&query)
                };
                log::trace!(" SQL generated {:?}", qb.sql());
                let built_query = qb.build();
                let data = built_query
                    .fetch_all(db.get_conn())
                    .await
                    .map_err(UtilError::from)?;
                let total: Option<i64> = data.iter().next().and_then(|r| r.try_get("total").ok());
                let records = data
                    .iter()
                    .map(|r| {
                         let result = Self::from_row(r);
                         if let Err(e) = &result {
                            log::error!("Failed to parse model from sql {:?}",e)
                         }
                         result
                        })
                    .filter_map(|r| r.ok())
                    .collect::<Vec<Self>>();
                Self::paginated_result(records,total,query)
            }

            async fn update<Q>(query: &Q,updated_model: impl UpdateModel,db: &RWDB) -> Result<Self,UtilError>
            where
                Q: ToSqlQuery + Pagination + ToSqlSort,
            {
                let mut qb = QueryBuilder::new(
                format!(
                    "UPDATE {} SET ",
                    Self::table_name(),
                ));
                updated_model.add_columns(&mut qb);
                query.add_where(&mut qb);
                qb.push(format!(" RETURNING {}",Self::select_fields_str()));

                log::trace!("UPDATE SQL generated {:?}", qb.sql());
                let built_query = qb.build();
                let data = built_query
                    .fetch_one(db.get_conn())
                    .await
                    .map_err(UtilError::from)?;
                Self::from_row(&data).map_err(UtilError::from)
            }

            async fn insert(new_model: impl NewModel,db: &RWDB) -> Result<Self,UtilError> {
                let mut qb = QueryBuilder::new(
                format!(
                    "INSERT INTO {} ",
                    Self::table_name(),
                ));
                new_model.add_column_names(&mut qb);
                qb.push(" VALUES ");
                new_model.add_column_values(&mut qb);
                qb.push(format!(" RETURNING {}",Self::select_fields_str()));
                log::trace!("Insert SQL generated {:?}", qb.sql());
                let built_query = qb.build();
                let data = built_query
                    .fetch_one(db.get_conn())
                    .await
                    .map_err(UtilError::from)?;
                Self::from_row(&data).map_err(UtilError::from)
            }

            async fn upsert(new_model: impl NewModel + UpdateModel,db: &RWDB) -> Result<Self,UtilError> {
                let mut qb = QueryBuilder::new(
                format!(
                    "INSERT INTO {} ",
                    Self::table_name(),
                ));
                new_model.add_column_names(&mut qb);
                qb.push(" VALUES ");
                new_model.add_column_values(&mut qb);
                qb.push("ON CONFLICT (id) DO UPDATE SET ");
                new_model.add_columns(&mut qb);

                qb.push(format!(" RETURNING {}",Self::select_fields_str()));
                log::trace!("Upsert SQL generated {:?}", qb.sql());
                let built_query = qb.build();
                let data = built_query
                    .fetch_one(db.get_conn())
                    .await
                    .map_err(UtilError::from)?;
                Self::from_row(&data).map_err(UtilError::from)
            }
        }
    }
}
