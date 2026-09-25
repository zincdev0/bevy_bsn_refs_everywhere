use proc_macro2::{TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::{
    Result, Token,
    parse::{Parse, ParseStream, discouraged::Speculative},
};

use crate::Expr;

#[derive(Debug)]
pub(crate) struct Closure {
    pub(crate) has_async: bool,
    pub(crate) has_move: bool,
    pub(crate) pats: TokenStream,
    pub(crate) expr: Box<Expr>,
}

impl Parse for Closure {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();

        let has_async = fork.parse::<Token![async]>().is_ok();
        let has_move = fork.parse::<Token![move]>().is_ok();

        fork.parse::<Token![|]>()?;
        let mut pats = TokenStream::new();
        while !fork.peek(Token![|]) {
            pats.extend([fork.parse::<TokenTree>()?]);
        }
        fork.parse::<Token![|]>()?;

        let expr = fork.parse()?;

        input.advance_to(&fork);
        Ok(Closure {
            has_async,
            has_move,
            pats,
            expr,
        })
    }
}

impl ToTokens for Closure {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Closure {
            has_async,
            has_move,
            pats,
            expr,
        } = self;
        let has_async = has_async.then(|| quote! { async });
        let has_move = has_move.then(|| quote! { move });
        quote! {
            #has_async #has_move |#pats| #expr
        }
        .to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;

    use crate::{Closure, expr::ExprBase};

    #[test]
    fn parse() {
        use crate::{assert_parse as a, test::assert_tokens};
        fn a(tokens: TokenStream) -> Closure {
            syn::parse2(tokens).unwrap()
        }

        let closure = a(quote! { || 0123});
        assert_eq!(closure.has_async, false);
        assert_eq!(closure.has_move, false);
        assert!(closure.pats.is_empty());
        assert!(matches!(closure.expr.base, ExprBase::Lit(_)));

        let closure = a(quote! { async || 0123 });
        assert_eq!(closure.has_async, true);
        assert_eq!(closure.has_move, false);

        let closure = a(quote! { move || 0123 });
        assert_eq!(closure.has_async, false);
        assert_eq!(closure.has_move, true);

        let closure = a(quote! { async move || 0123 });
        assert_eq!(closure.has_async, true);
        assert_eq!(closure.has_move, true);

        let closure = a(quote! { |ident: Type| 0123 });
        assert_tokens(closure.pats, quote! { ident: Type });

        a! { Closure, Err(_), move async || 0123 };
    }

    #[test]
    fn parse_is_polite() {
        use crate::test::assert_respect;

        assert_respect::<Closure>(quote! { |abc });
        assert_respect::<Closure>(quote! { |abc| });
    }

    #[test]
    fn to_tokens() {
        todo!();
        // a! {
        //     Closure {
        //         has_async: false,
        //         has_move: false,
        //         pats: TokenStream::new(),
        //         expr: Box::new(expr(ExprBase::Lit(quote! { 0123 }))),
        //     },
        //     || 0123,
        // };
        // a! {
        //     Closure {
        //         has_async: true,
        //         has_move: false,
        //         pats: TokenStream::new(),
        //         expr: Box::new(expr(ExprBase::Lit(quote! { 0123 }))),
        //     },
        //     async || 0123,
        // };
        // a! {
        //     Closure {
        //         has_async: false,
        //         has_move: true,
        //         pats: TokenStream::new(),
        //         expr: Box::new(expr(ExprBase::Lit(quote! { 0123 }))),
        //     },
        //     move || 0123,
        // };
        // a! {
        //     Closure {
        //         has_async: true,
        //         has_move: true,
        //         pats: TokenStream::new(),
        //         expr: Box::new(expr(ExprBase::Lit(quote! { 0123 }))),
        //     },
        //     async move || 0123,
        // };
        // a! {
        //     Closure {
        //         has_async: false,
        //         has_move: false,
        //         pats: quote! { ident: Type },
        //         expr: Box::new(expr(ExprBase::Lit(quote! { 0123 }))),
        //     },
        //     |ident: Type| 0123,
        // };
    }
}
