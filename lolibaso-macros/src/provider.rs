use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    DeriveInput, Ident, Lifetime, PredicateType, Token, parse::Parse, parse_quote,
    punctuated::Punctuated, spanned::Spanned,
};

pub fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_ident = &input.ident;
    let ds = match &input.data {
        syn::Data::Struct(ds) => ds,
        syn::Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "Enums are not supported to derive Provider",
            ));
        }
        syn::Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "Unions are not supported to derive Provider",
            ));
        }
    };

    let mut fields = vec![];
    for (idx, f) in ds.fields.iter().enumerate() {
        fields.push(ProviderField::from_syn(f, idx)?);
    }
    let (impl_generics, ty_generics, where_clause) = &input.generics.split_for_impl();

    let mut where_clause = where_clause.cloned().unwrap_or_else(|| syn::WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });

    for f in &fields {
        where_clause.predicates.push(f.impl_bound());
    }

    let stream = quote! {
        impl #impl_generics lolibaso::provider::Provider for #struct_ident #ty_generics #where_clause {
            fn build(ctx: &mut lolibaso::provider::ProviderContext) -> anyhow::Result<Self> {
                let this = #struct_ident {
                    #(#fields),*
                };
                Ok(this)
            }
        }
    };

    Ok(stream)
}

#[derive(Debug)]
struct ProviderField {
    name: syn::Ident,
    ty: syn::Type,
    kind: FieldKind,
}

#[derive(Debug)]
enum FieldKind {
    Provider,
    Instance,
    Default,
    With(syn::Expr),
}

impl ToTokens for ProviderField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let ty = &self.ty;

        self.name.to_tokens(tokens);
        Token![:](Span::call_site()).to_tokens(tokens);
        match &self.kind {
            FieldKind::Provider => {
                let path: syn::Expr = parse_quote!(lolibaso::provider::Provider::build(ctx)?);
                path.to_tokens(tokens);
            }
            FieldKind::Instance => {
                let expr: syn::Expr = parse_quote! {
                    match ctx.remove() {
                        Some(val) => val,
                        None => {
                            ::anyhow::bail!("Provider::build: Instance not found. field = {}. type = {}", stringify!(#name), stringify!(#ty))
                        }
                    }
                };
                expr.to_tokens(tokens);
            }
            FieldKind::Default => {
                let expr: syn::Expr = parse_quote! {
                    Default::default()
                };
                expr.to_tokens(tokens);
            }
            FieldKind::With(expr) => {
                expr.to_tokens(tokens);
            }
        }
    }
}

struct ProvideWithExpr(syn::Expr);

impl Parse for ProvideWithExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let with = input.parse::<syn::Ident>()?;
        if with.to_string() != "with" {
            return Err(syn::Error::new_spanned(with, "expected with"));
        }
        let _eq = input.parse::<syn::Token![=]>()?;
        let expr = input.parse::<syn::Expr>()?;
        Ok(ProvideWithExpr(expr))
    }
}

impl ProviderField {
    fn from_syn(f: &syn::Field, idx: usize) -> syn::Result<Self> {
        let mut kind = FieldKind::Provider;
        for attr in f.attrs.clone() {
            if attr.path().is_ident("provider") {
                if let Ok(ProvideWithExpr(expr)) = attr.parse_args::<ProvideWithExpr>() {
                    kind = FieldKind::With(expr);
                    continue;
                }
                let ident = attr.parse_args::<syn::Ident>()?;
                let ident = ident.to_string();
                match ident.as_str() {
                    "instance" => kind = FieldKind::Instance,
                    "default" => kind = FieldKind::Default,
                    _ => return Err(syn::Error::new_spanned(attr, "unknown Provider attribute")),
                }
            }
        }
        let ident = match f.ident.clone() {
            Some(i) => i,
            None => Ident::new(&idx.to_string(), f.span()),
        };

        Ok(ProviderField {
            name: ident,
            ty: f.ty.clone(),
            kind,
        })
    }

    fn impl_bound(&self) -> syn::WherePredicate {
        let mut bounds = vec![];
        match &self.kind {
            FieldKind::Provider => {
                bounds.push(parse_quote!(::lolibaso::provider::Provider));
            }
            FieldKind::Instance => {
                bounds.push(lifetime_bound("'static"));
            }
            FieldKind::Default => {
                bounds.push(trait_bound(parse_quote!(Default)));
                bounds.push(lifetime_bound("'static"));
            }
            FieldKind::With(_) => {}
        };

        syn::WherePredicate::Type(PredicateType {
            lifetimes: Default::default(),
            bounded_ty: Clone::clone(&self.ty),
            colon_token: Default::default(),
            bounds: Punctuated::from_iter(bounds),
        })
    }
}

fn lifetime_bound(lifetime: &str) -> syn::TypeParamBound {
    syn::TypeParamBound::Lifetime(Lifetime::new(lifetime, Span::call_site()))
}

fn trait_bound(path: syn::Path) -> syn::TypeParamBound {
    syn::TypeParamBound::Trait(syn::TraitBound {
        paren_token: Default::default(),
        modifier: syn::TraitBoundModifier::None,
        lifetimes: Default::default(),
        path,
    })
}
