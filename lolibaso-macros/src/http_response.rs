use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_quote, spanned::Spanned};

pub struct HttpResponse {
    type_name: syn::Ident,
    body: Option<(syn::Ident, syn::Type)>,
}

impl Parse for HttpResponse {
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

        Ok(Self {
            type_name: input.ident,
            body,
        })
    }
}

impl HttpResponse {
    pub fn expand(&self) -> TokenStream {
        let type_name = &self.type_name;
        let body_ty = self
            .body
            .as_ref()
            .map(|(_, ty)| ty.clone())
            .unwrap_or_else(|| parse_quote!(()));
        let self_body = self
            .body
            .as_ref()
            .map(|(ident, _)| quote!(self.#ident))
            .unwrap_or_else(|| quote!(()));

        quote!(impl lolibaso::http::ApiResponse for #type_name {
            type Body = #body_ty;

            fn headers(&self) -> Option<&reqwest::header::HeaderMap<reqwest::header::HeaderValue>> {
                None
            }

            fn body(&self) -> &Self::Body {
                &#self_body
            }

            fn into_parts(self) -> (lolibaso::http::response::Head, Self::Body) {
                let head = lolibaso::http::response::Head {
                    status: self.status(),
                    version: self.version(),
                    headers: None,
                };
                (head, #self_body)
            }
        })
    }
}
