use proc_macro::TokenStream;

mod api_requst;
mod api_response;
mod err_enum;
mod get_config;
mod init;
mod provider;

#[proc_macro_derive(Provider, attributes(provider))]
pub fn derive_provider(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    provider::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn BizErrEnum(_args: TokenStream, item: TokenStream) -> TokenStream {
    let entity = syn::parse_macro_input!(item as err_enum::ErrEnum);
    let output = entity
        .expand()
        .unwrap_or_else(syn::Error::into_compile_error);

    let stream: TokenStream = output.into();

    stream
}

#[proc_macro_derive(ApiRequest, attributes(request))]
pub fn derive_api_request(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as api_requst::Request);
    input.expand().into()
}

#[proc_macro_derive(ApiResponse, attributes(response))]
pub fn derive_api_response(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as api_response::ApiResponse);
    input.expand().into()
}

#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn InitFunction(args: TokenStream, item: TokenStream) -> TokenStream {
    let entity = syn::parse_macro_input!(item as init::InitMethod);
    let ext_fields = syn::parse_macro_input!(args as init::ExtFields);
    let output = entity.expand(ext_fields).unwrap_or_else(|err| {
        let err = err.to_compile_error();
        quote::quote! {
            #err
        }
    });

    let stream: TokenStream = output.into();

    stream
}

#[proc_macro_derive(GetConfig)]
pub fn derive_get_config(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as get_config::GetConfig);
    input
        .expand()
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
