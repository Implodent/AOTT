use proc_macro::TokenStream as TS;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
        parse::{Parse, Parser},
        ItemFn,
};

#[proc_macro]
pub fn parser(ts: TS) -> TS {
        let ts: TokenStream = ts.into();

        parser_impl(ts)
                .map_or_else(|e| e.to_compile_error(), |t| Into::<TokenStream>::into(t))
                .into()
}

fn parser_impl(ts: TokenStream) -> Result<TokenStream, syn::Error> {
        let f = ItemFn::parse.parse2(ts)?;
        Ok(quote! {
            #f
        })
}
