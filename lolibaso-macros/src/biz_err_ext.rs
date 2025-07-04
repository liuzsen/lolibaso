use convert_case::{Case, Casing};
use quote::quote;
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{Ident, parse::Parse};

pub struct ErrEnum {
    vis: syn::Visibility,
    ident: Ident,
    base_biz_code: u32,
    default_http_status: Option<u16>,
    variants: Vec<ErrVariant>,
}

struct ErrVariant {
    ident: Ident,
    desc: Option<String>,
    http_status: Option<u16>,
}

impl Parse for ErrEnum {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let input = syn::DeriveInput::parse(input)?;
        let data = match input.data {
            syn::Data::Enum(data_enum) => data_enum,
            _ => {
                return Err(syn::Error::new_spanned(input, "expected enum"));
            }
        };
        let mut variants = vec![];
        for variant in data.variants {
            if variant.discriminant.is_some() {
                return Err(syn::Error::new_spanned(
                    variant,
                    "enum variant must not have a discriminant",
                ));
            }
            let ident = variant.ident;
            let mut desc = None;
            let mut http_status = None;
            for attr in &variant.attrs {
                let Some(path) = attr.path().get_ident() else {
                    continue;
                };
                match &*path.to_string() {
                    "doc" => {
                        if desc.is_some() {
                            continue;
                        }
                        desc = Some(parse_variant_doc(attr)?);
                    }
                    "http_status" => {
                        http_status = Some(parse_http_status(attr)?);
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            attr,
                            format!("Unknown attribute: {}. Expected: `http_status`", path),
                        ));
                    }
                }
            }

            variants.push(ErrVariant {
                ident,
                desc,
                http_status,
            });
        }

        let mut default_http_status = None;
        let mut base_biz_code = None;
        for attr in &input.attrs {
            let Some(path) = attr.path().get_ident() else {
                continue;
            };
            match &*path.to_string() {
                "base_biz_code" => {
                    base_biz_code = Some(parse_base_biz_code(attr)?);
                }
                "default_http_status" => {
                    default_http_status = Some(parse_http_status(attr)?);
                }
                path => {
                    return Err(syn::Error::new_spanned(
                        attr,
                        format!(
                            "unknown attribute: {}. Expected: `base_biz_code`, `err_path`, `default_http_status`",
                            path
                        ),
                    ));
                }
            }
        }

        let base_biz_code = match base_biz_code {
            Some(v) => v,
            None => {
                return Err(syn::Error::new_spanned(
                    input.ident,
                    "`base_biz_code` must be set. Use #[base_biz_code = ...] attribute",
                ));
            }
        };

        Ok(ErrEnum {
            ident: input.ident,
            base_biz_code,
            default_http_status,
            variants,
            vis: input.vis,
        })
    }
}

fn require_lit_int(expr: &syn::Expr) -> syn::Result<u32> {
    let span = expr.span();
    let lit = require_literal(expr)?;
    match lit {
        syn::Lit::Int(lit_int) => Ok(lit_int.base10_parse()?),
        _ => Err(syn::Error::new(span, "expected literal integer")),
    }
}

fn require_lit_str(expr: &syn::Expr) -> syn::Result<&syn::LitStr> {
    let span = expr.span();
    let lit = require_literal(expr)?;
    match lit {
        syn::Lit::Str(lit_str) => Ok(lit_str),
        _ => Err(syn::Error::new(span, "expected literal string")),
    }
}

fn require_literal(expr: &syn::Expr) -> syn::Result<&syn::Lit> {
    match expr {
        syn::Expr::Lit(expr_lit) => Ok(&expr_lit.lit),
        _ => {
            return Err(syn::Error::new_spanned(expr, "expected literal"));
        }
    }
}

fn parse_http_status(attr: &syn::Attribute) -> syn::Result<u16> {
    let named = attr.meta.require_name_value()?;
    let lit = require_lit_int(&named.value)?;

    Ok(lit as u16)
}

fn parse_base_biz_code(attr: &syn::Attribute) -> syn::Result<u32> {
    let named = attr.meta.require_name_value()?;
    let lit = require_lit_int(&named.value)?;
    Ok(lit)
}

fn parse_variant_doc(attr: &syn::Attribute) -> syn::Result<String> {
    let named = attr.meta.require_name_value()?;
    let lit = require_lit_str(&named.value)?;
    Ok(lit.value().trim().to_string())
}

impl ErrEnum {
    pub fn expand(self) -> syn::Result<proc_macro2::TokenStream> {
        let ident = &self.ident;
        let kinds = self.gen_err_kinds();
        let (all_variants, name_arms) = self
            .variants
            .iter()
            .map(|v| {
                let variant = &v.ident;
                let v_str = variant.to_string();
                (
                    quote!(&BizError::#variant),
                    quote!(
                        #v_str => BizError::#variant,
                    ),
                )
            })
            .unzip::<_, _, Vec<_>, Vec<_>>();

        let vis = &self.vis;
        let stream = quote_spanned! { self.ident.span() =>
            #[allow(non_upper_case_globals)]
            #vis trait #ident {
                #(#kinds)*

                fn all_in_scope() -> &'static [&'static BizError] {
                    use lolibaso::http::error::BizError;
                    static ALL: &[&BizError] = &[#(#all_variants),*];
                    ALL
                }

                fn try_from_name<S: AsRef<str>>(name: &str, ctx: Option<S>) -> Option<lolibaso::http::error::BizError> {
                    let err = match name {
                        #(#name_arms)*
                        _ => return None,
                    };

                    if let Some(ctx) = ctx {
                        Some(err.with_context(ctx))
                    } else {
                        Some(err)
                    }
                }
            }

            impl #ident for lolibaso::http::error::BizError {}
        };

        Ok(stream)
    }

    fn gen_err_kinds(&self) -> Vec<proc_macro2::TokenStream> {
        let mut kinds = vec![];

        for (index, variant) in self.variants.iter().enumerate() {
            let v_ident = &variant.ident;
            let biz_code = self.base_biz_code + (index as u32) + 1;
            let desc = variant.desc.clone().unwrap_or_else(|| {
                let mut desc = v_ident.to_string().to_case(Case::Lower);
                let first_char = desc.chars().next().unwrap().to_uppercase();
                desc.replace_range(0..1, &first_char.to_string());
                desc
            });
            let http_status = variant
                .http_status
                .or(self.default_http_status)
                .unwrap_or(400);
            let kind = quote_spanned! { variant.ident.span() =>
                const #v_ident: BizError = BizError::new(#http_status, #biz_code, #desc);
            };
            kinds.push(kind);
        }

        kinds
    }
}
