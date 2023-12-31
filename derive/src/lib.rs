//! do not read till the end if you want to keep your sanity.
//! please.
//! i beg you.
//! WARNING: unformatted shitcode ahead.
//! fix PRs accepted.

use std::str::FromStr;

use proc_macro::TokenStream as TS;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
        parse::{Parse, Parser},
        punctuated::Punctuated,
        token::Comma,
        AngleBracketedGenericArguments, Expr, ExprLit, ExprPath, Fields, FnArg, GenericArgument,
        GenericParam, ItemFn, Lifetime, LifetimeParam, Lit, LitStr, Meta, MetaNameValue, Path,
        PathArguments, PathSegment, ReturnType, Type, TypePath, TypeReference, Variant,
};

// example usage:
// ```
// #[parser(extras = Extras)]
// fn case(input: Stream<Token>) -> Case {}
// ```
// into
// ```
// fn case<'input>(input: Input<'input, Stream<Token>, Extras>) -> IResult<'input, Stream<Token>, Extras, Case> {}
// ```

#[proc_macro_attribute]
pub fn parser(args: TS, ts: TS) -> TS {
        let ts: TokenStream = ts.into();

        parser_impl(args.into(), ts)
                .map_or_else(|e| e.to_compile_error(), Into::<TokenStream>::into)
                .into()
}

fn map_ty(lft: Lifetime, mut ty: Type) -> Type {
        match ty {
                Type::Reference(ref mut r) => {
                        let _ = r.lifetime.get_or_insert(lft);
                }
                _ => (),
        };
        ty
}

