use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Result, Token, bracketed,
    parse::{Parse, ParseStream, discouraged::Speculative},
    punctuated::Punctuated,
};

use crate::Expr;

#[derive(Debug)]
pub(crate) enum Array {
    Values(Vec<Expr>),
    Repeat { value: Box<Expr>, amount: Box<Expr> },
}

impl Parse for Array {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        let content;
        bracketed!(content in fork);

        let fork_1 = content.fork();
        if let Ok(exprs) = Punctuated::<Expr, Token![,]>::parse_terminated(&fork_1) {
            content.advance_to(&fork_1);
            input.advance_to(&fork);
            return Ok(Array::Values(exprs.into_iter().collect()));
        }

        let value = content.parse()?;
        _ = content.parse::<Token![;]>()?;
        let amount = content.parse()?;
        input.advance_to(&fork);
        Ok(Array::Repeat { value, amount })
    }
}

impl ToTokens for Array {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Array::Values(values) => quote! { [#(#values),*] },
            Array::Repeat { value, amount } => quote! { [#value; #amount] },
        }
        .to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;

    use crate::{Array, expr::ExprBase};

    #[test]
    fn parse() {
        fn a(tokens: TokenStream) -> Array {
            syn::parse2(tokens).unwrap()
        }

        let Array::Values(values) = a(quote! { [] }) else {
            panic!();
        };
        assert!(values.is_empty());

        let Array::Values(values) = a(quote! { [0123] }) else {
            panic!();
        };
        assert_eq!(values.len(), 1);
        assert!(matches!(values[0].base, ExprBase::Lit(_)));

        let Array::Values(values) = a(quote! { [0123,] }) else {
            panic!();
        };
        assert_eq!(values.len(), 1);
        assert!(matches!(values[0].base, ExprBase::Lit(_)));

        let Array::Values(values) = a(quote! { [0123, 0123] }) else {
            panic!();
        };
        assert_eq!(values.len(), 2);
        assert!(matches!(values[0].base, ExprBase::Lit(_)));
        assert!(matches!(values[1].base, ExprBase::Lit(_)));

        let Array::Repeat { value, amount } = a(quote! { ["foo"; 0123] }) else {
            panic!();
        };
        assert!(matches!(value.base, ExprBase::Lit(_)));
        assert!(matches!(amount.base, ExprBase::Lit(_)));
    }

    #[test]
    fn parse_is_polite() {
        use crate::test::assert_respect;
        assert_respect::<Array>(quote! { [<invalid expr>] });
    }

    #[test]
    fn to_tokens() {
        use crate::assert_to_tokens as a;
        a! { Array::Values(vec![]), [] };
        todo!();
        // a! { Array::Values(vec![expr(ExprBase::Lit(quote! { 0123 }))]), [123] };
        // a! { Array::Values(vec![expr(ExprBase::Lit(quote! { 0123 })), expr(ExprBase::Lit(quote! { "foo" }))]), [123, "foo"] };
        // a! { Array::Repeat { value: expr(ExprBase::Lit(quote! { "foo" })), amount: expr(ExprBase::Lit(quote! { 0123 })) }, ["foo"; 123] };
    }
}
