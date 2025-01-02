use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Field, PathSegment};

#[proc_macro_derive(Query, attributes(query))]
pub fn derive_query(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let ident = input.ident.clone();

    match &input.data {
        Data::Union(_) => panic!("cannot derive ToSqlQuery for unions"),
        Data::Struct(data) => derive_query_struct(ident, data).into(),
        Data::Enum(_) => panic!("cannot derive ToSqlQuery for enums"),
    }
}

fn is_option_wrapped(field: &Field) -> bool {
    match field.ty {
        syn::Type::Path(ref typepath) if typepath.qself.is_none() => typepath
            .path
            .segments
            .iter()
            .any(|s| matches!(s,PathSegment { ident, .. } if ident == "Option")),
        _ => false,
    }
}

fn add_where(struct_data: &DataStruct, builder_ast: &mut TokenStream) {
    for field in &struct_data.fields {
        if let Some(field_ident) = &field.ident {
            if field_ident != "sort" && field_ident != "paging" {
                if is_option_wrapped(field) {
                    builder_ast.extend(quote!(
                    if self.#field_ident.is_some() && !qb.sql().ends_with("WHERE ") {
                        qb.push(" WHERE ");
                    }));
                } else {
                    builder_ast.extend(quote!(if !qb.sql().ends_with("WHERE ") {
                        qb.push(" WHERE ");
                    }));
                }
            }
        }
    }
}

fn derive_query_struct(ident: Ident, struct_data: &DataStruct) -> TokenStream {
    let mut builder_ast = TokenStream::new();
    let mut sort_ast = TokenStream::new();
    let mut paging_ast = TokenStream::new();
    let mut sort_present = false;
    let mut paging_present = false;

    add_where(struct_data, &mut builder_ast);

    for field in &struct_data.fields {
        if let Some(field_ident) = &field.ident {
            if field_ident != "sort" && field_ident != "paging" {
                if is_option_wrapped(field) {
                    builder_ast.extend(quote!(if let Some(v) = &self.#field_ident {
                        if !qb.sql().ends_with("WHERE ") {
                            qb.push(" AND ");
                        }
                        qb.push(format!("{} = ", stringify!(#field_ident)));
                        qb.push_bind(v.clone());
                    }));
                } else {
                    builder_ast.extend(quote!(
                        if !qb.sql().ends_with("WHERE ") {
                            qb.push(" AND ");
                        }

                        qb.push(format!("{} = ", stringify!(#field_ident)));
                        qb.push_bind(self.#field_ident.clone());
                    ));
                }
            }
            if field_ident == "sort" {
                sort_present = true
            }
            if field_ident == "paging" {
                paging_present = true
            }
        }
    }

    if sort_present {
        sort_ast.extend(quote!(
        use util::JsonNum;

        impl ToSqlSort for #ident {
                fn direction(&self) -> String {
                    self.sort.clone().unwrap_or_default().direction
                        .and_then(|s| serde_json::to_string(&s).ok())
                        .unwrap_or("asc".to_owned())
                }

                fn column(&self) -> String {
                    serde_json::to_string(&self.sort.clone().unwrap_or_default()
                                          .sort_by.clone().unwrap_or_default()).unwrap_or_default()
                }
        }
            ));
    }

    if paging_present {
        paging_ast.extend(quote!(
        impl Pagination for #ident {
            fn page(&self) -> i64 {
                match &self.paging.clone().unwrap_or_default().page {
                    Some(JsonNum::I(i)) => *i,
                    Some(JsonNum::S(i_str)) => i_str.parse::<i64>().unwrap_or(1),
                    _ => 1,
                }
            }

            fn limit(&self) -> i64 {
                match &self.paging.clone().unwrap_or_default().limit {
                    Some(JsonNum::I(i)) => *i,
                    Some(JsonNum::S(i_str)) => i_str.parse::<i64>().unwrap_or(1),
                    _ => 100,
                }
            }

            fn offset(&self) -> i64 {
                match &self.paging.clone().unwrap_or_default().offset {
                    Some(JsonNum::I(i)) => *i,
                    Some(JsonNum::S(i_str)) => i_str.parse::<i64>().unwrap_or(1),
                    _ => {
                        let page = if self.page() == 0 {
                            self.page()
                        } else {
                            (self.page() - 1).abs()
                        };

                        self.limit() * page
                    },
                }
            }
        }
        ));
    }

    quote! {
        impl ToSqlQuery for #ident {
            fn add_where(&self,qb: &mut QueryBuilder<Postgres>) {
                #builder_ast
            }
        }

        #sort_ast
        #paging_ast
    }
}
