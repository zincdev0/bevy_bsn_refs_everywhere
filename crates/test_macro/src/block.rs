use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Result, Token,
    parse::{Parse, ParseStream, discouraged::Speculative},
};

use crate::Stmt;

#[derive(Debug)]
pub(crate) struct Block {
    kind: BlockKind,
    raw: BlockRaw,
}

impl Parse for Block {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        let kind = fork.parse()?;
        let raw = fork.parse()?;
        input.advance_to(&fork);
        Ok(Block { kind, raw })
    }
}

impl ToTokens for Block {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.kind.to_tokens(tokens);
        self.raw.to_tokens(tokens);
    }
}

#[derive(Debug)]
pub(crate) struct BlockRaw(Vec<Stmt>);

impl Parse for BlockRaw {
    fn parse(input: ParseStream) -> Result<Self> {
        todo!();
    }
}

impl ToTokens for BlockRaw {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let stmts = &self.0;
        quote! { { #(#stmts)* } }.to_tokens(tokens);
    }
}

#[derive(Debug)]
pub(crate) enum BlockKind {
    Default,
    Async,
    Const,
    Loop,
    Try,
    Unsafe,
}

impl Parse for BlockKind {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.parse::<Token![async]>().is_ok() {
            Ok(BlockKind::Async)
        } else if input.parse::<Token![const]>().is_ok() {
            Ok(BlockKind::Const)
        } else if input.parse::<Token![loop]>().is_ok() {
            Ok(BlockKind::Loop)
        } else if input.parse::<Token![try]>().is_ok() {
            Ok(BlockKind::Try)
        } else if input.parse::<Token![unsafe]>().is_ok() {
            Ok(BlockKind::Unsafe)
        } else {
            Ok(BlockKind::Default)
        }
    }
}

impl ToTokens for BlockKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            BlockKind::Default => {}
            BlockKind::Async => {
                quote! { async }.to_tokens(tokens);
            }
            BlockKind::Const => {
                quote! { const }.to_tokens(tokens);
            }
            BlockKind::Loop => {
                quote! { loop }.to_tokens(tokens);
            }
            BlockKind::Try => {
                quote! { try }.to_tokens(tokens);
            }
            BlockKind::Unsafe => {
                quote! { unsafe }.to_tokens(tokens);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::block::BlockKind;

    #[test]
    fn parse() {
        use crate::assert_parse as a;

        a! { _, Ok(BlockKind::Async), async };
        a! { _, Ok(BlockKind::Const), const };
        a! { _, Ok(BlockKind::Default), };
        a! { _, Ok(BlockKind::Loop), loop };
        a! { _, Ok(BlockKind::Try), try };
        a! { _, Ok(BlockKind::Unsafe), unsafe };

        a! { BlockKind, Err(_), foo };
    }

    #[test]
    fn to_tokens() {
        use crate::assert_to_tokens as a;
        a! { BlockKind::Async, async };
        a! { BlockKind::Const, const };
        a! { BlockKind::Default, };
        a! { BlockKind::Loop, loop };
        a! { BlockKind::Try, try };
        a! { BlockKind::Unsafe, unsafe };
    }
}
