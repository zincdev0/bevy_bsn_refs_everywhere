use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Result, Token, bracketed,
    parse::{Parse, ParseStream, discouraged::Speculative},
    token::Bracket,
};

use crate::{Array, Block, Closure, ForLoop, Prefix, Suffix};

#[derive(Debug)]
pub(crate) struct Expr {
    pub(crate) attrs: TokenStream,
    pub(crate) prefixes: Vec<Prefix>,
    pub(crate) base: ExprBase,
    pub(crate) suffixes: Vec<Suffix>,
}

impl Parse for Expr {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();

        let mut attrs = TokenStream::new();
        loop {
            let fork_1 = fork.fork();
            if fork_1.parse::<Token![#]>().is_err() {
                break;
            }
            let inner = fork_1.parse::<Token![!]>().ok();

            if !fork_1.peek(Bracket) {
                break;
            }
            let meta;
            bracketed!(meta in fork_1);
            let meta = meta.parse::<TokenStream>().unwrap();

            fork.advance_to(&fork_1);
            quote! { # #inner [#meta] }.to_tokens(&mut attrs);
        }

        let mut prefixes = Vec::new();
        while let Ok(prefix) = fork.parse::<Prefix>() {
            prefixes.push(prefix);
        }

        let base = fork.parse()?;
        input.advance_to(&fork);

        let mut suffixes = Vec::new();
        while let Ok(suffix) = input.parse::<Suffix>() {
            suffixes.push(suffix);
        }

        Ok(Expr {
            attrs,
            prefixes,
            base,
            suffixes,
        })
    }
}

impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Expr {
            attrs,
            prefixes,
            base,
            suffixes,
        } = self;
        quote! {
            #attrs
            #(#prefixes)*
            #base
            #(#suffixes)*
        }
        .to_tokens(tokens);
    }
}

#[expect(unused)]
#[derive(Debug)]
pub(crate) enum ExprBase {
    Array(Array),
    Block(Block),
    Closure(Closure),
    ForLoop(ForLoop),
    IfElse(),
    Let(),
    Lit(TokenStream),
    Macro(),
    Match(),
    Paren(Box<Expr>),
    Range(),
    Repeat(),
    Struct(),
    Tuple(Vec<Expr>),
    While(),
}

impl Parse for ExprBase {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Ok(array) = input.parse() {
            Ok(ExprBase::Array(array))
        } else if let Ok(block) = input.parse() {
            Ok(ExprBase::Block(block))
        } else if let Ok(closure) = input.parse() {
            Ok(ExprBase::Closure(closure))
        } else if let Ok(for_loop) = input.parse() {
            Ok(ExprBase::ForLoop(for_loop))
        } else if let Ok(lit) = input.parse::<syn::Lit>().map(ToTokens::into_token_stream) {
            Ok(ExprBase::Lit(lit))
        } else {
            Err(input.error("TODO: error strings"))
        }
    }
}

