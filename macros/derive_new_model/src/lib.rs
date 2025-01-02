use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Field, PathSegment};

#[proc_macro_derive(NewModel)]
pub fn derive_new_model(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let ident = input.ident.clone();

    match &input.data {
        Data::Union(_) => panic!("cannot derive NewModel for unions"),
        Data::Struct(data) => derive_new_model_struct(ident, data).into(),
        Data::Enum(_) => panic!("cannot derive NewModel for enums"),
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

fn derive_new_model_struct(ident: Ident, struct_data: &DataStruct) -> TokenStream {
    let mut column_name_ast = TokenStream::new();
    let mut column_val_ast = TokenStream::new();

    for field in &struct_data.fields {
        if let Some(field_ident) = &field.ident {
            if is_option_wrapped(field) {
                column_name_ast.extend(quote!(if let Some(v) = &self.#field_ident {
                    if !qb.sql().ends_with("(") {
                    qb.push(", ");
                    }
                    qb.push(stringify!(#field_ident));
                }));

                column_val_ast.extend(quote!(if let Some(v) = &self.#field_ident {
                    if !qb.sql().ends_with("(") {
                    qb.push(", ");
                    }
                    qb.push_bind(self.#field_ident.clone());
                }));
            } else {
                column_name_ast.extend(quote!(
                    if !qb.sql().ends_with("(") {
                    qb.push(", ");
                    }
                    qb.push(stringify!(#field_ident));
                ));

                column_val_ast.extend(quote!(
                    if !qb.sql().ends_with("(") {
                    qb.push(", ");
                    }
                    qb.push_bind(self.#field_ident.clone());
                ));
            }
        }
    }

    quote! {
        impl NewModel for #ident {
            fn add_column_names(&self,qb: &mut QueryBuilder<Postgres>) {
               qb.push("(");
                #column_name_ast
               qb.push(") ");
            }
            fn add_column_values(&self,qb: &mut QueryBuilder<Postgres>) {
               qb.push("(");
                #column_val_ast
               qb.push(") ");
            }
        }

    }
}
