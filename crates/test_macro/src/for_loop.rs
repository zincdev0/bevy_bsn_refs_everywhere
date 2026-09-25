use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Result, Token,
    parse::{Parse, ParseStream, discouraged::Speculative},
};

use crate::{BlockRaw, Expr};

#[derive(Debug)]
pub(crate) struct ForLoop {
    pat: TokenStream,
    iter: Box<Expr>,
    block: BlockRaw,
}

impl Parse for ForLoop {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        fork.parse::<Token![for]>()?;
        let pat = syn::Pat::parse_single(&fork)?.into_token_stream();
        fork.parse::<Token![in]>()?;
        let iter = fork.parse()?;
        let block = fork.parse()?;
        input.advance_to(&fork);
        Ok(ForLoop { pat, iter, block })
    }
}

impl ToTokens for ForLoop {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ForLoop { pat, iter, block } = self;
        quote! { for #pat in #iter #block }.to_tokens(tokens);
    }
}
