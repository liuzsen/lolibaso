use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse::Parse, spanned::Spanned};

pub struct HttpRequest {
    type_name: syn::Ident,
    body: Option<(syn::Ident, syn::Type)>,
    query: Option<(syn::Ident, syn::Type)>,
}

impl Parse for HttpRequest {
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
        let mut query = None;
        for field in fields {
            let Some(ident) = field.ident else {
                abort!(field.span(), "expected field with ident");
            };

            let ty = field.ty;
            match ident.to_string().as_str() {
                "body" => body = Some((ident, ty)),
                "query" => query = Some((ident, ty)),
                _ => {
                    abort!(
                        ident.span(),
                        "Unknown field: {}. Expected fields: body",
                        ident
                    );
                }
            }
        }

        Ok(HttpRequest {
            type_name: input.ident,
            body,
            query,
        })
    }
}

impl HttpRequest {
    pub fn expand(&self) -> TokenStream {
        let type_name = &self.type_name;
        let parse_body = self.parse_body();
        let assign_body = self.body.is_some().then(|| quote!(body,));
        let parse_query = self.parse_query();
        let assign_query = self.query.is_some().then(|| quote!(query,));

        let unit = syn::parse_quote!(());
        let body_ty = self.body.as_ref().map(|(_, ty)| ty).unwrap_or(&unit);
        let query_ty = self.query.as_ref().map(|(_, ty)| ty).unwrap_or(&unit);

        let output = quote_spanned! { self.type_name.span() =>
                const _: () =  {
                    use lolibaso::http::{
                        adapter::{FromHttpRequest, HttpRequestModel},
                        error::BizError,
                        request::HttpRequest,
                    };

                    impl lolibaso::http::adapter::HttpRequestModel for #type_name {
                        type Query = #query_ty;

                        type Body = #body_ty;
                    }

                    impl<'a> FromHttpRequest<'a> for #type_name {
                        fn from_http_req<R, P, F>(req: &'a R, parser: P) -> Result<Self, BizError>
                        where
                            R: HttpRequest,
                            F: lolibaso::http::parser::Format,
                            P: lolibaso::http::parser::Parser<
                                    'a,
                                    <Self as HttpRequestModel>::Query,
                                    lolibaso::http::parser::UrlEncodedQuery,
                                >,
                            P: lolibaso::http::parser::Parser<'a, <Self as HttpRequestModel>::Body, F>
                        {
                            #parse_body
                            #parse_query

                            Ok(Request { #assign_body #assign_query })
                        }
                    }
                };
        };

        output
    }

    fn parse_query(&self) -> Option<TokenStream> {
        let (_query_ident, query_ty) = self.query.as_ref()?;
        let output = quote! {
            let query = req.uri().query().unwrap_or_default();
            let query: #query_ty = match parser.parse(query.as_bytes())
            {
                Ok(q) => q,
                Err(e) => match e {
                    lolibaso::http::parser::ParseError::Custom { err_name, err_msg } => {
                        let err = BizError::try_from_name(&err_name, err_msg.as_ref());
                        return Err(err.unwrap_or_else(|| {
                            tracing::warn!(
                                "Cannot convert custom error `{err_name}` to BizError in Query"
                            );
                            match err_msg {
                                Some(msg) => BizError::InvalidQuery.with_context(msg),
                                None => BizError::InvalidQuery.with_context(err_name),
                            }
                        }));
                    }
                    lolibaso::http::parser::ParseError::BizErr(biz_error) => {
                        return Err(BizError::InvalidQuery.with_context(biz_error.to_string()));
                    }
                },
            };
        };

        Some(output)
    }

    fn parse_body(&self) -> Option<TokenStream> {
        let (_body_ident, body_ty) = self.body.as_ref()?;
        let output = quote! {
            let body = req.body().unwrap_or(b"{}");
            let body: #body_ty = match parser.parse(body) {
                Ok(body) => body,
                Err(e) => match e {
                    lolibaso::http::parser::ParseError::Custom { err_name, err_msg } => {
                        let err = BizError::try_from_name(&err_name, err_msg.as_ref());
                        return Err(err.unwrap_or_else(|| {
                            tracing::warn!(
                                "Cannot convert custom error `{err_name}` to BizError in Body"
                            );
                            match err_msg {
                                Some(msg) => BizError::InvalidRequestBody.with_context(msg),
                                None => BizError::InvalidRequestBody.with_context(err_name),
                            }
                        }));
                    }
                    lolibaso::http::parser::ParseError::BizErr(biz_error) => {
                        return Err(
                            BizError::InvalidRequestBody.with_context(biz_error.to_string())
                        );
                    }
                },
            };

        };

        Some(output)
    }
}
