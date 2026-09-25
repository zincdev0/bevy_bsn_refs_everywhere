use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Result, bracketed,
    parse::{Parse, ParseStream, discouraged::Speculative},
};

use crate::Expr;

#[derive(Debug)]
pub(crate) struct Index(Box<Expr>);

impl Parse for Index {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        let content;
        bracketed!(content in fork);
        let index = content.parse()?;
        if !content.is_empty() {
            return Err(input.error("unexpected trailing tokens in indexing expression"));
        }
        input.advance_to(&fork);
        Ok(Index(index))
    }
}

impl ToTokens for Index {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Index(index) = self;
        quote! { [#index] }.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;

    use crate::{Index, expr::ExprBase};

    #[test]
    fn parse() {
        use crate::assert_parse as a;
        fn a(tokens: TokenStream) -> Index {
            syn::parse2(tokens).unwrap()
        }

        assert!(matches!(a(quote! { [0123] }).0.base, ExprBase::Lit(_)));
        assert!(matches!(a(quote! { ["foo"] }).0.base, ExprBase::Lit(_)));

        a! { Index, Err(_), };
        a! { Index, Err(_), [0123 +] };
    }

    #[test]
    fn parse_is_polite() {
        use crate::test::assert_respect;
        assert_respect::<Index>(quote! { [0123 +] });
    }

    #[test]
    fn to_tokens() {
        todo!();
        // a! { Index { index: Box::new(expr(ExprBase::Lit(quote! { 0123 }))) }, 123 };
        // a! { Index { index: Box::new(expr(ExprBase::Lit(quote! { "foo" }))) }, "foo" };
    }
}