fn parser_impl(args: TokenStream, ts: TokenStream) -> Result<TokenStream, syn::Error> {
        let meta: Punctuated<Meta, Comma> = Punctuated::parse_terminated.parse2(args)?;
        let mut f = ItemFn::parse.parse2(ts)?;
        let mut inline = true;
        let extras = meta.iter().find_map(|meth| match meth {
                Meta::NameValue(MetaNameValue {
                        path,
                        value: Expr::Path(ExprPath { path: ext, .. }),
                        ..
                }) if path.is_ident(&Ident::new("extras", Span::call_site())) => {
                        Some(ext.to_token_stream().to_string())
                }
                Meta::Path(path) if path.is_ident(&Ident::new("noinline", Span::call_site())) => {
                        inline = false;
                        None
                }
                Meta::NameValue(MetaNameValue {
                        path,
                        value:
                                Expr::Lit(ExprLit {
                                        lit: Lit::Str(lit), ..
                                }),
                        ..
                }) if path.is_ident(&Ident::new("extras", Span::call_site())) => {
                        Some(lit.value().to_owned())
                }
                _ => None,
        });
        let mut lifetimes = vec![];
        let lifetime = Lifetime::new("'a", Span::call_site());
        let mut inputs = vec![];
        for inp in f.sig.inputs {
                match inp {
                        FnArg::Receiver(_) => {}
                        FnArg::Typed(mut pat) => {
                                let ty = match *pat.ty {
                                        Type::Reference(mut r) => {
                                                if r.lifetime.is_none() {
                                                        r.lifetime = Some(lifetime.clone());

                                                        lifetimes.push(lifetime.clone());
                                                }
                                                Type::Reference(r)
                                        }
                                        t => t,
                                };
                                let extras = extras
                                        .map(|t| {
                                                Type::parse
                                                        .parse2(TokenStream::from_str(&t).unwrap())
                                                        .unwrap()
                                        })
                                        .unwrap_or_else(|| {
                                                Type::parse
                                                        .parse2(quote!(::aott::extra::Err<#ty>))
                                                        .unwrap()
                                        });
                                let ty_for_ret = ty.clone();
                                let extras_for_ret = extras.clone();
                                let output = f.sig.output;
                                f.sig.output = ReturnType::Type(Default::default(), Box::new(Type::Path(TypePath { qself: None, path: Path {leading_colon: None, segments: [PathSegment { ident: Ident::new("aott", Span::call_site()), arguments: PathArguments::None }, PathSegment {ident: Ident::new("PResult", Span::call_site()), arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments { colon2_token: None, gt_token: Default::default(), lt_token: Default::default(), args: Punctuated::from_iter([GenericArgument::Type(ty_for_ret), GenericArgument::Type(match output { ReturnType::Default => Type::Tuple(syn::TypeTuple { paren_token: Default::default(), elems: Punctuated::new() }), ReturnType::Type(_, t) => map_ty(lifetime, *t) }), GenericArgument::Type(extras_for_ret)]) })}].into_iter().collect()} })));

                                pat.ty = Box::new(Type::Reference(TypeReference {
                                        elem: Box::new(Type::Path(TypePath {
                                                qself: None,
                                                path: Path {
                                                        leading_colon: None,
                                                        segments: [PathSegment { ident: Ident::new("aott", Span::call_site()), arguments: PathArguments::None }, PathSegment { ident: Ident::new("input", Span::call_site()), arguments: PathArguments::None }, PathSegment {
                                                                ident: Ident::new("Input", Span::call_site()),
                                                                arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                                                                        args: Punctuated::from_iter([
                                                                                GenericArgument::Type(ty),
                                                                                GenericArgument::Type(extras)
                                                                        ]),
                                                                        colon2_token: None,
                                                                        lt_token: Default::default(),
                                                                        gt_token: Default::default(),
                                                                })
                                                        }].into_iter().collect(),
                                                },
                                        })),
                                        and_token: Default::default(),
                                        lifetime: None,
                                        mutability: Some(Default::default())
                                }));
                                inputs.push(FnArg::Typed(pat));
                                break;
                        }
                }
        }
        f.sig.inputs = inputs.into_iter().collect();
        f.sig.generics
                .params
                .extend(lifetimes.into_iter().map(|lifetime| {
                        GenericParam::Lifetime(LifetimeParam {
                                attrs: vec![],
                                lifetime,
                                colon_token: None,
                                bounds: Punctuated::new(),
                        })
                }));
        let inl = inline.then(|| quote!(#[inline(always)]));
        Ok(quote! {
            #inl
            #f
        })
}

#[proc_macro_derive(IntoString)]
pub fn into_string(input: TS) -> TS {
        let input: TokenStream = input.into();

        into_string_impl(input)
                .map_or_else(|e| e.to_compile_error(), Into::<TokenStream>::into)
                .into()
}

fn into_string_impl(input: TokenStream) -> syn::Result<TokenStream> {
        let input = syn::DeriveInput::parse.parse2(input)?;
        let ident = input.ident;

        let variants = match input.data {
                syn::Data::Enum(en) => en
                        .variants
                        .iter()
                        .map(
                                |Variant {
                                         fields,
                                         ident,
                                         attrs,
                                         ..
                                 }| match fields {
                                        Fields::Unnamed(_) => {
                                                quote!(Self::#ident(bruh) => bruh)
                                        }
                                        Fields::Unit => {
                                                if let Some(token_attr) = attrs.iter().find(|x| {
                                                        let wtf = x.path().get_ident().unwrap().to_string();
                                                        matches!(wtf.as_str(), "token" | "regex")
                                                }) {
                                                        let real = LitStr::new(&token_attr
                                                                .parse_args::<LitStr>().unwrap().value(), Span::call_site());

                                                        return quote!(Self::#ident => String::from(#real))
                                                } else {
                                                        panic!("no token attr on unit variant {ident}")
                                                }
                                        }
                                        _ => panic!("unsupported unit variant or whatever"),
                                },
                        )
                        .collect::<Vec<_>>(),
                _ => panic!("unsupported"),
        };

        Ok(quote!(impl Into<String> for #ident {
                fn into(self) -> String {
                        match self {
                                #(#variants,)*
                        }
                }
        }))
}
