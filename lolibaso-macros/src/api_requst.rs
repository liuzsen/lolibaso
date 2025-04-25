use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, spanned::Spanned};

pub struct Request {
    type_name: syn::Ident,
    body: Option<(syn::Ident, syn::Type)>,
}

impl Parse for Request {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let input = syn::DeriveInput::parse(input)?;
        let span = input.span();
        let data = match input.data {
            syn::Data::Struct(data_struct) => data_struct,
            _ => {
                abort!(span, "expected struct");
            }
        };
        let fields = match data.fields {
            syn::Fields::Named(fields_named) => fields_named.named,
            _ => {
                abort!(span, "expected struct with named fields");
            }
        };
        let mut body = None;
        for field in fields {
            let Some(ident) = field.ident else {
                abort!(field.span(), "expected field with ident");
            };

            let ty = field.ty;
            match ident.to_string().as_str() {
                "body" => {
                    body = Some((ident, ty));
                }
                _ => {
                    abort!(
                        ident.span(),
                        "Unknown field: {}. Expected fields: body",
                        ident
                    );
                }
            }
        }

        Ok(Request {
            type_name: input.ident,
            body,
        })
    }
}

impl Request {
    pub fn expand(&self) -> TokenStream {
        let type_name = &self.type_name;
        let parse_body = self.parse_body();
        let assign_body = self.body.is_some().then(|| quote!(body));
        let output = quote! {
                const _: () =  {
                    use lolibaso::{
                        http::error::BizError, http::json::JsonParser, http::request::HttpRequest,
                    };
                    use crate::{adapters::api_http::ErrorKind};

                    impl #type_name {
                        fn from_http<R, P>(request: R, json_parser: P) -> Result<Self, BizError>
                        where
                            R: HttpRequest,
                            P: JsonParser,
                        {
                            #parse_body

                            Ok(Request { #assign_body })
                        }
                    }
                };
        };

        output
    }

    fn parse_body(&self) -> Option<TokenStream> {
        let (_body_ident, body_ty) = self.body.as_ref()?;
        let output = quote! {
            let body = request.body().unwrap_or(b"{}");
            let body: #body_ty = match json_parser.parse(body) {
                Ok(body) => body,
                Err(e) => match e {
                    lolibaso::http::json::JsonError::Custom { err_name, err_msg } => {
                        let err = ErrorKind::try_from_name(&err_name, err_msg.as_ref());
                        return Err(err.unwrap_or_else(|| match err_msg {
                            Some(msg) => BizError::InvalidJsonBody.with_context(msg),
                            None => BizError::InvalidJsonBody,
                        }));
                    }
                    lolibaso::http::json::JsonError::InvalidJson(e) => {
                        return Err(BizError::InvalidJsonBody.with_context(e.to_string()));
                    }
                },
            };
        };

        Some(output)
    }
}
