use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Field, PathSegment};

#[proc_macro_derive(UpdateModel)]
pub fn derive_update_model(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let ident = input.ident.clone();

    match &input.data {
        Data::Union(_) => panic!("cannot derive UpdateModel for unions"),
        Data::Struct(data) => derive_update_model_struct(ident, data).into(),
        Data::Enum(_) => panic!("cannot derive UpdateModel for enums"),
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

fn derive_update_model_struct(ident: Ident, struct_data: &DataStruct) -> TokenStream {
    let mut columns_ast = TokenStream::new();

    for field in &struct_data.fields {
        if let Some(field_ident) = &field.ident {
            if is_option_wrapped(field) {
                columns_ast.extend(quote!(if let Some(v) = &self.#field_ident {
                    if !qb.sql().ends_with("SET ") {
                    qb.push(", ");
                    }
                    qb.push(stringify!(#field_ident));
                    qb.push(" = ");
                    qb.push_bind(self.#field_ident.clone());
                }));
            } else {
                columns_ast.extend(quote!(
                    if !qb.sql().ends_with("SET ") {
                    qb.push(", ");
                    }
                    qb.push(stringify!(#field_ident));
                    qb.push(" = ");
                    qb.push_bind(self.#field_ident.clone());
                ));
            }
        }
    }

    quote! {
        impl UpdateModel for #ident {
            fn add_columns(&self,qb: &mut QueryBuilder<Postgres>) {
                #columns_ast
            }
        }

    }
}
