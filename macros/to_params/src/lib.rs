#![recursion_limit = "128"]
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;

use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, Type};

use std::collections::BTreeMap;

#[proc_macro_derive(FromParams)]
pub fn from_params(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    // parse out all the field names in the struct as `Ident`s
    let fields = match ast.data {
        Data::Struct(st) => st.fields,
        _ => panic!("Implementation must be a struct"),
    };
    let idents: Vec<&Ident> = fields
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .collect::<Vec<&Ident>>();

    let value_indexes: Vec<usize> = (1..idents.len()).step_by(2).collect();
    // parse out all the primitive types in the struct as Idents
    let typecalls: Vec<Type> = fields
        .iter()
        .map(|field| field.ty.clone())
        .collect::<Vec<Type>>();

    let name: &Ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let tokens = quote! {
        use util::{FromParams};

        impl #impl_generics FromParams for #name #ty_generics #where_clause {

            fn from_params(mut params: Vec<String>) -> #name {
                let mut result_struct = #name::default();
                #(

                    let value = match serde_json::from_str::<#typecalls>(&params[#value_indexes]) {
                        Ok(val) => val,
                        _ => panic!("Cannot parse out param entry")
                    };

                    result_struct.#idents = value;
                )*
                result_struct
            }
        }
    };
    TokenStream::from(tokens)
}

#[proc_macro_derive(ToParams)]
pub fn to_params(input_struct: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input_struct as DeriveInput);

    let fields = match ast.data {
        Data::Struct(st) => st.fields,
        _ => panic!("Implementation must be a struct"),
    };
    let idents: Vec<&Ident> = fields
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .collect::<Vec<&Ident>>();

    let keys: Vec<String> = idents
        .clone()
        .iter()
        .map(|ident| ident.to_string())
        .collect::<Vec<String>>();

    // get the name identifier of the struct input AST
    let name: &Ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let tokens = quote! {
        use util::{ToParams};

        impl #impl_generics ToParams for #name #ty_generics #where_clause {
            fn to_params(&self) -> Vec<String> {
                let mut params: Vec<String> = vec![];
                #(
                    params.push(#keys.to_string());
                    params.push(serde_json::to_string(&self.#idents).unwrap());
                )*
                params
            }
        }
    };
    TokenStream::from(tokens)
}
