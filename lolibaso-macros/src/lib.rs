use proc_macro::TokenStream;

mod err_enum;
mod get_config;
mod http_requst;
mod http_response;
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

#[proc_macro_derive(HttpRequest, attributes(request))]
pub fn derive_http_request(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as http_requst::HttpRequest);
    input.expand().into()
}

#[proc_macro_derive(HttpResponse, attributes(response))]
pub fn derive_http_response(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as http_response::HttpResponse);
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
