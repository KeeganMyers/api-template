use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(LogAndParse, attributes(log_path))]
pub fn derive_error_response(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let ident = input.ident;

    match input.data {
        Data::Union(_) => panic!("cannot derive LogAndParse for unions"),
        Data::Struct(_struct_data) => derive_trait_for_struct(ident).into(),
        Data::Enum(_) => panic!("cannot derive ErrorResponse for enums"),
    }
}

fn derive_trait_for_struct(ident: Ident) -> TokenStream {
    quote! {
        #[async_trait::async_trait]
        impl util::prelude::LogAndParse for #ident {
            type ReturnType = #ident;

            async fn log_and_parse(response: reqwest::Response) -> Result<Self::ReturnType,util::error::AppError> {
                if !response.status().is_success() || response.status().is_client_error() {
                    return Err(util::error::AppError::Other(anyhow::Error::msg(format!(
                        "{} Request failed with status {:?}",
                        stringify!(#ident),response.status()
                    ))));
                }

                let response_text = response.text().await?;
                //If this fails we need to see the original json the serde error is not detailed enough
                match serde_json::from_str::<Self>(&response_text) {
                    Ok(log_result) => {
                        log::trace!("{} Parse original json {}", stringify!(#ident),response_text);
                        Ok(log_result)
                    },
                    Err(e) => {
                        error!("{} Parse failed {:?} original json {}",stringify!(#ident), e, response_text);
                        Err(util::error::AppError::from(e))
                    },
                }
            }
        }
    }
}