impl ToTokens for ExprBase {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ExprBase::Array(array) => array.to_tokens(tokens),
            ExprBase::Block(block) => block.to_tokens(tokens),
            ExprBase::Closure(closure) => closure.to_tokens(tokens),
            ExprBase::ForLoop(for_loop) => for_loop.to_tokens(tokens),
            ExprBase::IfElse() => todo!(),
            ExprBase::Let() => todo!(),
            ExprBase::Lit(lit) => lit.to_tokens(tokens),
            ExprBase::Macro() => todo!(),
            ExprBase::Match() => todo!(),
            ExprBase::Paren(expr) => quote! { (#expr) }.to_tokens(tokens),
            ExprBase::Range() => todo!(),
            ExprBase::Repeat() => todo!(),
            ExprBase::Struct() => todo!(),
            ExprBase::Tuple(exprs) => quote! { (#(#exprs),*) }.to_tokens(tokens),
            ExprBase::While() => todo!(),
        };
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;

    use crate::{Expr, Prefix, Suffix, expr::ExprBase};

    #[test]
    fn parse() {
        use crate::test::assert_tokens;
        fn a(tokens: TokenStream) -> Expr {
            syn::parse2(tokens).unwrap()
        }

        let expr = a(quote! { 0123 });
        assert!(expr.attrs.is_empty());
        assert!(expr.prefixes.is_empty());
        assert!(matches!(expr.base, ExprBase::Lit(_)));
        assert!(expr.suffixes.is_empty());

        let expr = a(quote! { #[meta] 0123 });
        assert_tokens(expr.attrs, quote! { #[meta] });
        assert!(expr.prefixes.is_empty());
        assert!(matches!(expr.base, ExprBase::Lit(_)));
        assert!(expr.suffixes.is_empty());

        let expr = a(quote! { #[meta(foo = [0123])] 0123 });
        assert_tokens(expr.attrs, quote! { #[meta(foo = [0123])] });

        let expr = a(quote! { #[meta] #[meta_foo] 0123 });
        assert_tokens(expr.attrs, quote! { #[meta] #[meta_foo] });

        let expr = a(quote! { #![meta] 0123 });
        assert_tokens(expr.attrs, quote! { #![meta] });

        let expr = a(quote! { !0123 });
        assert_eq!(expr.prefixes.len(), 1);
        assert!(matches!(expr.prefixes[0], Prefix::Not));

        let expr = a(quote! { !!!0123 });
        assert_eq!(expr.prefixes.len(), 3);
        assert!(matches!(expr.prefixes[0], Prefix::Not));
        assert!(matches!(expr.prefixes[1], Prefix::Not));
        assert!(matches!(expr.prefixes[2], Prefix::Not));

        let expr = a(quote! { 0123? });
        assert_eq!(expr.suffixes.len(), 1);
        assert!(matches!(expr.suffixes[0], Suffix::Try));

        let expr = a(quote! { 0123??? });
        assert_eq!(expr.suffixes.len(), 3);
        assert!(matches!(expr.suffixes[0], Suffix::Try));
        assert!(matches!(expr.suffixes[1], Suffix::Try));
        assert!(matches!(expr.suffixes[2], Suffix::Try));

        let expr = a(quote! { #[meta] #[meta] !!!0123??? });
        assert_tokens(expr.attrs, quote! { #[meta] #[meta] });
        assert_eq!(expr.prefixes.len(), 3);
        assert!(matches!(expr.prefixes[0], Prefix::Not));
        assert!(matches!(expr.prefixes[1], Prefix::Not));
        assert!(matches!(expr.prefixes[2], Prefix::Not));
        assert!(matches!(expr.base, ExprBase::Lit(_)));
        assert_eq!(expr.suffixes.len(), 3);
        assert!(matches!(expr.suffixes[0], Suffix::Try));
        assert!(matches!(expr.suffixes[1], Suffix::Try));
        assert!(matches!(expr.suffixes[2], Suffix::Try));
    }

    #[test]
    fn parse_is_polite() {
        use crate::test::assert_respect;
        assert_respect::<Expr>(quote! { #[meta] });
        assert_respect::<Expr>(quote! { #[meta] * });
        assert_respect::<Expr>(quote! { #[meta] *&raw & });
    }

    #[test]
    fn to_tokens() {
        todo!();
        // assert_to_tokens(
        //     Expr {
        //         attrs: quote! { #![meta] #[meta] },
        //         prefixes: vec![Prefix::Not, Prefix::Neg],
        //         base: ExprBase::Lit(quote! { 0123 }),
        //         suffixes: vec![
        //             Suffix::Try,
        //             Suffix::Binary(Binary {
        //                 op: BinaryOp::Add,
        //                 expr: Box::new(expr(ExprBase::Lit(quote! { "foo" }))),
        //             }),
        //         ],
        //     },
        //     "#![meta] #[meta] !-0123? + \"foo\"",
        // );
    }
}
